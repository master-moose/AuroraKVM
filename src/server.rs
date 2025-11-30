use crate::config::Config;
use crate::event::{Button, Key, KvmEvent};
use crate::net::{KvmStream, KvmWriter};
use crate::topology::{Focus, Topology};
use anyhow::Result;
use dirs;
use rdev::{EventType, listen};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

pub async fn run(port: u16) -> Result<()> {
    // Load config from file
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("aurora_kvm")
        .join("config.json");

    let config = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                println!("Warning: Failed to parse config, using default");
                Config::default()
            })
    } else {
        println!(
            "Warning: No config file found at {:?}, using default",
            config_path
        );
        Config::default()
    };

    let topology = Arc::new(Mutex::new(Topology::new(config)));

    // Channel for broadcasting events to clients
    let (tx, _rx) = tokio::sync::broadcast::channel::<KvmEvent>(100);

    // Start Input Capture Thread
    let topology_clone = topology.clone();
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            let mut topo = topology_clone.lock().unwrap();

            // Check for edge switching if local
            if let EventType::MouseMove { x, y } = event.event_type {
                if *topo.get_focus() == Focus::Local {
                    if let Some(new_focus) = topo.check_edge(x, y) {
                        println!("Switching focus to {:?}", new_focus);
                        topo.set_focus(new_focus);
                    }
                }
            }

            match topo.get_focus() {
                Focus::Local => {} // Pass through (listen doesn't block)
                Focus::Client(_) => {
                    // Forward to clients
                    if let Some(kvm_event) = map_event(event.event_type) {
                        let _ = tx_clone.send(kvm_event);
                    }
                }
            }
        }) {
            println!("Error: {:?}", error);
        }
    });

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Server listening on port {}", port);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let kvm_stream = KvmStream::new(stream);
            let (_reader, mut writer) = kvm_stream.split();

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = writer.send(&event).await {
                            println!("Failed to send to client {}: {}", addr, e);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Broadcast error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

fn map_event(event_type: EventType) -> Option<KvmEvent> {
    match event_type {
        EventType::MouseMove { x, y } => Some(KvmEvent::MouseMove { x, y }),
        EventType::ButtonPress(btn) => Some(KvmEvent::MouseButton {
            button: map_button(btn),
            pressed: true,
        }),
        EventType::ButtonRelease(btn) => Some(KvmEvent::MouseButton {
            button: map_button(btn),
            pressed: false,
        }),
        EventType::KeyPress(key) => Some(KvmEvent::Key {
            code: map_key(key),
            pressed: true,
        }),
        EventType::KeyRelease(key) => Some(KvmEvent::Key {
            code: map_key(key),
            pressed: false,
        }),
        EventType::Wheel { delta_x, delta_y } => Some(KvmEvent::Wheel { delta_x, delta_y }),
    }
}

fn map_button(btn: rdev::Button) -> Button {
    match btn {
        rdev::Button::Left => Button::Left,
        rdev::Button::Right => Button::Right,
        rdev::Button::Middle => Button::Middle,
        _ => Button::Unknown(0),
    }
}

fn map_key(key: rdev::Key) -> Key {
    match key {
        rdev::Key::Alt => Key::Alt,
        rdev::Key::AltGr => Key::AltGr,
        rdev::Key::Backspace => Key::Backspace,
        rdev::Key::CapsLock => Key::CapsLock,
        rdev::Key::ControlLeft => Key::ControlLeft,
        rdev::Key::ControlRight => Key::ControlRight,
        rdev::Key::Delete => Key::Delete,
        rdev::Key::DownArrow => Key::DownArrow,
        rdev::Key::End => Key::End,
        rdev::Key::Escape => Key::Escape,
        rdev::Key::F1 => Key::F1,
        rdev::Key::F2 => Key::F2,
        rdev::Key::F3 => Key::F3,
        rdev::Key::F4 => Key::F4,
        rdev::Key::F5 => Key::F5,
        rdev::Key::F6 => Key::F6,
        rdev::Key::F7 => Key::F7,
        rdev::Key::F8 => Key::F8,
        rdev::Key::F9 => Key::F9,
        rdev::Key::F10 => Key::F10,
        rdev::Key::F11 => Key::F11,
        rdev::Key::F12 => Key::F12,
        rdev::Key::Home => Key::Home,
        rdev::Key::LeftArrow => Key::LeftArrow,
        rdev::Key::MetaLeft => Key::MetaLeft,
        rdev::Key::MetaRight => Key::MetaRight,
        rdev::Key::PageDown => Key::PageDown,
        rdev::Key::PageUp => Key::PageUp,
        rdev::Key::Return => Key::Return,
        rdev::Key::RightArrow => Key::RightArrow,
        rdev::Key::ShiftLeft => Key::ShiftLeft,
        rdev::Key::ShiftRight => Key::ShiftRight,
        rdev::Key::Space => Key::Space,
        rdev::Key::Tab => Key::Tab,
        rdev::Key::UpArrow => Key::UpArrow,
        rdev::Key::PrintScreen => Key::PrintScreen,
        rdev::Key::ScrollLock => Key::ScrollLock,
        rdev::Key::Pause => Key::Pause,
        rdev::Key::NumLock => Key::NumLock,
        rdev::Key::BackQuote => Key::BackQuote,
        rdev::Key::Num1 => Key::Num1,
        rdev::Key::Num2 => Key::Num2,
        rdev::Key::Num3 => Key::Num3,
        rdev::Key::Num4 => Key::Num4,
        rdev::Key::Num5 => Key::Num5,
        rdev::Key::Num6 => Key::Num6,
        rdev::Key::Num7 => Key::Num7,
        rdev::Key::Num8 => Key::Num8,
        rdev::Key::Num9 => Key::Num9,
        rdev::Key::Num0 => Key::Num0,
        rdev::Key::Minus => Key::Minus,
        rdev::Key::Equal => Key::Equal,
        rdev::Key::KeyQ => Key::KeyQ,
        rdev::Key::KeyW => Key::KeyW,
        rdev::Key::KeyE => Key::KeyE,
        rdev::Key::KeyR => Key::KeyR,
        rdev::Key::KeyT => Key::KeyT,
        rdev::Key::KeyY => Key::KeyY,
        rdev::Key::KeyU => Key::KeyU,
        rdev::Key::KeyI => Key::KeyI,
        rdev::Key::KeyO => Key::KeyO,
        rdev::Key::KeyP => Key::KeyP,
        rdev::Key::LeftBracket => Key::LeftBracket,
        rdev::Key::RightBracket => Key::RightBracket,
        rdev::Key::KeyA => Key::KeyA,
        rdev::Key::KeyS => Key::KeyS,
        rdev::Key::KeyD => Key::KeyD,
        rdev::Key::KeyF => Key::KeyF,
        rdev::Key::KeyG => Key::KeyG,
        rdev::Key::KeyH => Key::KeyH,
        rdev::Key::KeyJ => Key::KeyJ,
        rdev::Key::KeyK => Key::KeyK,
        rdev::Key::KeyL => Key::KeyL,
        rdev::Key::SemiColon => Key::SemiColon,
        rdev::Key::Quote => Key::Quote,
        rdev::Key::BackSlash => Key::BackSlash,
        rdev::Key::IntlBackslash => Key::IntlBackslash,
        rdev::Key::KeyZ => Key::KeyZ,
        rdev::Key::KeyX => Key::KeyX,
        rdev::Key::KeyC => Key::KeyC,
        rdev::Key::KeyV => Key::KeyV,
        rdev::Key::KeyB => Key::KeyB,
        rdev::Key::KeyN => Key::KeyN,
        rdev::Key::KeyM => Key::KeyM,
        rdev::Key::Comma => Key::Comma,
        rdev::Key::Dot => Key::Dot,
        rdev::Key::Slash => Key::Slash,
        rdev::Key::Insert => Key::Insert,
        rdev::Key::KpReturn => Key::KpReturn,
        rdev::Key::KpMinus => Key::KpMinus,
        rdev::Key::KpPlus => Key::KpPlus,
        rdev::Key::KpMultiply => Key::KpMultiply,
        rdev::Key::KpDivide => Key::KpDivide,
        rdev::Key::Kp0 => Key::Kp0,
        rdev::Key::Kp1 => Key::Kp1,
        rdev::Key::Kp2 => Key::Kp2,
        rdev::Key::Kp3 => Key::Kp3,
        rdev::Key::Kp4 => Key::Kp4,
        rdev::Key::Kp5 => Key::Kp5,
        rdev::Key::Kp6 => Key::Kp6,
        rdev::Key::Kp7 => Key::Kp7,
        rdev::Key::Kp8 => Key::Kp8,
        rdev::Key::Kp9 => Key::Kp9,
        rdev::Key::KpDelete => Key::KpDelete,
        rdev::Key::Function => Key::Function,
        _ => Key::Unknown(0),
    }
}
