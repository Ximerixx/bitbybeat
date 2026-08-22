//! OSC-выход (md_plans/06): UDP на 127.0.0.1:7700, каналы `/<name> <float>`.
//! Отправка вынесена в отдельный поток со своим таймером (`osc_rate_hz`), независимым от
//! частоты обсчёта: compute-loop кладёт последний снимок в `shared.osc_out`, здесь мы его пушим.

use crate::config::OscCfg;
use crate::shared::Shared;
use anyhow::Result;
use rosc::{encoder, OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::net::UdpSocket;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct OscSender {
    socket: UdpSocket,
    target: String,
}

impl OscSender {
    pub fn new(cfg: &OscCfg) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        Ok(Self { socket, target: format!("{}:{}", cfg.host, cfg.port) })
    }

    /// Отправить набор (address, value). Одним bundle либо по сообщению — по конфигу.
    pub fn send(&self, channels: &[(String, f32)], bundle: bool) -> Result<()> {
        if bundle {
            let msgs = channels
                .iter()
                .map(|(addr, v)| {
                    OscPacket::Message(OscMessage {
                        addr: format!("/{addr}"),
                        args: vec![OscType::Float(*v)],
                    })
                })
                .collect();
            let packet = OscPacket::Bundle(OscBundle { timetag: OscTime::from((0u32, 1u32)), content: msgs });
            let buf = encoder::encode(&packet)?;
            self.socket.send_to(&buf, &self.target)?;
        } else {
            for (addr, v) in channels {
                let packet = OscPacket::Message(OscMessage {
                    addr: format!("/{addr}"),
                    args: vec![OscType::Float(*v)],
                });
                let buf = encoder::encode(&packet)?;
                self.socket.send_to(&buf, &self.target)?;
            }
        }
        Ok(())
    }
}

/// Поток отправки OSC: тикает на `osc_rate_hz`, читает последний снимок из `shared.osc_out`.
/// Пересоздаёт сокет при смене host/port. Отвязан от частоты обсчёта.
pub fn spawn(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut sender: Option<OscSender> = None;
        let mut last_target = String::new();

        while shared.running.load(Ordering::Relaxed) {
            let t0 = Instant::now();
            let (enabled, bundle, host, port, rate) = {
                let c = shared.config.lock().unwrap();
                (c.osc.enabled, c.osc.bundle, c.osc.host.clone(), c.osc.port, c.osc_rate_hz)
            };
            let rate = rate.clamp(1.0, 480.0);
            let tick = Duration::from_secs_f32(1.0 / rate);

            if enabled {
                let target = format!("{host}:{port}");
                if sender.is_none() || target != last_target {
                    sender = OscSender::new(&OscCfg { enabled, host, port, bundle }).ok();
                    last_target = target;
                }
                if let Some(s) = sender.as_ref() {
                    let snapshot = shared.osc_out.lock().unwrap().clone();
                    if !snapshot.is_empty() {
                        let _ = s.send(&snapshot, bundle);
                    }
                }
            }

            let elapsed = t0.elapsed();
            if elapsed < tick {
                std::thread::sleep(tick - elapsed);
            }
        }
    })
}
