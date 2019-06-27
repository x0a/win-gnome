#![windows_subsystem = "windows"]
extern crate winapi;

use std::ptr::null_mut;
use std::thread::sleep;
use std::time::Duration;

use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{
    keybd_event, FindWindowExW, FindWindowW, GetDesktopWindow, GetForegroundWindow,
    GetWindowRect, MessageBoxW, RegisterHotKey, ShowWindow, GetClassNameW, KEYEVENTF_KEYUP, MB_ICONEXCLAMATION,
    MB_OK, MOD_WIN, SW_HIDE, SW_SHOW, VK_ESCAPE, VK_LWIN, WM_MOUSEMOVE
};
mod window;
use window::win32_string;

#[macro_use]
mod hook;
pub mod errors;

const IDENTIFIER: &str = "win_gnome";
// We have to encode text to wide format for Windows
#[allow(dead_code)]
struct Tray {
    tray_height: i32,
    tray_handle: HWND,
    start_menu: HWND,
    icon_overflow: HWND,
    showing: bool,
}
impl Tray {
    fn new() -> Tray {
        let tray_class = win32_string("Shell_TrayWnd");
        let start_menu_class = win32_string("Windows.UI.Core.CoreWindow");
        let overflow_class = win32_string("NotifyIconOverflowWindow");

        unsafe {
            let tray_handle = FindWindowW(tray_class.as_ptr(), null_mut());
            let start_menu = FindWindowW(start_menu_class.as_ptr(), null_mut());
            let icon_overflow = FindWindowW(overflow_class.as_ptr(), null_mut());
            let (_, tray_height) = Desktop::get_window_dimensions(tray_handle);

            return Tray {
                start_menu,
                tray_handle,
                tray_height,
                icon_overflow,
                showing: true,
            };
        }
    }

    fn hide(&mut self) {
        unsafe {
            ShowWindow(self.tray_handle, SW_HIDE);
            self.showing = false;
        }
    }

    fn show(&mut self) {
        unsafe {
            ShowWindow(self.tray_handle, SW_SHOW);
            self.showing = true;
        }
    }

}

struct Desktop {
    height: i32,
    width: i32,
    enabled: bool,
    window: HWND,
    tray: Tray,
}

impl Desktop {
    unsafe fn new() -> Desktop {
        let g_desktop = GetDesktopWindow();
        let (width, height, window) = Desktop::get_actual_desktop(g_desktop);
        let tray = Tray::new();

        return Desktop {
            height,
            width,
            tray,
            window,
            enabled: true,
        };
    }
    fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        return self.enabled;
    }
    fn is_bottom_left(&self, x: i32, y: i32) -> bool {
        return x < 45 && y > self.height - self.tray.tray_height;
    }

    fn is_tray_region(&self, y: i32) -> bool {
        return y > self.height - self.tray.tray_height;
    }

    fn _tray_focused(&self) -> bool {
        unsafe {
            let fg: HWND = GetForegroundWindow();
            return fg == self.tray.icon_overflow
                || fg == self.tray.tray_handle
                || fg == self.tray.start_menu;
        }
    }
    pub unsafe fn get_actual_desktop(g_desktop: HWND) -> (i32, i32, HWND) {
        let (width, height) = Desktop::get_window_dimensions(g_desktop);
        let desktop_window = Desktop::find_matching_window(g_desktop, width, height, "WorkerW")
            .or_else(|| Desktop::find_matching_window(g_desktop, width, height, "Progman"))
            .expect("Could not find window");

        return (width, height, desktop_window);
    }
    pub unsafe fn find_matching_window(
        parent: HWND,
        width: i32,
        height: i32,
        class_name: &str,
    ) -> Option<HWND> {
        let search = win32_string(class_name).as_ptr();
        let mut desktop_window = FindWindowW(search, null_mut());

        while !desktop_window.is_null() {
            let (_width, _height) = Desktop::get_window_dimensions(desktop_window);
            if _width == width && _height == height {
                // found a window that matches resolution
                break;
            }
            desktop_window = FindWindowExW(parent, desktop_window, search, null_mut());

        }
        if desktop_window.is_null() {
            return None;
        } else {
            return Some(desktop_window);
        }
    }
    unsafe fn open_start_menu(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    unsafe fn is_menu_open(&self) -> bool {
        let (_, b, _, _) = Desktop::get_window_pos(self.tray.tray_handle);
        return b <= self.height + (self.tray.tray_height / 2); // still halfway shown
    }
    unsafe fn full_screen_program(&self) -> bool {
        let current = GetForegroundWindow();
        if current == self.window {
            return false;
        }
        let (top, bottom, left, right) = Desktop::get_window_pos(current);
        return top == 0 && left == 0 && bottom == self.height && right == self.width;
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
    unsafe fn _debug_cur_window(&self) {
        let current = GetForegroundWindow();
        self._debug_window(current);
    }
    unsafe fn _debug_window(&self, window: HWND) {
        let mut name_buffer:Vec<u16> = vec![0; 255];

        let count = GetClassNameW(window, name_buffer.as_mut_ptr(), 255) as usize;
        let class_name = String::from_utf16(&name_buffer[0..count]).unwrap();

        let (w, h) = Desktop::get_window_dimensions(window);
        println!(
            "className: '{}' {:?}, {}x{}",
            class_name,
            window,
            w,
            h
        );
    }
}
#[allow(non_upper_case_globals)]
static mut desktop: Desktop = Desktop {
    height: 0,
    width: 0,
    enabled: true,
    window: null_mut(),
    tray: Tray {
        tray_handle: null_mut(),
        start_menu: null_mut(),
        icon_overflow: null_mut(),
        tray_height: 0,
        showing: false,
    },
};

static mut DELAY: bool = false;
static mut LASTX: i32 = 0;
static mut LASTY: i32 = 0;

unsafe fn delay_further(ms: u64) {
    DELAY = true;
    std::thread::spawn(move || {
        let dur = Duration::from_millis(ms);
        sleep(dur);
        DELAY = false;
        mouse_move(LASTX, LASTY);
    });
}

fn mouse_move(x: i32, y: i32) {
    unsafe {
        if !desktop.enabled || DELAY {
            LASTX = x;
            LASTY = y;
            return ();
        }
        //desktop._debug_cur_window();
        if !desktop.tray.showing {
            if desktop.is_bottom_left(x, y) && !desktop.full_screen_program() {
                desktop.tray.showing = true;
                desktop.open_start_menu();
                desktop.tray.show();
                delay_further(300);
            }
        } else {
            if !desktop.is_tray_region(y) && !desktop.is_menu_open() {
                desktop.tray.showing = false;
                desktop.tray.hide();
            }
        }

    }

}
windows_hook! {
    pub fn mouse_hook(context: &mut MouseLL){
        if context.message() == WM_MOUSEMOVE{
                mouse_move(context.pt_x(), context.pt_y());
        }
    }
}

fn main() {
    unsafe {
        if window::previous_instance(IDENTIFIER) {
            MessageBoxW(
                null_mut(),
                win32_string("Previous instance of win-gnome already running").as_ptr(),
                win32_string("WinGnome").as_ptr(),
                MB_OK | MB_ICONEXCLAMATION,
            );
            return ();
        }

        let mut _window = window::create_hidden_window(IDENTIFIER).unwrap();
        RegisterHotKey(_window.handle, 0, MOD_WIN as u32, VK_ESCAPE as u32);
        desktop = Desktop::new();
        desktop.tray.hide();

        let hotkey_callback = || {
            if !desktop.toggle() {
                desktop.tray.show();
            } else {
                desktop.tray.hide();
            }
            return true;
        };

        let _hook = mouse_hook().unwrap();

        loop {
            if !window::handle_message(&mut _window, &hotkey_callback) {
                break;
            }
        }
    }

}