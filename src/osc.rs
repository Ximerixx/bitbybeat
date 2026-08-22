//! OSC-выход (md_plans/06): UDP на 127.0.0.1:7700, каналы `/<name> <float>`.

use crate::config::OscCfg;
use anyhow::Result;
use rosc::{encoder, OscBundle, OscMessage, OscPacket, OscTime, OscType};
use std::net::UdpSocket;

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
