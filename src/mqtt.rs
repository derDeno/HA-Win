use crate::{
    discovery::{self, Command},
    settings::{machine_name, Settings},
    UiEvent, UiSender,
};
use anyhow::{bail, Result};
use rumqttc::{Client, Connection, Event, LastWill, MqttOptions, Outgoing, Packet, QoS, Transport};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

enum Control {
    Apply(Settings),
    Stop,
}

pub struct Service {
    commands: Sender<Control>,
    thread: Option<JoinHandle<()>>,
}

impl Service {
    pub fn new(settings: Settings, ui: UiSender) -> Self {
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || run(settings, rx, ui));
        Self {
            commands: tx,
            thread: Some(thread),
        }
    }

    pub fn apply(&self, settings: Settings) {
        let _ = self.commands.send(Control::Apply(settings));
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.commands.send(Control::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn connect(settings: &Settings, test: bool) -> Result<(Client, Connection)> {
    settings.validate()?;
    if settings.broker_host.trim().is_empty() {
        bail!("Configure a broker hostname first.");
    }
    let id = settings.device_id();
    let mut client_id = if settings.client_id.trim().is_empty() {
        format!("HaWin-{id}")
    } else {
        settings.client_id.clone()
    };
    if test {
        client_id.push_str(&format!("-test-{}", std::process::id()));
    }
    let mut options =
        MqttOptions::new(client_id, settings.broker_host.trim(), settings.broker_port);
    options
        .set_keep_alive(Duration::from_secs(20))
        .set_clean_session(true)
        .set_max_packet_size(65536, 65536);
    if !settings.username.trim().is_empty() {
        options.set_credentials(&settings.username, &settings.password);
    }
    if settings.use_tls {
        options.set_transport(Transport::tls_with_config(
            rumqttc::TlsConfiguration::Native,
        ));
    }
    if !test {
        options.set_last_will(LastWill::new(
            discovery::status(&id),
            "offline",
            QoS::AtLeastOnce,
            true,
        ));
    }
    let (client, mut connection) = Client::new(options, 32);
    connection
        .eventloop
        .network_options
        .set_connection_timeout(5);
    Ok((client, connection))
}

pub fn test_connection(settings: &Settings) -> Result<()> {
    let (client, mut connection) = connect(settings, true)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match connection.recv_timeout(Duration::from_secs(6)) {
            Ok(Ok(Event::Incoming(Packet::ConnAck(_)))) => {
                client.try_disconnect()?;
                let _ = connection.recv_timeout(Duration::from_secs(1));
                return Ok(());
            }
            Ok(Err(e)) => return Err(e.into()),
            _ => {}
        }
    }
    bail!("Connection timed out.")
}

fn announce(client: &Client, id: &str) -> Result<()> {
    for (topic, payload) in discovery::configs(id, &machine_name()) {
        client.try_publish(topic, QoS::AtLeastOnce, true, payload)?;
    }
    client.try_publish(discovery::status(id), QoS::AtLeastOnce, true, "online")?;
    Ok(())
}

fn close(client: &Client, connection: &mut Connection, id: &str) {
    // Flush offline before DISCONNECT, which otherwise suppresses the last will.
    let _ = client.try_publish(discovery::status(id), QoS::AtMostOnce, true, "offline");
    let _ = client.try_disconnect();
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        match connection.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(Event::Outgoing(Outgoing::Disconnect))) | Ok(Err(_)) => break,
            _ => {}
        }
    }
}

fn run(mut settings: Settings, commands: Receiver<Control>, ui: UiSender) {
    'configure: loop {
        ui.send(UiEvent::Connection(
            false,
            if settings.broker_host.trim().is_empty() {
                "Configure an MQTT broker.".into()
            } else {
                "Connecting…".into()
            },
        ));
        let (client, mut connection) = match connect(&settings, false) {
            Ok(pair) => pair,
            Err(e) => {
                ui.send(UiEvent::Connection(false, e.to_string()));
                match commands.recv() {
                    Ok(Control::Apply(next)) => {
                        settings = next;
                        continue;
                    }
                    _ => return,
                }
            }
        };
        let id = settings.device_id();
        let mut connected = false;
        let mut retry_at = Instant::now();
        loop {
            match commands.try_recv() {
                Ok(Control::Apply(next)) => {
                    if connected {
                        close(&client, &mut connection, &id);
                    }
                    settings = next;
                    continue 'configure;
                }
                Ok(Control::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                    if connected {
                        close(&client, &mut connection, &id);
                    }
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            if Instant::now() < retry_at {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            // A short timeout cancels the TCP/TLS handshake; allow the full network timeout until connected.
            let timeout = if connected {
                Duration::from_millis(200)
            } else {
                Duration::from_secs(6)
            };
            match connection.recv_timeout(timeout) {
                Ok(Ok(Event::Incoming(Packet::ConnAck(_)))) => {
                    let result = (|| -> Result<()> {
                        client.try_subscribe(
                            format!("{}/+/set", discovery::base(&id)),
                            QoS::AtLeastOnce,
                        )?;
                        client.try_subscribe(discovery::notify(&id), QoS::AtLeastOnce)?;
                        client.try_subscribe("homeassistant/status", QoS::AtLeastOnce)?;
                        announce(&client, &id)
                    })();
                    connected = true;
                    ui.send(UiEvent::Connection(true, "Running".into()));
                    if let Err(e) = result {
                        ui.send(UiEvent::Error(format!("Discovery failed: {e}")));
                    }
                }
                Ok(Ok(Event::Incoming(Packet::Publish(message)))) => {
                    if message.topic == "homeassistant/status" && message.payload == "online" {
                        if let Err(e) = announce(&client, &id) {
                            ui.send(UiEvent::Error(e.to_string()));
                        }
                    } else if let Some(command) =
                        discovery::command(&id, &message.topic, &message.payload, message.retain)
                    {
                        match command {
                            Command::Notification { title, message } => {
                                ui.send(UiEvent::Notification(title, message))
                            }
                            Command::Action(action) => {
                                if let Err(e) = crate::platform::device_action(action) {
                                    ui.send(UiEvent::Error(e.to_string()));
                                }
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    connected = false;
                    ui.send(UiEvent::Connection(
                        false,
                        format!("Disconnected: {e}. Retrying in 5 seconds."),
                    ));
                    retry_at = Instant::now() + Duration::from_secs(5);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    fn packet(socket: &mut std::net::TcpStream) -> (u8, Vec<u8>) {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        let header = byte[0];
        let (mut length, mut multiplier) = (0, 1);
        loop {
            socket.read_exact(&mut byte).unwrap();
            length += usize::from(byte[0] & 127) * multiplier;
            if byte[0] & 128 == 0 {
                break;
            }
            multiplier *= 128;
        }
        let mut body = vec![0; length];
        socket.read_exact(&mut body).unwrap();
        (header, body)
    }

    #[test]
    fn mqtt_connect_discovery_notification_and_clean_shutdown() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            assert_eq!(packet(&mut socket).0 >> 4, 1);
            thread::sleep(Duration::from_millis(350));
            socket.write_all(&[0x20, 2, 0, 0]).unwrap();
            let mut discovery_count = 0;
            let mut offline = false;
            loop {
                let (header, body) = packet(&mut socket);
                match header >> 4 {
                    8 => {
                        socket.write_all(&[0x90, 3, body[0], body[1], 1]).unwrap();
                        let topic_len = u16::from_be_bytes([body[2], body[3]]) as usize;
                        if &body[4..4 + topic_len] == b"ha-win/test_pc/notify" {
                            let topic = b"ha-win/test_pc/notify";
                            let mut publish =
                                vec![0x30, (2 + topic.len() + 5) as u8, 0, topic.len() as u8];
                            publish.extend_from_slice(topic);
                            publish.extend_from_slice(b"Hello");
                            socket.write_all(&publish).unwrap();
                        }
                    }
                    3 => {
                        let len = u16::from_be_bytes([body[0], body[1]]) as usize;
                        let topic = std::str::from_utf8(&body[2..2 + len]).unwrap();
                        let qos = (header >> 1) & 3;
                        let offset = 2 + len + if qos > 0 { 2 } else { 0 };
                        if topic.starts_with("homeassistant/") {
                            discovery_count += 1;
                            assert_ne!(header & 1, 0);
                        }
                        if topic.ends_with("/status") && &body[offset..] == b"offline" {
                            offline = true;
                        }
                        if qos == 1 {
                            socket
                                .write_all(&[0x40, 2, body[2 + len], body[3 + len]])
                                .unwrap();
                        }
                    }
                    14 => {
                        assert_eq!(discovery_count, 4);
                        assert!(offline);
                        break;
                    }
                    _ => {}
                }
            }
        });
        let (tx, rx) = mpsc::channel();
        let service = Service::new(
            Settings {
                broker_host: "127.0.0.1".into(),
                broker_port: port,
                namespace: "test_pc".into(),
                ..Default::default()
            },
            UiSender { tx, notice: None },
        );
        loop {
            if let UiEvent::Notification(title, text) =
                rx.recv_timeout(Duration::from_secs(10)).unwrap()
            {
                assert_eq!(title, "Home Assistant");
                assert_eq!(text, "Hello");
                break;
            }
        }
        drop(service);
        server.join().unwrap();
    }

    #[test]
    fn connection_test_handles_slow_brokers_and_authentication_rejection() {
        for return_code in [0, 5] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let server = thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let (header, body) = packet(&mut socket);
                assert_eq!(header >> 4, 1);
                assert_eq!(
                    body[7] & 4,
                    0,
                    "Connection tests must not set an availability will"
                );
                thread::sleep(Duration::from_millis(350));
                socket.write_all(&[0x20, 2, 0, return_code]).unwrap();
                if return_code == 0 {
                    assert_eq!(packet(&mut socket).0 >> 4, 14);
                }
            });
            let result = test_connection(&Settings {
                broker_host: "127.0.0.1".into(),
                broker_port: port,
                ..Default::default()
            });
            assert_eq!(result.is_ok(), return_code == 0);
            server.join().unwrap();
        }
    }

    #[test]
    fn reconnects_after_broker_loss_and_can_disable_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut first, _) = listener.accept().unwrap();
            first
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            assert_eq!(packet(&mut first).0 >> 4, 1);
            first.write_all(&[0x20, 2, 0, 0]).unwrap();
            thread::sleep(Duration::from_millis(100));
            drop(first);
            let (mut second, _) = listener.accept().unwrap();
            second
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            assert_eq!(packet(&mut second).0 >> 4, 1);
            second.write_all(&[0x20, 2, 0, 0]).unwrap();
            second
        });
        let (tx, rx) = mpsc::channel();
        let service = Service::new(
            Settings {
                broker_host: "127.0.0.1".into(),
                broker_port: port,
                ..Default::default()
            },
            UiSender { tx, notice: None },
        );
        let mut connections = 0;
        while connections < 2 {
            if let UiEvent::Connection(true, _) = rx.recv_timeout(Duration::from_secs(12)).unwrap()
            {
                connections += 1;
            }
        }
        let _socket = server.join().unwrap();
        service.apply(Settings::default());
        loop {
            if let UiEvent::Connection(false, message) =
                rx.recv_timeout(Duration::from_secs(5)).unwrap()
            {
                if message.contains("Configure") {
                    break;
                }
            }
        }
        drop(service);
    }
}
