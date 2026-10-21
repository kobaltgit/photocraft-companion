#[cfg(windows)]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
#[cfg(windows)]
const SM_CXSCREEN: i32 = 0;
#[cfg(windows)]
const SM_CYSCREEN: i32 = 1;
#[cfg(windows)]
const MONITOR_DEFAULTTONULL: u32 = 0;

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
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn GetDpiForSystem() -> u32;
    fn MonitorFromRect(lprc: *const [i32; 4], dwFlags: u32) -> *mut std::ffi::c_void;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn OpenProcess(
        dwDesiredAccess: u32,
        bInheritHandle: i32,
        dwProcessId: u32,
    ) -> *mut std::ffi::c_void;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn QueryFullProcessImageNameW(
        hProcess: *mut std::ffi::c_void,
        dwFlags: u32,
        lpExeName: *mut u16,
        lpdwSize: *mut u32,
    ) -> i32;
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_callback(hwnd: *mut std::ffi::c_void, lparam: isize) -> i32 {
    let result_ptr = lparam as *mut Option<[i32; 4]>;
    if IsWindowVisible(hwnd) != 0 {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let my_pid = GetCurrentProcessId();

        // Never match our own process windows (toolbar, manager, etc.)
        if pid != my_pid && pid != 0 {
            let h_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !h_process.is_null() {
                let mut exe_buf = [0u16; 1024];
                let mut size = exe_buf.len() as u32;
                let query_success =
                    QueryFullProcessImageNameW(h_process, 0, exe_buf.as_mut_ptr(), &mut size);
                CloseHandle(h_process); // Essential: Prevent Win32 handle leaks

                if query_success != 0 && size > 0 {
                    let exe_path =
                        String::from_utf16_lossy(&exe_buf[..size as usize]).to_lowercase();
                    let is_photocraft_exe =
                        exe_path.ends_with("photocraft.exe") && !exe_path.contains("companion");

                    if is_photocraft_exe {
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

                                let has_target_title = title.contains("PhotoCraft")
                                    && !title.contains("Toolbar")
                                    && !title.contains("Plugins Manager")
                                    && !title.contains("Companion");

                                if has_target_title {
                                    *result_ptr = Some(rect);
                                    return 0; // Found target main editor window
                                }
                            }
                        }
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

/// Calculates primary display center position with safe DPI scaling
pub fn calculate_primary_screen_center(width: f32, height: f32) -> [f32; 2] {
    #[cfg(windows)]
    unsafe {
        let screen_w = GetSystemMetrics(SM_CXSCREEN) as f32;
        let screen_h = GetSystemMetrics(SM_CYSCREEN) as f32;
        let dpi = GetDpiForSystem();
        let dpi_scale = if dpi == 0 {
            1.0
        } else {
            (dpi as f32 / 96.0).clamp(0.5, 4.0)
        };
        let logical_w = if screen_w > 0.0 {
            screen_w / dpi_scale
        } else {
            1920.0
        };
        let logical_h = if screen_h > 0.0 {
            screen_h / dpi_scale
        } else {
            1080.0
        };
        let x = ((logical_w - width) / 2.0).max(20.0);
        let y = ((logical_h - height) / 2.0).max(20.0);
        [x, y]
    }
    #[cfg(not(windows))]
    {
        let _ = (width, height);
        [400.0, 250.0]
    }
}

/// Verifies whether the specified window rectangle is positioned on any active monitor
pub fn is_position_visible_on_screen(pos: [f32; 2], size: [f32; 2]) -> bool {
    if !pos[0].is_finite() || !pos[1].is_finite() || !size[0].is_finite() || !size[1].is_finite() {
        return false;
    }
    #[cfg(windows)]
    unsafe {
        let dpi = GetDpiForSystem();
        let dpi_scale = if dpi == 0 {
            1.0
        } else {
            (dpi as f32 / 96.0).clamp(0.5, 4.0)
        };
        let left = (pos[0] * dpi_scale) as i32;
        let top = (pos[1] * dpi_scale) as i32;
        let w_phys = (size[0] * dpi_scale).max(1.0) as i32;
        let h_phys = (size[1] * dpi_scale).max(1.0) as i32;
        let rect = [left, top, left + w_phys, top + h_phys];
        let monitor = MonitorFromRect(&rect as *const [i32; 4], MONITOR_DEFAULTTONULL);
        !monitor.is_null()
    }
    #[cfg(not(windows))]
    {
        pos[0] >= 0.0 && pos[1] >= 0.0
    }
}

/// Calculates default spawn position for the toolstrip according to spec:
/// Centered safely on primary display.
pub fn calculate_toolstrip_spawn_pos(panel_width: f32) -> Option<[f32; 2]> {
    Some(calculate_primary_screen_center(panel_width, 70.0))
}
