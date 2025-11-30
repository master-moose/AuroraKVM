use crate::event::{Button, Key, KvmEvent};
use crate::net::KvmStream;
use anyhow::Result;
use rdev::{EventType, simulate};
use tokio::net::TcpStream;

pub async fn run(host: String) -> Result<()> {
    println!("Connecting to {}", host);
    let stream = TcpStream::connect(host).await?;
    let kvm_stream = KvmStream::new(stream);
    let (mut reader, _writer) = kvm_stream.split();

    // Receive loop
    loop {
        match reader.receive().await {
            Ok(event) => {
                inject_event(event);
            }
            Err(e) => {
                println!("Connection lost: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn inject_event(event: KvmEvent) {
    let event_type = match event {
        KvmEvent::MouseMove { x, y } => Some(EventType::MouseMove { x, y }),
        KvmEvent::MouseButton { button, pressed } => {
            let btn = map_button(button);
            if pressed {
                Some(EventType::ButtonPress(btn))
            } else {
                Some(EventType::ButtonRelease(btn))
            }
        }
        KvmEvent::Key { code, pressed } => {
            let key = map_key(code);
            if pressed {
                Some(EventType::KeyPress(key))
            } else {
                Some(EventType::KeyRelease(key))
            }
        }
        KvmEvent::Wheel { delta_x, delta_y } => Some(EventType::Wheel { delta_x, delta_y }),
    };

    if let Some(et) = event_type {
        if let Err(e) = simulate(&et) {
            println!("Failed to simulate event: {:?}", e);
        }
    }
}

fn map_button(btn: Button) -> rdev::Button {
    match btn {
        Button::Left => rdev::Button::Left,
        Button::Right => rdev::Button::Right,
        Button::Middle => rdev::Button::Middle,
        _ => rdev::Button::Unknown(0),
    }
}

fn map_key(key: Key) -> rdev::Key {
    match key {
        Key::Alt => rdev::Key::Alt,
        Key::AltGr => rdev::Key::AltGr,
        Key::Backspace => rdev::Key::Backspace,
        Key::CapsLock => rdev::Key::CapsLock,
        Key::ControlLeft => rdev::Key::ControlLeft,
        Key::ControlRight => rdev::Key::ControlRight,
        Key::Delete => rdev::Key::Delete,
        Key::DownArrow => rdev::Key::DownArrow,
        Key::End => rdev::Key::End,
        Key::Escape => rdev::Key::Escape,
        Key::F1 => rdev::Key::F1,
        Key::F2 => rdev::Key::F2,
        Key::F3 => rdev::Key::F3,
        Key::F4 => rdev::Key::F4,
        Key::F5 => rdev::Key::F5,
        Key::F6 => rdev::Key::F6,
        Key::F7 => rdev::Key::F7,
        Key::F8 => rdev::Key::F8,
        Key::F9 => rdev::Key::F9,
        Key::F10 => rdev::Key::F10,
        Key::F11 => rdev::Key::F11,
        Key::F12 => rdev::Key::F12,
        Key::Home => rdev::Key::Home,
        Key::LeftArrow => rdev::Key::LeftArrow,
        Key::MetaLeft => rdev::Key::MetaLeft,
        Key::MetaRight => rdev::Key::MetaRight,
        Key::PageDown => rdev::Key::PageDown,
        Key::PageUp => rdev::Key::PageUp,
        Key::Return => rdev::Key::Return,
        Key::RightArrow => rdev::Key::RightArrow,
        Key::ShiftLeft => rdev::Key::ShiftLeft,
        Key::ShiftRight => rdev::Key::ShiftRight,
        Key::Space => rdev::Key::Space,
        Key::Tab => rdev::Key::Tab,
        Key::UpArrow => rdev::Key::UpArrow,
        Key::PrintScreen => rdev::Key::PrintScreen,
        Key::ScrollLock => rdev::Key::ScrollLock,
        Key::Pause => rdev::Key::Pause,
        Key::NumLock => rdev::Key::NumLock,
        Key::BackQuote => rdev::Key::BackQuote,
        Key::Num1 => rdev::Key::Num1,
        Key::Num2 => rdev::Key::Num2,
        Key::Num3 => rdev::Key::Num3,
        Key::Num4 => rdev::Key::Num4,
        Key::Num5 => rdev::Key::Num5,
        Key::Num6 => rdev::Key::Num6,
        Key::Num7 => rdev::Key::Num7,
        Key::Num8 => rdev::Key::Num8,
        Key::Num9 => rdev::Key::Num9,
        Key::Num0 => rdev::Key::Num0,
        Key::Minus => rdev::Key::Minus,
        Key::Equal => rdev::Key::Equal,
        Key::KeyQ => rdev::Key::KeyQ,
        Key::KeyW => rdev::Key::KeyW,
        Key::KeyE => rdev::Key::KeyE,
        Key::KeyR => rdev::Key::KeyR,
        Key::KeyT => rdev::Key::KeyT,
        Key::KeyY => rdev::Key::KeyY,
        Key::KeyU => rdev::Key::KeyU,
        Key::KeyI => rdev::Key::KeyI,
        Key::KeyO => rdev::Key::KeyO,
        Key::KeyP => rdev::Key::KeyP,
        Key::LeftBracket => rdev::Key::LeftBracket,
        Key::RightBracket => rdev::Key::RightBracket,
        Key::KeyA => rdev::Key::KeyA,
        Key::KeyS => rdev::Key::KeyS,
        Key::KeyD => rdev::Key::KeyD,
        Key::KeyF => rdev::Key::KeyF,
        Key::KeyG => rdev::Key::KeyG,
        Key::KeyH => rdev::Key::KeyH,
        Key::KeyJ => rdev::Key::KeyJ,
        Key::KeyK => rdev::Key::KeyK,
        Key::KeyL => rdev::Key::KeyL,
        Key::SemiColon => rdev::Key::SemiColon,
        Key::Quote => rdev::Key::Quote,
        Key::BackSlash => rdev::Key::BackSlash,
        Key::IntlBackslash => rdev::Key::IntlBackslash,
        Key::KeyZ => rdev::Key::KeyZ,
        Key::KeyX => rdev::Key::KeyX,
        Key::KeyC => rdev::Key::KeyC,
        Key::KeyV => rdev::Key::KeyV,
        Key::KeyB => rdev::Key::KeyB,
        Key::KeyN => rdev::Key::KeyN,
        Key::KeyM => rdev::Key::KeyM,
        Key::Comma => rdev::Key::Comma,
        Key::Dot => rdev::Key::Dot,
        Key::Slash => rdev::Key::Slash,
        Key::Insert => rdev::Key::Insert,
        Key::KpReturn => rdev::Key::KpReturn,
        Key::KpMinus => rdev::Key::KpMinus,
        Key::KpPlus => rdev::Key::KpPlus,
        Key::KpMultiply => rdev::Key::KpMultiply,
        Key::KpDivide => rdev::Key::KpDivide,
        Key::Kp0 => rdev::Key::Kp0,
        Key::Kp1 => rdev::Key::Kp1,
        Key::Kp2 => rdev::Key::Kp2,
        Key::Kp3 => rdev::Key::Kp3,
        Key::Kp4 => rdev::Key::Kp4,
        Key::Kp5 => rdev::Key::Kp5,
        Key::Kp6 => rdev::Key::Kp6,
        Key::Kp7 => rdev::Key::Kp7,
        Key::Kp8 => rdev::Key::Kp8,
        Key::Kp9 => rdev::Key::Kp9,
        Key::KpDelete => rdev::Key::KpDelete,
        Key::Function => rdev::Key::Function,
        _ => rdev::Key::Unknown(0),
    }
}
