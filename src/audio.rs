//! Аудиовход через cpal (md_plans/10 R6). Приоритет — устройства, включая monitor-источники
//! (loopback системного выхода). Сэмплы даунмиксятся в моно и пушатся в lock-free кольцо.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::Producer;
use ringbuf::HeapProd;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

/// Частота, которую запрашиваем у parec (PulseAudio ресемплит источник под неё).
pub const PULSE_RATE: u32 = 48000;

/// Список входных устройств. Monitor-источники (в PipeWire/Pulse — `*.monitor`) тоже сюда попадают.
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    let mut out = Vec::new();
    if let Ok(devs) = host.input_devices() {
        for d in devs {
            if let Ok(name) = d.name() {
                out.push(name);
            }
        }
    }
    out
}

/// Похоже ли имя на monitor/loopback-источник.
pub fn is_monitor(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("monitor") || n.contains("loopback")
}

/// PulseAudio-источник (в т.ч. `.monitor` выходных устройств).
#[derive(Clone, Debug)]
pub struct PulseSource {
    pub name: String,
    pub is_monitor: bool,
}

/// Список источников PulseAudio через `pactl list short sources`.
/// На Pulse/PipeWire это единственный способ увидеть monitor выходов (cpal/ALSA их не отдаёт).
pub fn list_pulse_sources() -> Vec<PulseSource> {
    let mut out = Vec::new();
    let Ok(res) = std::process::Command::new("pactl").args(["list", "short", "sources"]).output() else {
        return out;
    };
    let text = String::from_utf8_lossy(&res.stdout);
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 2 {
            let name = cols[1].to_string();
            let is_monitor = name.ends_with(".monitor") || is_monitor(&name);
            out.push(PulseSource { name, is_monitor });
        }
    }
    out
}

/// Бэкенд захвата: либо cpal-поток (железо), либо процесс `parec` (Pulse-источник/monitor).
/// Держится живым ради RAII (остановка потока/процесса при drop), напрямую не читается.
#[allow(dead_code)]
enum Backend {
    Cpal(cpal::Stream),
    Pulse(PulseCapture),
}

/// Захват через утилиту `parec` — надёжный путь для Pulse-мониторов в обход cpal/ALSA-плагина
/// `pulse` (который в cpal 0.15 паникует на htstamp). См. md_plans/11.
struct PulseCapture {
    child: Child,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}
impl Drop for PulseCapture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Открытый вход. Держите структуру живой, пока нужен звук.
pub struct AudioInput {
    _backend: Backend,
    pub sample_rate: f32,
    #[allow(dead_code)]
    pub channels: u16,
    pub device_name: String,
}

impl AudioInput {
    /// Открыть источник и пушить моно-сэмплы в `producer`.
    ///
    /// Если задан `pulse_source` — захватываем его через `parec` (в т.ч. `.monitor` системного
    /// выхода, md_plans/10 R6). Иначе — cpal (ALSA-устройство или default).
    pub fn open(
        device_name: Option<&str>,
        pulse_source: Option<&str>,
        producer: HeapProd<f32>,
    ) -> Result<Self> {
        if let Some(src) = pulse_source {
            return Self::open_pulse(src, producer);
        }
        Self::open_cpal(device_name, producer)
    }

    /// Захват Pulse-источника через `parec` (float32le, mono).
    fn open_pulse(source: &str, mut producer: HeapProd<f32>) -> Result<Self> {
        let mut child = Command::new("parec")
            .args([
                "-d",
                source,
                "--format=float32le",
                "--channels=1",
                &format!("--rate={PULSE_RATE}"),
                "--latency-msec=20",
                "-n",
                "bitbybeat",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow!("не удалось запустить parec: {e}"))?;

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("нет stdout у parec"))?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let handle = std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut carry: Vec<u8> = Vec::with_capacity(4);
            while !stop2.load(Ordering::Relaxed) {
                match stdout.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        carry.extend_from_slice(&buf[..n]);
                        let full = carry.len() / 4 * 4;
                        let mut i = 0;
                        while i < full {
                            let s = f32::from_le_bytes([
                                carry[i], carry[i + 1], carry[i + 2], carry[i + 3],
                            ]);
                            let _ = producer.try_push(s);
                            i += 4;
                        }
                        carry.drain(..full);
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            _backend: Backend::Pulse(PulseCapture { child, stop, handle: Some(handle) }),
            sample_rate: PULSE_RATE as f32,
            channels: 1,
            device_name: format!("pulse:{source}"),
        })
    }

    /// Захват cpal-устройства (ALSA-железо / default).
    fn open_cpal(device_name: Option<&str>, mut producer: HeapProd<f32>) -> Result<Self> {
        let host = cpal::default_host();

        let device = match device_name {
            Some(name) => host
                .input_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| anyhow!("устройство не найдено: {name}"))?,
            None => host
                .default_input_device()
                .ok_or_else(|| anyhow!("нет входного устройства по умолчанию"))?,
        };
        let name = device.name().unwrap_or_else(|_| "?".into());
        let supported = device.default_input_config()?;
        let sample_rate = supported.sample_rate().0 as f32;
        let channels = supported.channels();
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();

        let err_fn = |e| eprintln!("[audio] ошибка потока: {e}");
        let ch = channels as usize;

        // даунмикс кадра в моно и push
        let push_mono = move |frame: &[f32], prod: &mut HeapProd<f32>| {
            let m: f32 = frame.iter().copied().sum::<f32>() / ch as f32;
            let _ = prod.try_push(m); // переполнение кольца просто дропаем
        };

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    for frame in data.chunks(ch) {
                        push_mono(frame, &mut producer);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    for frame in data.chunks(ch) {
                        let m: f32 = frame.iter().map(|&s| s as f32 / i16::MAX as f32).sum::<f32>() / ch as f32;
                        let _ = producer.try_push(m);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    for frame in data.chunks(ch) {
                        let m: f32 = frame
                            .iter()
                            .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                            .sum::<f32>() / ch as f32;
                        let _ = producer.try_push(m);
                    }
                },
                err_fn,
                None,
            )?,
            other => return Err(anyhow!("неподдерживаемый формат сэмплов: {other:?}")),
        };

        stream.play()?;
        Ok(Self { _backend: Backend::Cpal(stream), sample_rate, channels, device_name: name })
    }
}
