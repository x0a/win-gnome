use crate::window::win32_string;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{
    keybd_event, FindWindowExW, FindWindowW, GetClassNameW, GetDesktopWindow, GetForegroundWindow,
    GetWindowRect, ShowWindow, IsWindow, GetWindowLongW, GetParent, 
    KEYEVENTF_KEYUP, SW_HIDE, SW_SHOW, VK_LWIN, VK_TAB, GWL_STYLE, GWL_EXSTYLE, WS_VISIBLE
};
use winapi::um::dwmapi::{DwmGetWindowAttribute, DwmFlush, DWMWA_CLOAKED};
use winapi::ctypes::c_void;
use std::ptr::null_mut;
#[allow(dead_code)]

pub enum TrayOrientation {
    Bottom,
    Top,
    Left,
    Right,
}
pub struct Tray {
    pub orientation: TrayOrientation,
    pub bar: HWND,
    pub start_button: HWND,
    pub start_menu: HWND,
    pub icon_overflow: HWND,
    pub showing: bool,
    pub startmenu_showing: bool,
    pub overflow_showing: bool,
    pub parent_width: i32,
    pub parent_height: i32,
    pub start_height: i32,
    pub start_width: i32,
    pub hot_width: i32,
    pub hot_height: i32,
}
impl Tray {
    fn new(parent: HWND, parent_width: i32, parent_height: i32) -> Tray {
        let bar = Desktop::find_window(Some("Shell_TrayWnd"), None)
            .expect("Could not find window for system tray");
        let start_button = Desktop::find_child(bar, "Start")
            .expect("Could not find start button");
        let icon_overflow = Desktop::find_window(Some("NotifyIconOverflowWindow"), None)
            .unwrap_or_else(|| null_mut());

        let (start_width, start_height) = Desktop::get_window_dimensions(start_button);
        let (hot_width, hot_height) = (
            Tray::apply_sensitivity(start_width,  unsafe{ super::SENSITIVITY }),
            Tray::apply_sensitivity(start_height,  unsafe{ super::SENSITIVITY }));
        let orientation = Tray::get_orientation(parent_width, parent_height, Desktop::get_window_pos(bar));
        let start_menu = Desktop::find_by_position(
            parent,
            "Windows.UI.Core.CoreWindow",
            Tray::get_menu_offsets(start_width, parent_width, &orientation))
            .expect("Unable to find start menu");
        return Tray {
            orientation,
            bar,
            start_button,
            start_menu,
            icon_overflow,
            parent_width,
            parent_height,
            start_width,
            start_height,
            hot_width,
            hot_height,
            showing: true,
            startmenu_showing: false,
            overflow_showing: false
        };
    }
    pub fn default() -> Tray {
        return Tray {
            orientation: TrayOrientation::Bottom,
            bar: null_mut(),
            start_button: null_mut(),
            start_menu: null_mut(),
            icon_overflow: null_mut(),
            parent_width: 0,
            parent_height: 0,
            start_width: 0,
            start_height: 0,
            hot_width: 0,
            hot_height: 0,
            showing: true,
            startmenu_showing: false,
            overflow_showing: false
        };
    }
    fn apply_sensitivity(dimension: i32, sensitivity: i32) -> i32{
        let dimension = dimension as f32;
        let sensitivity = sensitivity as f32;
        ((dimension / 100 as f32) * sensitivity).floor() as i32
    }

    pub fn get_menu_offsets(button_width: i32, parent_width: i32, orientation: &TrayOrientation) -> (Option<i32>, Option<i32>, Option<i32>, Option<i32>) {
        match orientation{
            TrayOrientation::Bottom => (None, None, Some(button_width), None),
            TrayOrientation::Top => (None, None, Some(button_width), None),
            TrayOrientation::Left => (Some(0), None, Some(button_width), None),
            TrayOrientation::Right => (Some(0), None, None, Some(parent_width - button_width))
        }
    }
    pub fn is_tray_open(&self) -> bool {
        let (top, bottom, left, right) = Desktop::get_window_pos(self.bar);

        match self.orientation {
            TrayOrientation::Bottom => bottom == self.parent_height,
            TrayOrientation::Top => top == 0,
            TrayOrientation::Left => left == 0,
            TrayOrientation::Right => right == self.parent_width,
        }
    }
    pub fn is_tray_region(&self, x: i32, y: i32) -> bool {
        match self.orientation {
            TrayOrientation::Bottom => y > self.parent_height - self.start_height,
            TrayOrientation::Top => y < self.start_height,
            TrayOrientation::Left => x < self.start_width,
            TrayOrientation::Right => x > self.parent_width - self.start_width,
        }
    }
    pub fn is_hot_corner(&self, x: i32, y: i32) -> bool {
        match self.orientation {
            TrayOrientation::Bottom => x < self.hot_width && y > self.parent_height - self.hot_height,
            TrayOrientation::Top => x < self.hot_width && y < self.hot_height,
            TrayOrientation::Left => x < self.hot_width && y < self.hot_height,
            TrayOrientation::Right => x > self.parent_width  - self.hot_width && y < self.hot_height
        }
    }
    pub fn hide(&mut self) -> bool {
        if unsafe { IsWindow(self.bar) == 0 } {
            return false;
        }
        unsafe { ShowWindow(self.bar, SW_HIDE) };
        self.showing = false;
        true
    }
    pub fn show(&mut self) {
        unsafe { ShowWindow(self.bar, SW_SHOW) };
        self.showing = true;
    }
    fn get_orientation(
        width: i32,
        height: i32,
        (top, bottom, left, right): (i32, i32, i32, i32),
    ) -> TrayOrientation {
        if right - left > bottom - top {
            // top or bottom
            if top < height / 2 {
                TrayOrientation::Top
            } else {
                TrayOrientation::Bottom
            }
        } else {
            if left < width / 2 {
                TrayOrientation::Left
            } else {
                TrayOrientation::Right
            }
        }
    }
}

pub struct Desktop {
    pub height: i32,
    pub width: i32,
    pub enabled: bool,
    pub last_window: HWND,
    pub shell_window: HWND,
    pub shell_parent: HWND,
    pub tray: Tray,
}

impl Desktop {
    pub fn new() -> Desktop {
        let mut desktop = Desktop::default();
        desktop.refresh();
        return desktop;
    }
    pub fn default() -> Desktop {
        return Desktop {
            height: 0,
            width: 0,
            enabled: true,
            last_window: null_mut(),
            shell_window: null_mut(),
            shell_parent: null_mut(),
            tray: Tray::default(),
        };
    }
    pub fn refresh(&mut self) {
        let top_desktop = unsafe { GetDesktopWindow() };
        let (width, height, shell_window, shell_parent) = Desktop::get_actual_desktop(top_desktop);
        let tray = Tray::new(top_desktop, width, height);

        self.height = height;
        self.width = width;
        self.shell_window = shell_window;
        self.shell_parent = shell_parent;
        self.tray = tray;
        self.foreground_changed(unsafe { GetForegroundWindow() });
        /* unsafe {
            self._debug_window(shell_window);
            self._debug_window(shell_parent);
            self._debug_window(self.tray.bar);
            self._debug_window(self.tray.start_menu);
            self._debug_window(self.tray.start_button);
        } */
    }
    pub fn foreground_changed(&mut self, window: HWND) -> bool{
        self.last_window = window;
        
        if self.tray.start_menu == window{
            self.tray.show();
            self.tray.startmenu_showing = true;
        } else if self.tray.icon_overflow == window {
            self.tray.overflow_showing = true;
        } else if self.tray.overflow_showing || self.tray.startmenu_showing{
            std::thread::sleep(std::time::Duration::from_millis(50));
            unsafe { DwmFlush() };
            if self.tray.overflow_showing {
                let styles = unsafe { GetWindowLongW(self.tray.icon_overflow, GWL_STYLE) } as u32;
                if styles & WS_VISIBLE == 0 {
                    self.tray.overflow_showing = false;
                    return true;
                }
            }
            if self.tray.startmenu_showing {
                let mut cloaked: u32 = 0;
                unsafe { 
                    DwmGetWindowAttribute(self.tray.start_menu, 
                        DWMWA_CLOAKED,
                        &mut cloaked as *mut u32 as *mut c_void,
                        std::mem::size_of::<u32>() as u32);
                }
                if cloaked != 0 {
                    self.tray.startmenu_showing = false;
                    return true;
                }
            }
        }
        false
    }
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    fn _tray_focused(&self) -> bool {
        self.last_window == self.tray.icon_overflow
            || self.last_window == self.tray.bar
            || self.last_window == self.tray.start_menu
    }
    pub fn get_actual_desktop(top_desktop: HWND) -> (i32, i32, HWND, HWND) {
        let (width, height) = Desktop::get_window_dimensions(top_desktop);
        let shell_parent = Desktop::find_by_dimensions(top_desktop, "WorkerW", width, height)
            .or_else(|| Desktop::find_by_dimensions(top_desktop, "Progman", width, height))
            .expect("Could not find desktop window");
        let shell_window = Desktop::find_child(shell_parent, "SHELLDLL_DefView")
            .expect("Couldn't find shell");

        (width, height, shell_window, shell_parent)
    }
    pub fn find_by_dimensions(
        parent: HWND,
        class_name: &str,
        width: i32,
        height: i32,
    ) -> Option<HWND> {
        let search = win32_string(class_name).as_ptr();
        let mut desktop_window = unsafe { FindWindowW(search, null_mut()) };

        while !desktop_window.is_null() {
            let (_width, _height) = Desktop::get_window_dimensions(desktop_window);
            if _width == width && _height == height {
                return Some(desktop_window);
            }
            desktop_window = unsafe { FindWindowExW(parent, desktop_window, search, null_mut()) };
        }
        None
    }
    pub fn find_child(parent: HWND, class_name: &str) -> Option<HWND> {
        let search = win32_string(class_name).as_ptr();
        let child = unsafe { FindWindowExW(parent, null_mut(), search, null_mut()) };
        
        if !child.is_null() {
            Some(child)
        } else {
            None
        }
    }
    pub fn find_window(class_name: Option<&str>, title: Option<&str>) -> Option<HWND> {
        let class_name = match class_name {
            Some(name) => win32_string(name).as_ptr(),
            None => null_mut(),
        };
        let title = match title {
            Some(title_str) => win32_string(title_str).as_ptr(),
            None => null_mut(),
        };
        let hwnd = unsafe { FindWindowW(class_name, title) };

        if hwnd.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
    pub fn find_by_position(
        parent: HWND,
        class_name: &str,
        (top, bottom, left, right):
        (Option<i32>, Option<i32>, Option<i32>, Option<i32>)
    ) -> Option<HWND> {

        let search = win32_string(class_name).as_ptr();
        let mut desktop_window = unsafe { FindWindowW(search, null_mut()) };

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

            desktop_window = unsafe { FindWindowExW(parent, desktop_window, search, null_mut()) };
        }
        None
    }

    pub unsafe fn open_start_menu(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }

    pub unsafe fn open_desktop_selector(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_TAB as u8, 0, 0, 0);
        keybd_event(VK_TAB as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    pub fn full_screen_program(&self) -> bool {
        if self.last_window == self.shell_parent || self.last_window == self.shell_window {
            return false;
        }
        let (top, bottom, left, right) = Desktop::get_window_pos(self.last_window);
        top == 0 && left == 0 && bottom == self.height && right == self.width
    }
    pub unsafe fn update_desktop(&mut self) -> bool {
        if IsWindow(self.shell_window) == 0 || GetParent(self.shell_window) != self.shell_parent {
            self.refresh();
            true
        } else {
            false
        }
    }

    pub fn get_window_dimensions(handle: HWND) -> (i32, i32) {
        let (top, bottom, left, right) = Desktop::get_window_pos(handle);
        (right - left, bottom - top)
    }
    pub fn get_window_pos(handle: HWND) -> (i32, i32, i32, i32) {
        let dimensions: *mut RECT = &mut RECT {
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
        };
        unsafe {
            GetWindowRect(handle, dimensions);
            (
                (*dimensions).top,
                (*dimensions).bottom,
                (*dimensions).left,
                (*dimensions).right,
            )
        }
    }
    pub unsafe fn _debug_cur_window(&self) {
        let current = GetForegroundWindow();
        self._debug_window(current);
    }
    unsafe fn _get_class_name(&self, window: HWND) -> Option<String> {
        let class_buffer: Vec<u16> = vec![0; 255];
        let char_count = GetClassNameW(window, class_buffer.as_ptr(), 255) as usize;
        if char_count != 0 {
            String::from_utf16(&class_buffer[0..char_count]).ok()
        } else {
            None
        }
    }
    pub unsafe fn _debug_window(&self, window: HWND) {
        let class_name = self._get_class_name(window);
        let extended_styles = GetWindowLongW(window, GWL_EXSTYLE);
        match class_name {
            Some(name) => {
                let (t, b, l, r) = Desktop::get_window_pos(window);
                let (w, h) = Desktop::get_window_dimensions(window);
                if window == self.shell_parent {
                    print!("[SHELLP] ");
                } else if window == self.shell_window {
                    print!("[SHELL] ");
                } else if name == "WorkerW" {
                    print!("[WDESKHANDL: {:?}]", self.shell_parent);
                }
                println!(
                    "class: '{}' | handle: {:?} | style: {} | {}x{} | top: {}, bottom: {}, left: {}, right: {}",
                    name, window, extended_styles, w, h, t, b, l, r
                );
            }
            None => println!("Couldn't get classname"),
        }
    }
}
