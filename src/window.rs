
use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use winapi::shared::windef::HWND;
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW,TranslateMessage,GetMessageW,RegisterClassW,
    CW_USEDEFAULT, WM_HOTKEY, WM_CLOSE, WNDCLASSW, MSG
};

#[cfg(windows)]
pub fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

// Window struct
#[cfg(windows)]
pub struct Window {
    pub handle: HWND,
}

// Create window function
#[cfg(windows)]
pub fn create_hidden_window(identifier: &str) -> Result<Window, Error> {
    let name = win32_string(identifier);
    let title = win32_string(identifier);

    unsafe {
        // Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
        let hinstance = GetModuleHandleW(null_mut());

        // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
        let wnd_class = WNDCLASSW {
            style: 0, // Style
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
pub fn handle_message(window: &Window, on_hot_key: impl Fn() -> bool, on_close: impl Fn() -> bool) -> bool {
    unsafe {
        let mut message: MSG = mem::uninitialized();

        // Get message from message queue with GetMessageW
        if GetMessageW(&mut message as *mut MSG, window.handle, 0, 0) > 0 {
            TranslateMessage(&message as *const MSG); // Translate message into something meaningful with TranslateMessage
            DispatchMessageW(&message as *const MSG); // Dispatch message with DispatchMessageW
            if message.message == WM_HOTKEY {
                return on_hot_key();
            } else if message.message == WM_CLOSE {
                return on_close();
            }
            return true;
        } else {
            return false;
        }
    }
}
pub fn previous_instance(identifier: &str) -> bool {
    unsafe {
        CreateMutexW(null_mut(), 1, win32_string(identifier).as_ptr());
        if GetLastError() == ERROR_ALREADY_EXISTS {
            true
        } else {
            false
        }
    }
}