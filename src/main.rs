#![windows_subsystem = "windows"]
use std::ffi::CString;

use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

#[macro_use]
mod hook;
pub mod errors;

extern crate winapi;

use self::winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use self::winapi::um::libloaderapi::GetModuleHandleW;
use self::winapi::um::winuser::{
    keybd_event, CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowExW, FindWindowW,
    GetClassNameA, GetDesktopWindow, GetForegroundWindow, GetMessageW, GetWindowRect,
    RegisterClassW, RegisterHotKey, ShowWindow, TranslateMessage,
};
use self::winapi::um::winuser::{
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, KEYEVENTF_KEYUP, MOD_WIN, MSG, SW_HIDE,
    SW_SHOW, VK_ESCAPE, VK_LWIN, WM_HOTKEY, WM_MOUSEMOVE, WNDCLASSW,
};
use winapi::shared::windef::{HWND, RECT};


// We have to encode text to wide format for Windows
#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

// Window struct
#[cfg(windows)]
struct Window {
    handle: HWND,
}

// Create window function
#[cfg(windows)]
fn create_window(name: &str, title: &str) -> Result<Window, Error> {
    let name = win32_string(name);
    let title = win32_string(title);

    unsafe {
        // Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
        let hinstance = GetModuleHandleW(null_mut());

        // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW, // Style
            lpfnWndProc: Some(DefWindowProcW), // The callbackfunction for any window event that can occur in our window!!! Here you could react to events like WM_SIZE or WM_QUIT.
            hInstance: hinstance, // The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            lpszClassName: name.as_ptr(), // Our class name which needs to be a UTF-16 string (defined earlier before unsafe). as_ptr() (Rust's own function) returns a raw pointer to the slice's buffer
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };

        // We have to register this class for Windows to use
        RegisterClassW(&wnd_class);

        // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
        // Create a window based on registered class
        let handle = CreateWindowExW(
            0,              // dwExStyle
            name.as_ptr(), // lpClassName, name of the class that we want to use for this window, which will be the same that we have registered before.
            title.as_ptr(), // lpWindowName
            0,             // dwStyle
            CW_USEDEFAULT, // Int x
            CW_USEDEFAULT, // Int y
            CW_USEDEFAULT, // Int nWidth
            CW_USEDEFAULT, // Int nHeight
            null_mut(),    // hWndParent
            null_mut(),    // hMenu
            hinstance,     // hInstance
            null_mut(),
        ); // lpParam

        if handle.is_null() {
            Err(Error::last_os_error())
        } else {
            Ok(Window { handle })
        }
    }
}

#[cfg(windows)]
// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message(window: &mut Window, on_hot_key: impl Fn() -> ()) -> bool {
    unsafe {
        let mut message: MSG = mem::uninitialized();

        // Get message from message queue with GetMessageW
        if GetMessageW(&mut message as *mut MSG, window.handle, 0, 0) > 0 {
            TranslateMessage(&message as *const MSG); // Translate message into something meaningful with TranslateMessage
            DispatchMessageW(&message as *const MSG); // Dispatch message with DispatchMessageW
            if message.message == WM_HOTKEY {
                on_hot_key();
            }
            return true;
        } else {
            return false;
        }
    }
}

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
        let window = Desktop::get_actual_desktop(g_desktop);
        let (width, height) = Desktop::get_window_dimensions(g_desktop);
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

    fn tray_focused(&self) -> bool {
        unsafe {
            let fg: HWND = GetForegroundWindow();
            return fg == self.tray.icon_overflow
                || fg == self.tray.tray_handle
                || fg == self.tray.start_menu;
        }
    }
    pub unsafe fn get_actual_desktop(g_desktop: HWND) -> HWND {
        let sibling = FindWindowExW(
            g_desktop,
            null_mut(),
            win32_string("OleDdeWndClass").as_ptr(),
            null_mut(),
        );
        let window = FindWindowExW(
            g_desktop,
            sibling,
            win32_string("WorkerW").as_ptr(),
            null_mut(),
        );
        return window;
    }
    unsafe fn open_start_menu(&self) {
        keybd_event(VK_LWIN as u8, 0, 0, 0);
        keybd_event(VK_LWIN as u8, 0, KEYEVENTF_KEYUP, 0)
    }
    unsafe fn is_menu_open(&self) -> bool {
        let (_, b, _, _) = Desktop::get_window_pos(self.tray.tray_handle);
        return b == self.height;
    }
    unsafe fn debug_cur_window(&self) {
        let current = GetForegroundWindow();
        let class_name = CString::new(" ".repeat(255)).unwrap().into_raw();
        //let class_name = win32_string(" ".repeat(255));
        let out = GetClassNameA(current, class_name, 255);

        let sstr = CString::from_raw(class_name);
        let (w, h) = Desktop::get_window_dimensions(current);
        println!(
            "className: '{}' {}, {}x{}",
            sstr.to_str().unwrap(),
            out,
            w,
            h
        );
    }
    unsafe fn full_screen_program(&self) -> bool {
        let current = GetForegroundWindow();
        if current == self.window {
            return false;
        };
        let (t, b, l, r) = Desktop::get_window_pos(current);
        return t == 0 && l == 0 && b == self.height && r == self.width;
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
    pub unsafe fn get_window_dimensions(handle: HWND) -> (i32, i32) {
        let (top, bottom, left, right) = Desktop::get_window_pos(handle);
        return (right - left, bottom - top);
    }

}
pub struct Context {
    pub code: i32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub result: LRESULT,
}
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
fn mouse_move(x: i32, y: i32) {
    unsafe {
        if !desktop.enabled {
            return ();
        }
        //desktop.debug_cur_window();
        if !desktop.tray.showing {
            if desktop.is_bottom_left(x, y) && !desktop.full_screen_program() {
                desktop.tray.showing = true;
                desktop.open_start_menu();
                desktop.tray.show();
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
    let mut window = create_window("win_gnome", "Gnome for Windows").unwrap();
    unsafe {
        desktop = Desktop::new();
        desktop.tray.hide();
        RegisterHotKey(window.handle, 0, MOD_WIN as u32, VK_ESCAPE as u32);
    }

    let hotkey_callback = || unsafe {
        if !desktop.toggle() {
            desktop.tray.show();
        } else {
            desktop.tray.hide();
        }
    };

    let m_hook = mouse_hook().unwrap();

    loop {
        if !handle_message(&mut window, &hotkey_callback) {
            break;
        }
    }
}