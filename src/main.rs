#![windows_subsystem = "windows"]
extern crate winapi;

use std::ptr::null_mut;
use std::thread;
use std::time::Duration;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{
    keybd_event, FindWindowExW, FindWindowW, GetClassNameW, GetDesktopWindow, GetForegroundWindow,
    GetWindowRect, MessageBoxW, RegisterHotKey, ShowWindow, KEYEVENTF_KEYUP, MB_ICONEXCLAMATION,
    MB_OK, MOD_WIN, SW_HIDE, SW_SHOW, VK_ESCAPE, VK_LWIN, VK_TAB, WM_MOUSEMOVE,
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
    fn new(parent: HWND) -> Tray {
        unsafe {
            let tray_handle =
                Desktop::find_window(Some("Shell_TrayWnd"), None).unwrap_or_else(|| null_mut()); //FindWindowW(tray_class, null_mut());
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
    last_window: HWND,
    window: HWND,
    tray: Tray,
}

impl Desktop {
    unsafe fn new() -> Desktop {
        let top_desktop = GetDesktopWindow();
        let (width, height, window) = Desktop::get_actual_desktop(top_desktop);
        let tray = Tray::new(top_desktop);

        return Desktop {
            height,
            width,
            tray,
            window,
            last_window: null_mut(),
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

    unsafe fn open_start_menu(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    unsafe fn open_desktop_selector(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_TAB as u8, 0, 0, 0);
        keybd_event(VK_TAB as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    unsafe fn is_tray_open(&self) -> bool {
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
    unsafe fn full_screen_program(&self) -> bool {
        if self.last_window == self.window {
            return false;
        }
        let (top, bottom, left, right) = Desktop::get_window_pos(self.last_window);
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
    unsafe fn _get_class_name(&self, window: HWND) -> Option<String> {
        let class_buffer: Vec<u16> = vec![0; 255];
        let char_count = GetClassNameW(window, class_buffer.as_ptr(), 255) as usize;
        if char_count != 0 {
            return String::from_utf16(&class_buffer[0..char_count]).ok()
        } else {
            None
        }
    }
    unsafe fn _debug_window(&self, window: HWND) {
        let class_name = self._get_class_name(window);
        match class_name {
            Some(name) => {
                let (t, b, l, r) = Desktop::get_window_pos(window);
                let (w, h) = Desktop::get_window_dimensions(window);
                println!(
                    "class: '{}' | handle: {:?} | {}x{} | top: {}, bottom: {}, left: {}, right: {}",
                    name, window, w, h, t, b, l, r
                );
            }
            None => println!("Couldn't get classname"),
        }
    }
}
#[allow(non_upper_case_globals)]
static mut desktop: Desktop = Desktop {
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

static mut DELAY: bool = false;
static mut LASTX: i32 = 0;
static mut LASTY: i32 = 0;

unsafe fn delay_next(ms: u64) {
    DELAY = true;
    std::thread::spawn(move || {
        let dur = Duration::from_millis(ms);
        thread::sleep(dur);
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
                desktop.tray.show();
                desktop.open_start_menu();
                delay_next(300);
            }
        } else {
            if !desktop.is_tray_region(y) && !desktop.is_tray_open() {
                desktop.tray.hide();
            }
        }

    }

}
enum CornerAction {
    open_menu,
    open_wintab,
}
windows_hook! {
    pub fn mouse_hook(context: &mut MouseLL){
        if context.message() == WM_MOUSEMOVE{
            mouse_move(context.pt_x(), context.pt_y());
        }
    }
}
winevent_hook! {
    pub fn fg_hook(context: &mut FgWinEvent){
        let hwnd = context.get_hwnd();
        unsafe{
            desktop.last_window = hwnd;
            if hwnd == desktop.tray.start_menu{
                desktop.tray.show();
            }
            // desktop._debug_window(hwnd);
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
        desktop = Desktop::new();

        let hotkey_callback = || {
            if !desktop.toggle() {
                desktop.tray.show();
            } else {
                desktop.tray.hide();
            }
            return true;
        };

        let close_callback = || {
            desktop.enabled = false;
            desktop.tray.show();
            return false;
        };

        let _mhook = mouse_hook().expect("Unable to install system-wide mouse hook");
        let _fhook = fg_hook().expect("Unable to install system-side foreground hook");
        RegisterHotKey(_window.handle, 0, MOD_WIN as u32, VK_ESCAPE as u32);

        desktop.tray.hide();

        loop {
            if !window::handle_message(&mut _window, &hotkey_callback, &close_callback) {
                break;
            }
        }
    }

}