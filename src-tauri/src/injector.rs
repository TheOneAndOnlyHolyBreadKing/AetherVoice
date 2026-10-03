#[cfg(not(windows))]
use enigo::{Enigo, Keyboard, Settings};
#[cfg(not(windows))]
use std::sync::Mutex;

#[cfg(not(windows))]
static ENIGO_INSTANCE: Mutex<Option<Enigo>> = Mutex::new(None);

#[cfg(windows)]
fn paste_via_clipboard(text: &str) -> Result<(), String> {
    use std::ptr;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
    };
    const CF_UNICODETEXT: u32 = 13;

    // Small delay so any physical hotkey release (e.g. AltRight) has settled in the OS
    std::thread::sleep(std::time::Duration::from_millis(40));

    // Normalize newlines to Windows standard CRLF (\r\n) for clipboard compatibility
    let mut normalized_text = String::with_capacity(text.len() + 16);
    let mut prev_char = ' ';
    for c in text.chars() {
        if c == '\n' && prev_char != '\r' {
            normalized_text.push('\r');
        }
        normalized_text.push(c);
        prev_char = c;
    }

    let utf16_units: Vec<u16> = normalized_text.encode_utf16().chain(std::iter::once(0)).collect();
    let size_bytes = utf16_units.len() * std::mem::size_of::<u16>();

    unsafe {
        // Retry OpenClipboard up to 5 times (in case another process briefly holds it)
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }

        if !opened {
            return Err("Failed to open Windows clipboard".to_string());
        }

        EmptyClipboard();

        let h_mem = GlobalAlloc(GMEM_MOVEABLE, size_bytes);
        if h_mem.is_null() {
            CloseClipboard();
            return Err("GlobalAlloc failed for clipboard data".to_string());
        }

        let p_mem = GlobalLock(h_mem) as *mut u16;
        if p_mem.is_null() {
            CloseClipboard();
            return Err("GlobalLock failed for clipboard data".to_string());
        }

        ptr::copy_nonoverlapping(utf16_units.as_ptr(), p_mem, utf16_units.len());
        GlobalUnlock(h_mem);

        if SetClipboardData(CF_UNICODETEXT, h_mem as _).is_null() {
            CloseClipboard();
            return Err("SetClipboardData failed".to_string());
        }

        CloseClipboard();

        // Synthesize Ctrl + V keypress
        let vk_v = 0x56u16; // 'V' virtual key code
        let inputs = [
            // Ctrl down
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // V down
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk_v,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // V up
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk_v,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // Ctrl up
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );

        if sent != inputs.len() as u32 {
            return Err(format!("SendInput sent {}/{} inputs", sent, inputs.len()));
        }
    }

    Ok(())
}

#[cfg(windows)]
fn type_via_keystrokes(text: &str) -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VK_RETURN, VK_TAB,
    };

    let mut inputs = Vec::with_capacity(text.len() * 2);

    for c in text.chars() {
        if c == '\r' {
            continue;
        }

        if c == '\n' {
            // Enter key down & up
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_RETURN,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_RETURN,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        } else if c == '\t' {
            // Tab key down & up
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_TAB,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_TAB,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        } else {
            // Unicode char down & up
            let mut utf16_buf = [0u16; 2];
            let encoded = c.encode_utf16(&mut utf16_buf);
            for &unit in &*encoded {
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: 0,
                            wScan: unit,
                            dwFlags: KEYEVENTF_UNICODE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: 0,
                            wScan: unit,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
            }
        }
    }

    if !inputs.is_empty() {
        unsafe {
            let sent = SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
            if sent == 0 {
                return Err("SendInput failed to send keystrokes".to_string());
            }
        }
    }

    Ok(())
}

/// Injects text directly into the active cursor position.
/// Uses instant Windows Clipboard paste with fallback to direct Unicode SendInput keystrokes.
pub fn inject_text(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    #[cfg(windows)]
    {
        match paste_via_clipboard(text) {
            Ok(_) => {
                println!("[AetherVoice] Injected {} characters via clipboard paste", text.len());
                return Ok(());
            }
            Err(err) => {
                eprintln!("[AetherVoice] Clipboard paste failed ({}), falling back to direct keystrokes", err);
                return type_via_keystrokes(text);
            }
        }
    }

    #[cfg(not(windows))]
    {
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
}
