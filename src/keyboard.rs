use rdev::{simulate, EventType, Key};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyboardError {
    #[error("failed to simulate key event: {0}")]
    SimulateError(#[from] rdev::SimulateError),
    #[error("unknown key: {0}")]
    UnknownKey(String),
}

/// Maps key string (e.g., "h", ";") to rdev Key
fn key_str_to_rdev_key(key_str: &str) -> Option<Key> {
    match key_str {
        "a" => Some(Key::KeyA),
        "b" => Some(Key::KeyB),
        "c" => Some(Key::KeyC),
        "d" => Some(Key::KeyD),
        "e" => Some(Key::KeyE),
        "f" => Some(Key::KeyF),
        "g" => Some(Key::KeyG),
        "h" => Some(Key::KeyH),
        "i" => Some(Key::KeyI),
        "j" => Some(Key::KeyJ),
        "k" => Some(Key::KeyK),
        "l" => Some(Key::KeyL),
        "m" => Some(Key::KeyM),
        "n" => Some(Key::KeyN),
        "o" => Some(Key::KeyO),
        "p" => Some(Key::KeyP),
        "q" => Some(Key::KeyQ),
        "r" => Some(Key::KeyR),
        "s" => Some(Key::KeyS),
        "t" => Some(Key::KeyT),
        "u" => Some(Key::KeyU),
        "v" => Some(Key::KeyV),
        "w" => Some(Key::KeyW),
        "x" => Some(Key::KeyX),
        "y" => Some(Key::KeyY),
        "z" => Some(Key::KeyZ),
        "0" => Some(Key::Num0),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),
        ";" => Some(Key::SemiColon),
        "," => Some(Key::Comma),
        "." => Some(Key::Dot),
        "/" => Some(Key::Slash),
        "'" => Some(Key::Quote),
        "[" => Some(Key::LeftBracket),
        "]" => Some(Key::RightBracket),
        "-" => Some(Key::Minus),
        "=" => Some(Key::Equal),
        "`" => Some(Key::BackQuote),
        "\\" => Some(Key::BackSlash),
        _ => None,
    }
}

/// Press a key down.
pub fn key_down(key: &str) -> Result<(), KeyboardError> {
    let rdev_key =
        key_str_to_rdev_key(key).ok_or_else(|| KeyboardError::UnknownKey(key.to_string()))?;
    let event = EventType::KeyPress(rdev_key);
    simulate(&event)?;
    Ok(())
}

/// Release a key.
pub fn key_up(key: &str) -> Result<(), KeyboardError> {
    let rdev_key =
        key_str_to_rdev_key(key).ok_or_else(|| KeyboardError::UnknownKey(key.to_string()))?;
    let event = EventType::KeyRelease(rdev_key);
    simulate(&event)?;
    Ok(())
}

/// Press multiple keys simultaneously (for chords), hold for duration, then release all.
pub fn chord_press(keys: &[&str], duration: Duration) -> Result<(), KeyboardError> {
    // Press all keys down first
    for key in keys {
        key_down(key)?;
    }

    // Hold for duration
    std::thread::sleep(duration);

    // Release all keys in reverse order
    for key in keys.iter().rev() {
        key_up(key)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_str_to_rdev_key_letters() {
        assert_eq!(key_str_to_rdev_key("h"), Some(Key::KeyH));
        assert_eq!(key_str_to_rdev_key("j"), Some(Key::KeyJ));
        assert_eq!(key_str_to_rdev_key("k"), Some(Key::KeyK));
        assert_eq!(key_str_to_rdev_key("l"), Some(Key::KeyL));
        assert_eq!(key_str_to_rdev_key(";"), Some(Key::SemiColon));
        assert_eq!(key_str_to_rdev_key(","), Some(Key::Comma));
        assert_eq!(key_str_to_rdev_key("."), Some(Key::Dot));
        assert_eq!(key_str_to_rdev_key("/"), Some(Key::Slash));
        assert_eq!(key_str_to_rdev_key("y"), Some(Key::KeyY));
        assert_eq!(key_str_to_rdev_key("u"), Some(Key::KeyU));
        assert_eq!(key_str_to_rdev_key("i"), Some(Key::KeyI));
        assert_eq!(key_str_to_rdev_key("o"), Some(Key::KeyO));
        assert_eq!(key_str_to_rdev_key("p"), Some(Key::KeyP));
    }

    #[test]
    fn test_key_str_to_rdev_key_numbers() {
        assert_eq!(key_str_to_rdev_key("0"), Some(Key::Num0));
        assert_eq!(key_str_to_rdev_key("9"), Some(Key::Num9));
    }

    #[test]
    fn test_key_str_to_rdev_key_invalid() {
        assert_eq!(key_str_to_rdev_key("space"), None);
        assert_eq!(key_str_to_rdev_key("enter"), None);
        assert_eq!(key_str_to_rdev_key(""), None);
        assert_eq!(key_str_to_rdev_key("abc"), None);
    }

    #[test]
    fn test_unknown_key() {
        // key_down with unknown key should return error
        let result = key_down("space");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyboardError::UnknownKey(_)));
    }

    #[test]
    fn test_chord_press_empty() {
        // Empty chord should be a no-op
        let result = chord_press(&[], Duration::from_millis(10));
        assert!(result.is_ok());
    }
}
