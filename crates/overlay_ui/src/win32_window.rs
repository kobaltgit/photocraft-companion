#[cfg(windows)]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
#[cfg(windows)]
const SM_CXSCREEN: i32 = 0;
#[cfg(windows)]
const SM_CYSCREEN: i32 = 1;
#[cfg(windows)]
const SM_CXSMICON: i32 = 49;
#[cfg(windows)]
const SM_CXICON: i32 = 11;
#[cfg(windows)]
const MONITOR_DEFAULTTONULL: u32 = 0;
#[cfg(windows)]
const WM_SETICON: u32 = 0x0080;
#[cfg(windows)]
const ICON_SMALL: usize = 0;
#[cfg(windows)]
const ICON_BIG: usize = 1;
#[cfg(windows)]
const SWP_NOSIZE: u32 = 0x0001;
#[cfg(windows)]
const SWP_NOMOVE: u32 = 0x0002;
#[cfg(windows)]
const SWP_NOZORDER: u32 = 0x0004;
#[cfg(windows)]
const SWP_NOACTIVATE: u32 = 0x0010;
#[cfg(windows)]
const SWP_FRAMECHANGED: u32 = 0x0020;

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
    fn CreateIconFromResourceEx(
        presbits: *const u8,
        dwResSize: u32,
        fIcon: i32,
        dwVer: u32,
        cxDesired: i32,
        cyDesired: i32,
        Flags: u32,
    ) -> *mut std::ffi::c_void;
    fn SendMessageW(hWnd: *mut std::ffi::c_void, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn GetPropW(hWnd: *mut std::ffi::c_void, lpString: *const u16) -> *mut std::ffi::c_void;
    fn SetPropW(
        hWnd: *mut std::ffi::c_void,
        lpString: *const u16,
        hData: *mut std::ffi::c_void,
    ) -> i32;
    fn EnumThreadWindows(
        dwThreadId: u32,
        lpfn: unsafe extern "system" fn(*mut std::ffi::c_void, isize) -> i32,
        lParam: isize,
    ) -> i32;
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> *mut std::ffi::c_void;
    fn SetWindowPos(
        hWnd: *mut std::ffi::c_void,
        hWndInsertAfter: *mut std::ffi::c_void,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetCurrentThreadId() -> u32;
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

/// Calculates adjacent spawn position for an interactive plugin window next to the toolstrip.
/// toolbar_rect: [min_x, min_y, max_x, max_y]
/// window_size: [width, height]
/// is_vertical: whether the toolstrip is vertically oriented
pub fn calculate_adjacent_spawn_position(
    toolbar_rect: [f32; 4],
    window_size: [f32; 2],
    is_vertical: bool,
) -> [f32; 2] {
    if !toolbar_rect.iter().all(|v| v.is_finite()) || !window_size.iter().all(|v| v.is_finite()) {
        return [100.0, 100.0];
    }

    let min_x = toolbar_rect[0];
    let min_y = toolbar_rect[1];
    let max_x = toolbar_rect[2];
    let max_y = toolbar_rect[3];
    let offset = 8.0;

    if is_vertical {
        // Try placing to the right of the vertical toolbar
        let right_pos = [max_x + offset, min_y];
        if is_position_visible_on_screen(right_pos, window_size) {
            return right_pos;
        }
        // Try placing to the left of the vertical toolbar
        let left_pos = [min_x - window_size[0] - offset, min_y];
        if is_position_visible_on_screen(left_pos, window_size) {
            return left_pos;
        }
    } else {
        // Try placing below the horizontal toolbar
        let bottom_pos = [min_x, max_y + offset];
        if is_position_visible_on_screen(bottom_pos, window_size) {
            return bottom_pos;
        }
        // Try placing above the horizontal toolbar
        let top_pos = [min_x, min_y - window_size[1] - offset];
        if is_position_visible_on_screen(top_pos, window_size) {
            return top_pos;
        }
    }

    // Fallback: place aligned with toolbar start or clamp
    [min_x + offset, min_y + offset]
}

/// Returns the system small icon dimension (width in pixels) based on SM_CXSMICON.
/// Defaults to 16 if not on Windows or if query fails.
pub fn get_system_small_icon_size() -> u32 {
    #[cfg(windows)]
    unsafe {
        let size = GetSystemMetrics(SM_CXSMICON);
        if size > 0 {
            size as u32
        } else {
            16
        }
    }
    #[cfg(not(windows))]
    {
        16
    }
}

/// Returns the system large window icon dimension (width in pixels) based on SM_CXICON.
/// Defaults to 64 if not on Windows or if query fails.
pub fn get_system_window_icon_size() -> u32 {
    #[cfg(windows)]
    unsafe {
        let s = GetSystemMetrics(SM_CXICON);
        if s > 0 {
            (s as u32).max(64)
        } else {
            64
        }
    }
    #[cfg(not(windows))]
    {
        64
    }
}

#[cfg(windows)]
static NATIVE_ICONS: std::sync::OnceLock<Option<(isize, isize)>> = std::sync::OnceLock::new();

#[cfg(windows)]
fn get_or_create_native_icons() -> Option<(isize, isize)> {
    *NATIVE_ICONS.get_or_init(|| {
        let small_size = get_system_small_icon_size();
        let big_size = get_system_window_icon_size();

        // 1. Dedicated small icon rendered strictly from icon_small.svg for window caption corner
        let rgba_small = crate::icons::render_svg_icon(
            crate::icons::APP_ICON_SMALL_SVG,
            small_size,
            small_size,
        )?;
        let dib_small = crate::icons::encode_ico_dib(&rgba_small, small_size, small_size);

        // 2. Original high-resolution icon from main icon.svg for taskbar and alt-tab
        let rgba_big =
            crate::icons::render_svg_icon(crate::icons::APP_ICON_SVG, big_size, big_size)?;
        let dib_big = crate::icons::encode_ico_dib(&rgba_big, big_size, big_size);

        unsafe {
            let hicon_small = CreateIconFromResourceEx(
                dib_small.as_ptr(),
                dib_small.len() as u32,
                1,
                0x00030000,
                small_size as i32,
                small_size as i32,
                0,
            );

            let hicon_big = CreateIconFromResourceEx(
                dib_big.as_ptr(),
                dib_big.len() as u32,
                1,
                0x00030000,
                big_size as i32,
                big_size as i32,
                0,
            );

            let h_small = if !hicon_small.is_null() {
                hicon_small as isize
            } else {
                0
            };

            let h_big = if !hicon_big.is_null() {
                hicon_big as isize
            } else {
                h_small
            };

            if h_small != 0 || h_big != 0 {
                Some((h_small, h_big))
            } else {
                None
            }
        }
    })
}

#[cfg(windows)]
struct FindWindowContext<'a> {
    target_title: &'a str,
    found_hwnd: Option<*mut std::ffi::c_void>,
}

#[cfg(windows)]
unsafe extern "system" fn enum_thread_windows_callback(
    hwnd: *mut std::ffi::c_void,
    lparam: isize,
) -> i32 {
    let ctx = &mut *(lparam as *mut FindWindowContext);
    let mut buf = [0u16; 512];
    let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 512);
    if len > 0 {
        let win_title = String::from_utf16_lossy(&buf[..len as usize]);
        if win_title == ctx.target_title || win_title.contains(ctx.target_title) {
            ctx.found_hwnd = Some(hwnd);
            return 0; // Found target window
        }
    }
    1 // Continue enumeration
}

/// Applies crisp native Windows icons: icon_small.svg strictly for the window corner,
/// and full-resolution icon.svg for taskbar and alt-tab.
/// Returns true if the icons have been successfully calibrated or were already calibrated.
#[cfg(windows)]
pub fn apply_crisp_window_icons_by_title(title: &str) -> bool {
    let mut ctx = FindWindowContext {
        target_title: title,
        found_hwnd: None,
    };

    unsafe {
        let thread_id = GetCurrentThreadId();
        let lparam = &mut ctx as *mut FindWindowContext as isize;
        EnumThreadWindows(thread_id, enum_thread_windows_callback, lparam);

        let hwnd = ctx.found_hwnd.unwrap_or_else(|| {
            let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            FindWindowW(std::ptr::null(), wide_title.as_ptr())
        });

        if !hwnd.is_null() {
            const PROP_NAME: &[u16] = &[
                b'P' as u16,
                b'C' as u16,
                b'_' as u16,
                b'C' as u16,
                b'R' as u16,
                b'I' as u16,
                b'S' as u16,
                b'P' as u16,
                b'_' as u16,
                b'I' as u16,
                b'C' as u16,
                b'O' as u16,
                b'N' as u16,
                b'S' as u16,
                0,
            ];
            let prop = GetPropW(hwnd, PROP_NAME.as_ptr());
            if prop.is_null() {
                if let Some((hicon_small, hicon_big)) = get_or_create_native_icons() {
                    if hicon_small != 0 {
                        SendMessageW(hwnd, WM_SETICON, ICON_SMALL, hicon_small);
                    }
                    if hicon_big != 0 {
                        SendMessageW(hwnd, WM_SETICON, ICON_BIG, hicon_big);
                    }

                    // Immediately redraw non-client area (titlebar frame)
                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );

                    SetPropW(
                        hwnd,
                        PROP_NAME.as_ptr(),
                        std::ptr::dangling_mut::<std::ffi::c_void>(),
                    );
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    false
}

/// No-op fallback on non-Windows platforms.
#[cfg(not(windows))]
pub fn apply_crisp_window_icons_by_title(_title: &str) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_crisp_window_icons_smoke() {
        let _ = apply_crisp_window_icons_by_title("Non_Existent_Window_Test_Title_12345");
        #[cfg(windows)]
        {
            let icons = get_or_create_native_icons();
            assert!(
                icons.is_some(),
                "get_or_create_native_icons must return Some on Windows"
            );
        }
    }

    #[test]
    fn test_adjacent_spawn_vertical_toolbar() {
        let toolbar_rect = [100.0, 100.0, 140.0, 500.0];
        let window_size = [300.0, 400.0];
        let pos = calculate_adjacent_spawn_position(toolbar_rect, window_size, true);
        assert!(pos[0].is_finite());
        assert!(pos[1].is_finite());
        assert!(pos[0] == 148.0 || pos[0] == -208.0 || pos[0] == 108.0);
        assert!(pos[1] == 100.0 || pos[1] == 108.0);
    }

    #[test]
    fn test_adjacent_spawn_horizontal_toolbar() {
        let toolbar_rect = [200.0, 100.0, 600.0, 140.0];
        let window_size = [320.0, 450.0];
        let pos = calculate_adjacent_spawn_position(toolbar_rect, window_size, false);
        assert!(pos[0].is_finite());
        assert!(pos[1].is_finite());
        assert!(pos[1] == 148.0 || pos[1] == -358.0 || pos[1] == 108.0);
        assert!(pos[0] == 200.0 || pos[0] == 208.0);
    }

    #[test]
    fn test_adjacent_spawn_nan_inf_handling() {
        let toolbar_rect = [f32::NAN, 100.0, 140.0, 500.0];
        let window_size = [300.0, f32::INFINITY];
        let pos = calculate_adjacent_spawn_position(toolbar_rect, window_size, true);
        assert!(pos[0].is_finite());
        assert!(pos[1].is_finite());
        assert_eq!(pos, [100.0, 100.0]);
    }
}
