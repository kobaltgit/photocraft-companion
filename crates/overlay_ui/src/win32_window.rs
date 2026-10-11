#[cfg(windows)]
#[link(name = "user32")]
extern "system" {
    fn GetWindowRect(hWnd: *mut std::ffi::c_void, lpRect: *mut [i32; 4]) -> i32;
    fn IsWindowVisible(hWnd: *mut std::ffi::c_void) -> i32;
    fn EnumWindows(
        lpEnumFunc: unsafe extern "system" fn(*mut std::ffi::c_void, isize) -> i32,
        lParam: isize,
    ) -> i32;
    fn GetWindowTextW(hWnd: *mut std::ffi::c_void, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn GetWindowThreadProcessId(hWnd: *mut std::ffi::c_void, lpdwProcessId: *mut u32) -> u32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_callback(
    hwnd: *mut std::ffi::c_void,
    lparam: isize,
) -> i32 {
    let result_ptr = lparam as *mut Option<[i32; 4]>;
    if IsWindowVisible(hwnd) != 0 {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let my_pid = GetCurrentProcessId();

        // Never match our own process windows (toolbar, manager, etc.)
        if pid != my_pid {
            let mut rect = [0i32; 4]; // left, top, right, bottom
            if GetWindowRect(hwnd, &mut rect) != 0 {
                let w = rect[2] - rect[0];
                let h = rect[3] - rect[1];

                // Must be an active visible window with non-minimized desktop coordinates
                if rect[0] > -1000 && rect[1] > -1000 && w >= 600 && h >= 400 {
                    let mut buf = [0u16; 512];
                    let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 512);
                    let title = if len > 0 {
                        String::from_utf16_lossy(&buf[..len as usize])
                    } else {
                        String::new()
                    };

                    let is_excluded = title.contains("Visual Studio")
                        || title.contains("Code")
                        || title.contains("Terminal")
                        || title.contains("cmd.exe")
                        || title.contains("powershell");

                    if !is_excluded {
                        *result_ptr = Some(rect);
                        return 0; // Found target main editor window
                    }
                }
            }
        }
    }
    1 // continue enumeration
}

/// Finds the RECT of the main PhotoCraft editor window via Win32 API
pub fn find_photocraft_window_rect() -> Option<[i32; 4]> {
    #[cfg(windows)]
    unsafe {
        let mut found: Option<[i32; 4]> = None;
        let lparam = &mut found as *mut Option<[i32; 4]> as isize;
        EnumWindows(enum_windows_callback, lparam);
        found
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Calculates default spawn position for the toolstrip according to spec:
/// Y: top + ~75px (under options bar and tabs)
/// X: right - panel_width - 320px (left of right dock palettes with margin)
/// Always provides a guaranteed visible on-screen fallback if editor window is minimized or not found.
pub fn calculate_toolstrip_spawn_pos(panel_width: f32) -> Option<[f32; 2]> {
    if let Some(rect) = find_photocraft_window_rect() {
        let left = rect[0] as f32;
        let top = rect[1] as f32;
        let right = rect[2] as f32;

        let spawn_y = (top + 75.0).max(50.0);
        let spawn_x = (right - panel_width - 320.0).max(left + 20.0).max(50.0);
        return Some([spawn_x, spawn_y]);
    }
    // Guaranteed visible position on primary display
    Some([280.0, 80.0])
}
