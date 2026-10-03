use enigo::{Enigo, Keyboard, Settings};
use std::sync::Mutex;

static ENIGO_INSTANCE: Mutex<Option<Enigo>> = Mutex::new(None);

/// Injects text directly into the active cursor position using OS keyboard emulation (SendInput on Windows).
pub fn inject_text(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    let mut lock = ENIGO_INSTANCE
        .lock()
        .map_err(|e| format!("Failed to acquire Enigo lock: {}", e))?;

    if lock.is_none() {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to initialize Enigo input synthesis: {}", e))?;
        *lock = Some(enigo);
    }

    if let Some(enigo) = lock.as_mut() {
        enigo
            .text(text)
            .map_err(|e| format!("Failed to synthesize keystrokes: {}", e))?;
    }

    Ok(())
}
