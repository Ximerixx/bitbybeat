//! OSC-выход: один bundle на тик. UDP — send_to без connect (как в alpha / QLC+).

use crate::config::OscCfg;
use crate::diag;
use crate::osc_map::{channels_for_send, is_trigger_address, OscSnapshot, TriggerPulse};
use crate::shared::Shared;
use anyhow::{Context, Result};
use rosc::{encoder, OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

enum Transport {
    Udp { socket: UdpSocket, dest: SocketAddr },
    Tcp(TcpStream),
}

struct OscSender {
    transport: Transport,
}

impl OscSender {
    fn connect(cfg: &OscCfg) -> Result<Self> {
        let addr = resolve(&cfg.host, cfg.port)?;
        match cfg.transport {
            crate::config::OscTransport::Udp => {
                let socket = UdpSocket::bind("0.0.0.0:0").context("OSC UDP bind")?;
                Ok(Self {
                    transport: Transport::Udp { socket, dest: addr },
                })
            }
            crate::config::OscTransport::Tcp => {
                let stream = TcpStream::connect(&addr).context("OSC TCP connect")?;
                if let Err(e) = stream.set_nodelay(true) {
                    // Не фатально: соединение работает, но пакеты могут буферизоваться Nagle.
                    diag::warn("osc", format!("tcp set_nodelay failed: {e}"));
                }
                Ok(Self { transport: Transport::Tcp(stream) })
            }
        }
    }

    fn send_raw(&mut self, buf: &[u8]) -> Result<()> {
        match &mut self.transport {
            Transport::Udp { socket, dest } => {
                socket.send_to(buf, *dest)?;
            }
            Transport::Tcp(s) => {
                let len = (buf.len() as u32).to_be_bytes();
                s.write_all(&len)?;
                s.write_all(buf)?;
                s.flush()?;
            }
        }
        Ok(())
    }

    fn send_bundle(&mut self, packets: Vec<OscPacket>, timetag: OscTime) -> Result<()> {
        if packets.is_empty() {
            return Ok(());
        }
        let packet = OscPacket::Bundle(OscBundle { timetag, content: packets });
        let buf = encoder::encode(&packet)?;
        self.send_raw(&buf)
    }
}

fn resolve(host: &str, port: u16) -> Result<std::net::SocketAddr> {
    let mut addrs = (host, port).to_socket_addrs().context("OSC resolve")?;
    addrs.next().context("OSC: no addresses")
}

/// Сколько compute-кадров импульс ждёт своей доли такта, прежде чем его выбросить.
/// ~2 с при 120 Гц: фаза двигается только от kick, и если он замолчал, ожидание иначе
/// вечное — очередь растёт, а при возврате фазы копившееся уходит одной вспышкой.
const MAX_PENDING_AGE_FRAMES: u64 = 240;

/// Страховка на случай, если кадры почему-то перестали расти.
const MAX_PENDING_TRIGGERS: usize = 1024;

/// Выбросить импульсы, не дождавшиеся своей фазы. Возвращает число выброшенных.
fn prune_stale_triggers(pending: &mut Vec<TriggerPulse>, frame_id: u64) -> usize {
    let before = pending.len();
    pending.retain(|p| frame_id.saturating_sub(p.frame_id) <= MAX_PENDING_AGE_FRAMES);
    if pending.len() > MAX_PENDING_TRIGGERS {
        let excess = pending.len() - MAX_PENDING_TRIGGERS;
        pending.drain(..excess);
    }
    before - pending.len()
}

fn quantize_phase(phase: f32, grid: f32) -> f32 {
    let g = grid.clamp(0.01, 1.0);
    ((phase / g).round() * g).clamp(0.0, 1.0)
}

fn sleep_until(deadline: Instant) {
    let now = Instant::now();
    if deadline > now {
        std::thread::sleep(deadline - now);
    }
}

fn osc_msg(addr: &str, args: Vec<OscType>) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: format!("/{addr}"),
        args,
    })
}

fn osc_float(addr: &str, v: f32) -> OscPacket {
    osc_msg(addr, vec![OscType::Float(v)])
}

/// Собрать один bundle: meta + каналы + квантованные импульсы.
/// `bundleTime` — время compute-кадра (`snapshot.t_mono`); `bundleSendTime` — момент send.
fn build_bundle_packets(
    snapshot: &OscSnapshot,
    pending: &mut Vec<TriggerPulse>,
    osc: &OscCfg,
    bundle_seq: u64,
    send_mono: f64,
) -> Vec<OscPacket> {
    let mut packets = Vec::new();
    let frame_id = snapshot.frame_id;
    let phase_cfg = &osc.phase;

    if osc.bundle_meta {
        packets.push(osc_msg("bundleSeq", vec![OscType::Int(bundle_seq as i32)]));
        packets.push(osc_float("bundleTime", snapshot.t_mono as f32));
        packets.push(osc_msg("bundleFrame", vec![OscType::Int(frame_id as i32)]));
        packets.push(osc_float("bundleSendTime", send_mono as f32));
    }

    let sent = channels_for_send(&snapshot.channels, osc);
    let mut already: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for (addr, val) in &sent {
        packets.push(osc_float(addr, *val));
        already.insert(addr.clone(), *val);
    }

    pending.retain(|p| {
        if !osc.sends(p.address) {
            return false;
        }
        if phase_cfg.quantize_triggers && !phase_cfg.immediate_triggers {
            let q = quantize_phase(p.phase, phase_cfg.phase_grid);
            let cur = quantize_phase(snapshot.beat_phase, phase_cfg.phase_grid);
            if (q - cur).abs() > phase_cfg.phase_grid * 0.51 {
                return true;
            }
        }
        // Не дублировать адрес, если канал этого тика уже несёт 1.
        if already.get(p.address).copied().unwrap_or(0.0) > 0.5 {
            return false;
        }
        if is_trigger_address(p.address) {
            packets.push(osc_float(p.address, 1.0));
        }
        false
    });

    packets
}

fn report_send_err(shared: &Arc<Shared>, e: impl std::fmt::Display) {
    let msg = format!("{e}");
    diag::error("osc", &msg);
    shared.metrics.record_osc_err(msg);
}

fn report_send_ok(shared: &Arc<Shared>, seq: u64) {
    shared.metrics.record_osc_ok(seq);
}

pub fn spawn(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || run(shared))
}

fn run(shared: Arc<Shared>) {
    let mut sender: Option<OscSender> = None;
    let mut last_key = String::new();
    let mut pending_triggers: Vec<TriggerPulse> = Vec::new();
    let mut jitter_ema = 0.0f32;
    let mut stale_dropped = 0usize;
    let bundle_seq = AtomicU64::new(0);

    diag::info("osc", "поток OSC запущен");

    while shared.running.load(Ordering::Acquire) {
        let cfg = shared.config.load();
        let osc = &cfg.osc;
        let rate = cfg.osc_rate_hz.clamp(1.0, 480.0);

        let deadline = if osc.phase.sync_timeline {
            shared.timeline.next_osc_deadline(rate)
        } else {
            Instant::now() + Duration::from_secs_f32(1.0 / rate)
        };

        if osc.enabled {
            let key = format!("{:?}:{}:{}", osc.transport, osc.host, osc.port);
            if sender.is_none() || key != last_key {
                match OscSender::connect(osc) {
                    Ok(s) => {
                        diag::info("osc", format!("подключено: {key}"));
                        diag::debug("osc", format!("transport open {key}"));
                        sender = Some(s);
                        last_key = key;
                    }
                    Err(e) => {
                        report_send_err(&shared, e);
                        sender = None;
                        sleep_until(deadline);
                        continue;
                    }
                }
            }

            if let Some(s) = sender.as_mut() {
                let snapshot = shared.osc_out.latest();
                let send_mono = shared.timeline.mono_secs();
                let send_latency_ms =
                    ((send_mono - snapshot.t_mono).max(0.0) * 1000.0) as f32;
                shared.metrics.set_osc_send_latency(send_latency_ms);

                pending_triggers.extend(shared.trigger_queue.drain());
                if osc.phase.immediate_triggers {
                    pending_triggers.extend(snapshot.pulses.iter().cloned());
                }
                let dropped = prune_stale_triggers(&mut pending_triggers, snapshot.frame_id);
                if dropped > 0 {
                    stale_dropped += dropped;
                    // Одиночные потери — норма при смене темпа; поток означает, что kick
                    // не детектится и фаза стоит.
                    if stale_dropped == 1 || stale_dropped % 100 == 0 {
                        diag::warn(
                            "osc",
                            format!("импульсов не дождалось своей фазы: {stale_dropped}"),
                        );
                    }
                }

                let seq = bundle_seq.fetch_add(1, Ordering::Relaxed) + 1;
                // OSC immediate: QLC+ (и alpha) ждут (0, 1), не wall-clock NTP.
                let timetag = OscTime::from((0u32, 1u32));
                let packets = build_bundle_packets(
                    &snapshot,
                    &mut pending_triggers,
                    osc,
                    seq,
                    send_mono,
                );

                if !packets.is_empty() {
                    if osc.bundle {
                        match s.send_bundle(packets, timetag) {
                            Ok(()) => report_send_ok(&shared, seq),
                            Err(e) => report_send_err(&shared, e),
                        }
                    } else {
                        let mut ok = true;
                        for pkt in &packets {
                            match encoder::encode(pkt) {
                                Ok(buf) => {
                                    if let Err(e) = s.send_raw(&buf) {
                                        report_send_err(&shared, e);
                                        ok = false;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    report_send_err(&shared, e);
                                    ok = false;
                                    break;
                                }
                            }
                        }
                        if ok {
                            report_send_ok(&shared, seq);
                        }
                    }
                }
            }
        } else {
            sender = None;
            pending_triggers.clear();
            let _ = shared.trigger_queue.drain();
        }

        sleep_until(deadline);

        let after = Instant::now();
        let lateness_ms = after.saturating_duration_since(deadline).as_secs_f32() * 1000.0;
        jitter_ema = jitter_ema * 0.9 + lateness_ms * 0.1;
        shared.metrics.set_osc_jitter(jitter_ema);
    }

    diag::info("osc", "поток OSC остановлен");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::osc_map::{OscChannel, OscChannelKind};

    fn pulse(address: &'static str, phase: f32, frame_id: u64) -> TriggerPulse {
        TriggerPulse { address, phase, frame_id }
    }

    #[test]
    fn fresh_triggers_are_kept() {
        let mut pending = vec![pulse("kick", 0.0, 100), pulse("snare", 0.25, 120)];
        assert_eq!(prune_stale_triggers(&mut pending, 130), 0);
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn triggers_that_never_met_their_phase_are_dropped() {
        // Фаза стоит на месте (kick не детектится) — импульс ждал бы вечно.
        let mut pending = vec![pulse("snare", 0.75, 10), pulse("snare", 0.75, 900)];
        assert_eq!(prune_stale_triggers(&mut pending, 1000), 1);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].frame_id, 900);
    }

    #[test]
    fn queue_is_capped_even_if_frame_ids_do_not_advance() {
        let mut pending: Vec<_> = (0..MAX_PENDING_TRIGGERS + 50)
            .map(|i| pulse("snare", 0.5, i as u64))
            .collect();
        let dropped = prune_stale_triggers(&mut pending, 0);
        assert_eq!(dropped, 50);
        assert_eq!(pending.len(), MAX_PENDING_TRIGGERS);
        // Выбрасываем самые старые, свежие остаются.
        assert_eq!(pending[0].frame_id, 50);
    }

    #[test]
    fn pruning_an_empty_queue_is_a_no_op() {
        let mut pending: Vec<TriggerPulse> = Vec::new();
        assert_eq!(prune_stale_triggers(&mut pending, 5000), 0);
    }

    #[test]
    fn quantize_snaps_to_the_grid_and_stays_in_range() {
        assert_eq!(quantize_phase(0.26, 0.25), 0.25);
        assert_eq!(quantize_phase(0.4, 0.25), 0.5);
        assert_eq!(quantize_phase(1.4, 0.25), 1.0);
        assert_eq!(quantize_phase(-0.3, 0.25), 0.0);
        // Нулевой шаг сетки не должен давать деление на ноль.
        assert!(quantize_phase(0.5, 0.0).is_finite());
    }

    fn snapshot_with(beat_phase: f32, frame_id: u64) -> OscSnapshot {
        OscSnapshot {
            frame_id,
            t_mono: 0.0,
            beat_phase,
            channels: vec![OscChannel {
                address: "low".into(),
                value: 0.5,
                kind: OscChannelKind::Continuous,
            }],
            pulses: Vec::new(),
        }
    }

    #[test]
    fn pulse_waits_until_its_slice_of_the_bar_comes_round() {
        let cfg = Config::default();
        let snap = snapshot_with(0.0, 10);
        let mut pending = vec![pulse("kick", 0.75, 10)];

        let packets = build_bundle_packets(&snap, &mut pending, &cfg.osc, 1, 0.0);
        // Фаза ещё не подошла: импульс остался в очереди и не отправлен.
        assert_eq!(pending.len(), 1);
        assert!(!packets.iter().any(|p| matches!(p, OscPacket::Message(m) if m.addr == "/kick")));

        let snap = snapshot_with(0.75, 11);
        let packets = build_bundle_packets(&snap, &mut pending, &cfg.osc, 2, 0.0);
        assert!(pending.is_empty());
        assert!(packets.iter().any(|p| matches!(p, OscPacket::Message(m) if m.addr == "/kick")));
    }

    #[test]
    fn disabled_address_drops_its_pending_pulses() {
        let mut cfg = Config::default();
        cfg.osc.set_sends("kick", false);
        let snap = snapshot_with(0.0, 10);
        let mut pending = vec![pulse("kick", 0.0, 10)];

        let packets = build_bundle_packets(&snap, &mut pending, &cfg.osc, 1, 0.0);
        assert!(pending.is_empty());
        assert!(!packets.iter().any(|p| matches!(p, OscPacket::Message(m) if m.addr == "/kick")));
    }
}
