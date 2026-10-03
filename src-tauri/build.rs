#[cfg(windows)]
extern "system" {
    fn GetShortPathNameW(
        lpszLongPath: *const u16,
        lpszShortPath: *mut u16,
        cchBuffer: u32,
    ) -> u32;
}

#[cfg(windows)]
fn to_short_path(path: &std::path::Path) -> std::path::PathBuf {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let len = unsafe { GetShortPathNameW(wide.as_ptr(), std::ptr::null_mut(), 0) };
    if len > 0 {
        let mut buf: Vec<u16> = vec![0; len as usize];
        let res = unsafe { GetShortPathNameW(wide.as_ptr(), buf.as_mut_ptr(), len) };
        if res > 0 && (res as usize) < buf.len() {
            buf.truncate(res as usize);
            return std::path::PathBuf::from(OsString::from_wide(&buf));
        }
    }
    path.to_path_buf()
}

fn main() {
    #[cfg(windows)]
    {
        // On Windows GNU targets, windres / cc1.exe fails if OUT_DIR or include paths contain spaces.
        // Convert OUT_DIR and current_dir to 8.3 short paths to guarantee 100% space-safe compilation.
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let short_out = to_short_path(std::path::Path::new(&out_dir));
            std::env::set_var("OUT_DIR", &short_out);
        }

        if let Ok(cur_dir) = std::env::current_dir() {
            let short_cur = to_short_path(&cur_dir);
            let _ = std::env::set_current_dir(&short_cur);
        }
    }

    let mut windows = tauri_build::WindowsAttributes::new();
    #[cfg(windows)]
    {
        let icon_path = std::path::Path::new("icons/icon.ico");
        if icon_path.exists() {
            let abs_icon = std::fs::canonicalize(icon_path).unwrap_or_else(|_| icon_path.to_path_buf());
            let short_icon = to_short_path(&abs_icon);
            windows = windows.window_icon_path(short_icon);
        }
    }

    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run build script");
}
