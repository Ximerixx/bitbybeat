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

/// Свести кадр в моно по выбранным каналам (пусто = среднее всех).
#[inline]
fn mono_pick(frame: &[f32], picks: &[usize]) -> f32 {
    if picks.is_empty() {
        frame.iter().copied().sum::<f32>() / frame.len().max(1) as f32
    } else {
        let mut sum = 0.0;
        let mut n = 0;
        for &i in picks {
            if let Some(&s) = frame.get(i) {
                sum += s;
                n += 1;
            }
        }
        if n == 0 { 0.0 } else { sum / n as f32 }
    }
}

/// То же, но с конверсией сэмпла в f32 (для i16/u16).
#[inline]
fn mono_pick_map<T: Copy>(frame: &[T], picks: &[usize], to_f32: impl Fn(T) -> f32) -> f32 {
    if picks.is_empty() {
        frame.iter().map(|&s| to_f32(s)).sum::<f32>() / frame.len().max(1) as f32
    } else {
        let mut sum = 0.0;
        let mut n = 0;
        for &i in picks {
            if let Some(&s) = frame.get(i) {
                sum += to_f32(s);
                n += 1;
            }
        }
        if n == 0 { 0.0 } else { sum / n as f32 }
    }
}

/// Имя устройства cpal (0.18: через `description()`).
fn dev_name(d: &cpal::Device) -> String {
    d.description().map(|desc| desc.name().to_string()).unwrap_or_else(|_| "?".into())
}

/// Реальное число входных каналов устройства (по default-конфигу — то, что откроем).
/// NB: `supported_input_configs()` у ALSA врёт (рекламирует до 64), поэтому не используем.
fn dev_channels(d: &cpal::Device) -> u16 {
    d.default_input_config().map(|c| c.channels()).unwrap_or(0)
}

/// Префикс имени в конфиге для захвата системного вывода («что слышно в колонках»).
///
/// Пустой остаток (просто `loopback:`) означает устройство вывода по умолчанию.
pub const LOOPBACK_PREFIX: &str = "loopback:";

/// Умеет ли ОС отдавать вывод, если открыть устройство вывода как вход.
///
/// Windows: WASAPI включает `AUDCLNT_STREAMFLAGS_LOOPBACK` прозрачно (cpal, host/wasapi).
/// Linux: тем же занимаются monitor-источники PulseAudio ([`list_pulse_sources`]), а
/// ALSA-устройство вывода как вход не открыть — там этот путь просто не предлагается.
pub const fn loopback_supported() -> bool {
    cfg!(target_os = "windows")
}

/// Ссылается ли имя устройства из конфига на захват системного вывода.
pub fn is_loopback_name(name: &str) -> bool {
    name.starts_with(LOOPBACK_PREFIX)
}

/// Имя устройства без служебного префикса — для показа в GUI.
pub fn display_device_name(name: &str) -> &str {
    name.strip_prefix(LOOPBACK_PREFIX).unwrap_or(name)
}

/// Входное устройство cpal с числом каналов.
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    /// Имя для конфига: у захвата вывода — с префиксом [`LOOPBACK_PREFIX`].
    pub name: String,
    pub channels: u16,
    /// Это устройство вывода, открываемое как вход (системный звук).
    pub is_loopback: bool,
}

/// Список того, что можно открыть на запись: входы плюс — где ОС умеет — выходы в режиме
/// loopback. Monitor-источники (в PipeWire/Pulse — `*.monitor`) тоже попадают в первую группу.
pub fn list_input_devices() -> Vec<DeviceInfo> {
    let host = cpal::default_host();
    let mut out = Vec::new();
    if let Ok(devs) = host.input_devices() {
        for d in devs {
            out.push(DeviceInfo { name: dev_name(&d), channels: dev_channels(&d), is_loopback: false });
        }
    }
    if loopback_supported() {
        match host.output_devices() {
            Ok(devs) => {
                for d in devs {
                    // У устройства вывода нет входного конфига — число каналов берём из выходного.
                    let channels = d.default_output_config().map(|c| c.channels()).unwrap_or(0);
                    out.push(DeviceInfo {
                        name: format!("{LOOPBACK_PREFIX}{}", dev_name(&d)),
                        channels,
                        is_loopback: true,
                    });
                }
            }
            Err(e) => crate::diag::warn("audio", format!("список устройств вывода: {e}")),
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
    /// Системное имя для `parec -d` (напр. `alsa_output.pci-...analog-stereo.monitor`).
    pub name: String,
    /// Человекочитаемое описание (напр. «Monitor of Built-in Audio Analog Stereo»).
    pub description: String,
    pub is_monitor: bool,
}

impl PulseSource {
    /// Что показывать в GUI: описание, если есть, иначе имя.
    pub fn label(&self) -> &str {
        if self.description.is_empty() { &self.name } else { &self.description }
    }
}

/// Список источников PulseAudio через `pactl list sources` (с `LC_ALL=C` для стабильных меток).
/// На Pulse/PipeWire это единственный способ увидеть monitor выходов (cpal/ALSA их не отдаёт),
/// плюс тут есть человекочитаемые Description вместо `hw:CARD=…`.
pub fn list_pulse_sources() -> Vec<PulseSource> {
    let res = match std::process::Command::new("pactl")
        .env("LC_ALL", "C")
        .args(["list", "sources"])
        .output()
    {
        Ok(res) => res,
        Err(e) => {
            crate::diag::debug("audio", format!("pactl недоступен: {e}"));
            return Vec::new();
        }
    };
    if !res.status.success() {
        let err = String::from_utf8_lossy(&res.stderr);
        crate::diag::warn("audio", format!("pactl list sources failed: {}", err.trim()));
        return Vec::new();
    }
    parse_pulse_sources(&String::from_utf8_lossy(&res.stdout))
}

/// Разбор вывода `pactl list sources` (`LC_ALL=C`). Вынесено из [`list_pulse_sources`],
/// чтобы формат можно было проверять тестами без запуска PulseAudio.
fn parse_pulse_sources(text: &str) -> Vec<PulseSource> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    let mut description = String::new();
    let mut monitor_of = false;

    let flush = |name: &mut Option<String>, description: &mut String, monitor_of: &mut bool, out: &mut Vec<PulseSource>| {
        if let Some(n) = name.take() {
            let is_monitor = *monitor_of || n.ends_with(".monitor") || is_monitor(&n);
            out.push(PulseSource { name: n, description: std::mem::take(description), is_monitor });
        }
        *monitor_of = false;
    };

    for line in text.lines() {
        if line.starts_with("Source #") {
            flush(&mut name, &mut description, &mut monitor_of, &mut out);
        } else if let Some(v) = line.trim().strip_prefix("Name: ") {
            name = Some(v.trim().to_string());
        } else if let Some(v) = line.trim().strip_prefix("Description: ") {
            description = v.trim().to_string();
        } else if let Some(v) = line.trim().strip_prefix("Monitor of Sink: ") {
            monitor_of = v.trim() != "n/a";
        }
    }
    flush(&mut name, &mut description, &mut monitor_of, &mut out);
    out
}

/// Бэкенд захвата: либо cpal-поток (железо), либо процесс `parec` (Pulse-источник/monitor).
/// Держится живым ради RAII (остановка потока/процесса при drop), напрямую не читается.
#[allow(dead_code)]
enum Backend {
    Cpal(cpal::Stream),
    Pulse(PulseCapture),
}

/// Найти устройство и конфиг, с которым его открывать.
///
/// Для loopback берётся устройство *вывода*: cpal на WASAPI сам поднимает
/// `AUDCLNT_STREAMFLAGS_LOOPBACK`, когда output открывают через `build_input_stream`.
/// Входного конфига у такого устройства нет, поэтому спрашиваем выходной.
fn resolve_cpal_device(
    host: &cpal::Host,
    device_name: Option<&str>,
) -> Result<(cpal::Device, cpal::SupportedStreamConfig)> {
    let Some(name) = device_name else {
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("нет входного устройства по умолчанию"))?;
        let cfg = device.default_input_config()?;
        return Ok((device, cfg));
    };

    if !is_loopback_name(name) {
        let device = host
            .input_devices()?
            .find(|d| dev_name(d) == name)
            .ok_or_else(|| anyhow!("устройство не найдено: {name}"))?;
        let cfg = device.default_input_config()?;
        return Ok((device, cfg));
    }

    if !loopback_supported() {
        return Err(anyhow!(
            "захват системного вывода недоступен на этой ОС — выберите monitor-источник PulseAudio"
        ));
    }
    let target = display_device_name(name);
    let device = if target.is_empty() {
        host.default_output_device()
            .ok_or_else(|| anyhow!("нет устройства вывода по умолчанию"))?
    } else {
        host.output_devices()?
            .find(|d| dev_name(d) == target)
            .ok_or_else(|| anyhow!("устройство вывода не найдено: {target}"))?
    };
    let cfg = device.default_output_config()?;
    crate::diag::info("audio", format!("захват системного вывода (loopback): {}", dev_name(&device)));
    Ok((device, cfg))
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
        channels_pick: &[usize],
        producer: HeapProd<f32>,
    ) -> Result<Self> {
        if let Some(src) = pulse_source {
            return Self::open_pulse(src, producer);
        }
        Self::open_cpal(device_name, channels_pick, producer)
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

    /// Захват cpal-устройства (микрофон/линейный вход, либо системный вывод в режиме loopback).
    /// `channels_pick` — индексы каналов для анализа; пусто = даунмикс всех в моно.
    fn open_cpal(device_name: Option<&str>, channels_pick: &[usize], mut producer: HeapProd<f32>) -> Result<Self> {
        let host = cpal::default_host();
        let (device, supported) = resolve_cpal_device(&host, device_name)?;
        let name = device_name
            .filter(|n| is_loopback_name(n))
            .map(|n| n.to_string())
            .unwrap_or_else(|| dev_name(&device));
        let sample_rate = supported.sample_rate() as f32; // 0.18: SampleRate = u32
        let channels = supported.channels();
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();

        // Xrun (пропуск буфера) — рядовое событие, особенно у loopback на простаивающем
        // выходе; логируем редко, чтобы не топить в нём настоящие отказы устройства.
        let xruns = std::sync::atomic::AtomicU64::new(0);
        let err_fn = move |e: cpal::Error| match e.kind() {
            cpal::ErrorKind::Xrun => {
                let n = xruns.fetch_add(1, Ordering::Relaxed) + 1;
                if n == 1 || n % 100 == 0 {
                    crate::diag::warn("audio", format!("пропуск буфера (xrun), всего {n}"));
                }
            }
            _ => crate::diag::error("audio", format!("ошибка потока: {e}")),
        };
        let ch = channels as usize;
        // `chunks(0)` паникует, а число каналов приходит от драйвера — не доверяем ему.
        if ch == 0 {
            return Err(anyhow!("устройство {name} сообщило 0 входных каналов"));
        }
        // Нулевая частота дискретизации разошлась бы по DSP делением на ноль (NaN в фильтрах).
        if !(sample_rate > 0.0) {
            return Err(anyhow!("устройство {name} сообщило некорректную частоту: {sample_rate}"));
        }

        // Валидные выбранные каналы (в пределах числа каналов устройства); пусто = все.
        let picks: Vec<usize> = channels_pick.iter().copied().filter(|&i| i < ch).collect();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                let picks = picks.clone();
                device.build_input_stream(
                    config,
                    move |data: &[f32], _| {
                        for frame in data.chunks(ch) {
                            let _ = producer.try_push(mono_pick(frame, &picks));
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::I16 => {
                let picks = picks.clone();
                device.build_input_stream(
                    config,
                    move |data: &[i16], _| {
                        for frame in data.chunks(ch) {
                            let m = mono_pick_map(frame, &picks, |s| s as f32 / i16::MAX as f32);
                            let _ = producer.try_push(m);
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::U16 => {
                let picks = picks.clone();
                device.build_input_stream(
                    config,
                    move |data: &[u16], _| {
                        for frame in data.chunks(ch) {
                            let m = mono_pick_map(frame, &picks, |s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0);
                            let _ = producer.try_push(m);
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            other => return Err(anyhow!("неподдерживаемый формат сэмплов: {other:?}")),
        };

        stream.play()?;
        Ok(Self { _backend: Backend::Cpal(stream), sample_rate, channels, device_name: name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACTL_SAMPLE: &str = "\
Source #0
\tState: SUSPENDED
\tName: alsa_output.pci-0000_00_1f.3.analog-stereo.monitor
\tDescription: Monitor of Built-in Audio Analog Stereo
\tMonitor of Sink: alsa_output.pci-0000_00_1f.3.analog-stereo
Source #1
\tState: RUNNING
\tName: alsa_input.pci-0000_00_1f.3.analog-stereo
\tDescription: Built-in Audio Analog Stereo
\tMonitor of Sink: n/a
";

    #[test]
    fn parses_name_description_and_monitor_flag() {
        let got = parse_pulse_sources(PACTL_SAMPLE);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].name, "alsa_output.pci-0000_00_1f.3.analog-stereo.monitor");
        assert_eq!(got[0].description, "Monitor of Built-in Audio Analog Stereo");
        assert!(got[0].is_monitor);
        assert_eq!(got[1].name, "alsa_input.pci-0000_00_1f.3.analog-stereo");
        assert!(!got[1].is_monitor);
    }

    #[test]
    fn parses_empty_and_garbage_input_without_panicking() {
        assert!(parse_pulse_sources("").is_empty());
        assert!(parse_pulse_sources("no colons here\n\n\t\n").is_empty());
        // Заголовок без Name не должен порождать запись.
        assert!(parse_pulse_sources("Source #0\n\tState: IDLE\n").is_empty());
    }

    #[test]
    fn last_source_is_flushed_without_trailing_header() {
        let got = parse_pulse_sources("Source #0\n\tName: solo\n");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "solo");
    }

    #[test]
    fn label_falls_back_to_name_when_description_is_empty() {
        let s = PulseSource { name: "raw".into(), description: String::new(), is_monitor: false };
        assert_eq!(s.label(), "raw");
        let s = PulseSource { name: "raw".into(), description: "Nice".into(), is_monitor: false };
        assert_eq!(s.label(), "Nice");
    }

    #[test]
    fn mono_pick_averages_all_channels_when_no_picks() {
        assert_eq!(mono_pick(&[1.0, 3.0], &[]), 2.0);
    }

    #[test]
    fn mono_pick_ignores_out_of_range_picks() {
        // Индексы каналов приходят из конфига и могут пережить смену устройства.
        assert_eq!(mono_pick(&[1.0, 3.0], &[1, 99]), 3.0);
        assert_eq!(mono_pick(&[1.0, 3.0], &[99]), 0.0);
    }

    #[test]
    fn mono_pick_on_empty_frame_is_zero() {
        assert_eq!(mono_pick(&[], &[]), 0.0);
    }

    #[test]
    fn mono_pick_map_converts_and_averages() {
        let got = mono_pick_map(&[i16::MAX, 0], &[], |s| s as f32 / i16::MAX as f32);
        assert!((got - 0.5).abs() < 1e-6);
    }

    #[test]
    fn loopback_names_round_trip_through_the_prefix() {
        let stored = format!("{LOOPBACK_PREFIX}Динамики (Realtek)");
        assert!(is_loopback_name(&stored));
        assert_eq!(display_device_name(&stored), "Динамики (Realtek)");
        // Пустой остаток = устройство вывода по умолчанию.
        assert!(is_loopback_name(LOOPBACK_PREFIX));
        assert_eq!(display_device_name(LOOPBACK_PREFIX), "");
    }

    #[test]
    fn plain_device_names_are_left_alone() {
        assert!(!is_loopback_name("Mic in at rear panel"));
        assert_eq!(display_device_name("Mic in at rear panel"), "Mic in at rear panel");
        // Устройство, у которого «loopback» просто внутри имени, не префикс.
        assert!(!is_loopback_name("Cable Loopback Input"));
    }

    #[test]
    fn is_monitor_matches_common_names() {
        assert!(is_monitor("alsa_output.stereo.monitor"));
        assert!(is_monitor("Loopback Device"));
        assert!(!is_monitor("Built-in Audio Analog Stereo"));
    }
}
