use crate::window::win32_string;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{
    keybd_event, FindWindowExW, FindWindowW, GetClassNameW, GetDesktopWindow, GetForegroundWindow,
    GetWindowRect, ShowWindow, IsWindow, KEYEVENTF_KEYUP, SW_HIDE, SW_SHOW, VK_LWIN, VK_TAB,
};
use std::ptr::null_mut;


#[allow(dead_code)]
pub struct Tray {
    pub tray_height: i32,
    pub tray_handle: HWND,
    pub start_menu: HWND,
    pub icon_overflow: HWND,
    pub showing: bool,
}
impl Tray {
    fn new(parent: HWND) -> Tray {
        unsafe {
            let tray_handle =
                Desktop::find_window(Some("Shell_TrayWnd"), None).unwrap_or_else(|| null_mut());
            let icon_overflow = Desktop::find_window(Some("NotifyIconOverflowWindow"), None)
                .unwrap_or_else(|| null_mut());
            let (_, tray_height) = Desktop::get_window_dimensions(tray_handle);
            let start_menu = Desktop::find_by_position(
                parent,
                "Windows.UI.Core.CoreWindow",
                None,
                None,
                Some(Tray::get_menu_offset(tray_height)),
                None,
            )
            .unwrap_or_else(|| null_mut());

            return Tray {
                start_menu,
                tray_handle,
                tray_height,
                icon_overflow,
                showing: true,
            };
        }
    }
    pub fn get_menu_offset(tray_height: i32) -> i32 {
        //  GetWindowPos(Windows.UI.Core.CoreWindow).left is always tray_height + trayheight / 10 * 2
        // So we use that to find the start menu

        return tray_height + ((tray_height / 10) * 2);
    }
    pub fn hide(&mut self) {
        unsafe {
            ShowWindow(self.tray_handle, SW_HIDE);
            self.showing = false;
        }
    }

    pub fn show(&mut self) {
        unsafe {
            ShowWindow(self.tray_handle, SW_SHOW);
            self.showing = true;
        }
    }

}

pub struct Desktop {
    pub height: i32,
    pub width: i32,
    pub enabled: bool,
    pub last_window: HWND,
    pub window: HWND,
    pub tray: Tray,
}

impl Desktop {
    pub unsafe fn new() -> Desktop {
        let mut next = Desktop {
            height: 0,
            width: 0,
            enabled: true,
            last_window: null_mut(),
            window: null_mut(),
            tray: Tray {
                tray_handle: null_mut(),
                start_menu: null_mut(),
                icon_overflow: null_mut(),
                tray_height: 0,
                showing: false,
            },
        };
        next.refresh();
        return next;
    }
    pub unsafe fn refresh(&mut self) {
        let top_desktop = GetDesktopWindow();
        let (width, height, window) = Desktop::get_actual_desktop(top_desktop);
        let tray = Tray::new(top_desktop);

        self.height = height;
        self.width = width;
        self.window = window;
        self.tray = tray;
    }
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        return self.enabled;
    }
    pub fn is_bottom_left(&self, x: i32, y: i32) -> bool {
        return x < 45 && y > self.height - self.tray.tray_height;
    }

    pub fn is_tray_region(&self, y: i32) -> bool {
        return y > self.height - self.tray.tray_height;
    }

    fn _tray_focused(&self) -> bool {
        return self.last_window == self.tray.icon_overflow
            || self.last_window == self.tray.tray_handle
            || self.last_window == self.tray.start_menu;
    }
    pub unsafe fn get_actual_desktop(top_desktop: HWND) -> (i32, i32, HWND) {
        let (width, height) = Desktop::get_window_dimensions(top_desktop);
        let desktop_window = Desktop::find_by_dimensions(top_desktop, "WorkerW", width, height)
            .or_else(|| Desktop::find_by_dimensions(top_desktop, "Progman", width, height))
            .expect("Could not find desktop window");

        return (width, height, desktop_window);
    }
    pub unsafe fn find_by_dimensions(
        parent: HWND,
        class_name: &str,
        width: i32,
        height: i32,
    ) -> Option<HWND> {
        let search = win32_string(class_name).as_ptr();
        let mut desktop_window = FindWindowW(search, null_mut());

        while !desktop_window.is_null() {
            let (_width, _height) = Desktop::get_window_dimensions(desktop_window);
            if _width == width && _height == height {
                return Some(desktop_window);
            }
            desktop_window = FindWindowExW(parent, desktop_window, search, null_mut());

        }
        return None;
    }
    pub unsafe fn find_window(class_name: Option<&str>, title: Option<&str>) -> Option<HWND> {
        let class_name = match class_name {
            Some(name) => win32_string(name).as_ptr(),
            None => null_mut(),
        };
        let title = match title {
            Some(title_str) => win32_string(title_str).as_ptr(),
            None => null_mut(),
        };
        let hwnd = FindWindowW(class_name, title);

        if hwnd.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
    pub unsafe fn find_by_position(
        parent: HWND,
        class_name: &str,
        top: Option<i32>,
        bottom: Option<i32>,
        left: Option<i32>,
        right: Option<i32>,
    ) -> Option<HWND> {
        let search = win32_string(class_name).as_ptr();
        let mut desktop_window = FindWindowW(search, null_mut());

        while !desktop_window.is_null() {
            let (t, b, l, r) = Desktop::get_window_pos(desktop_window);

            let top_match = match top {
                Some(pos) => pos == t,
                None => true,
            };
            let bottom_match = match bottom {
                Some(pos) => pos == b,
                None => true,
            };
            let left_match = match left {
                Some(pos) => pos == l,
                None => true,
            };
            let right_match = match right {
                Some(pos) => pos == r,
                None => true,
            };

            if top_match && bottom_match && left_match && right_match {
                return Some(desktop_window);
            }

            desktop_window = FindWindowExW(parent, desktop_window, search, null_mut());

        }
        return None;
    }

    pub unsafe fn open_start_menu(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    unsafe fn open_desktop_selector(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_TAB as u8, 0, 0, 0);
        keybd_event(VK_TAB as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    pub unsafe fn is_tray_open(&self) -> bool {
        let (_, b, _, _) = Desktop::get_window_pos(self.tray.tray_handle);
        return b == self.height; //<= self.height + (self.tray.tray_height / 2); // still halfway shown
    }
    unsafe fn _is_menu_open(&self) -> bool {
        let (width, height) = Desktop::get_window_dimensions(self.tray.start_menu);
        let result = width == self.width; // && height != self.height;
        println!(
            "menu is open: {}, {}x{} {}x{}",
            result, width, height, self.width, self.height
        );
        return result; // || right > self.width;
    }
    pub unsafe fn full_screen_program(&self) -> bool {
        if self.last_window == self.window {
            return false;
        }
        let (top, bottom, left, right) = Desktop::get_window_pos(self.last_window);
        return top == 0 && left == 0 && bottom == self.height && right == self.width;
    }
    pub unsafe fn update_desktop(&mut self) -> bool {
        if IsWindow(self.window) == 0 {
            self.refresh();
            return true;
        } else {
            return false;
        }
    }

    pub unsafe fn get_window_dimensions(handle: HWND) -> (i32, i32) {
        let (top, bottom, left, right) = Desktop::get_window_pos(handle);
        return (right - left, bottom - top);
    }
    pub unsafe fn get_window_pos(handle: HWND) -> (i32, i32, i32, i32) {
        let dimensions: *mut RECT = &mut RECT {
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect(handle, dimensions);
        return (
            (*dimensions).top,
            (*dimensions).bottom,
            (*dimensions).left,
            (*dimensions).right,
        );
    }
    pub unsafe fn _debug_cur_window(&self) {
        let current = GetForegroundWindow();
        self._debug_window(current);
    }
    unsafe fn _get_class_name(&self, window: HWND) -> Option<String> {
        let class_buffer: Vec<u16> = vec![0; 255];
        let char_count = GetClassNameW(window, class_buffer.as_ptr(), 255) as usize;
        if char_count != 0 {
            return String::from_utf16(&class_buffer[0..char_count]).ok();
        } else {
            None
        }
    }
    pub unsafe fn _debug_window(&self, window: HWND) {
        let class_name = self._get_class_name(window);
        match class_name {
            Some(name) => {
                let (t, b, l, r) = Desktop::get_window_pos(window);
                let (w, h) = Desktop::get_window_dimensions(window);
                if window == self.window {
                    print!("[DESKTOP]");
                }
                println!(
                    "class: '{}' | handle: {:?} | {}x{} | top: {}, bottom: {}, left: {}, right: {}",
                    name, window, w, h, t, b, l, r
                );
            }
            None => println!("Couldn't get classname"),
        }
    }
}
