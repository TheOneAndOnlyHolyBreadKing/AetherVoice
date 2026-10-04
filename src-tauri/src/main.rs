#![windows_subsystem = "windows"]

#[cfg(windows)]
fn check_single_instance() -> bool {
    use std::ptr::null;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_SYSTEMMODAL};

    // Global mutex name unique to AetherVoice
    let mutex_name: Vec<u16> = "Global\\AetherVoice_SingleInstance_Mutex\0"
        .encode_utf16()
        .collect();

    unsafe {
        let handle = CreateMutexW(null(), 1, mutex_name.as_ptr());
        if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            let title: Vec<u16> = "AetherVoice\0".encode_utf16().collect();
            let message: Vec<u16> = "AetherVoice is already running in your system tray or on screen.\n\nPlease check your taskbar tray icons or use your hotkey (Alt / F8) to interact with it.\0"
                .encode_utf16()
                .collect();
            MessageBoxW(
                core::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONINFORMATION | MB_SYSTEMMODAL,
            );
            return false;
        }
        // Retain the mutex handle so Windows keeps it open until this process exits
        static mut MUTEX_HANDLE: windows_sys::Win32::Foundation::HANDLE = core::ptr::null_mut();
        MUTEX_HANDLE = handle;
    }
    true
}

#[cfg(not(windows))]
fn check_single_instance() -> bool {
    true
}

fn main() {
    if !check_single_instance() {
        return;
    }
    aethervoice_lib::run()
}

