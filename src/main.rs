//#![windows_subsystem = "windows"]
extern crate winapi;

use std::ptr::null_mut;
use std::thread;
use std::time::Duration;

use winapi::um::winuser::{
    MessageBoxW, RegisterHotKey, MB_ICONEXCLAMATION, MB_OK, MOD_WIN, VK_ESCAPE, WM_MOUSEMOVE,
};
mod desktop;
use desktop::{Desktop, Tray};
mod window;
use window::win32_string;

#[macro_use]
mod hook;
pub mod errors;

const IDENTIFIER: &str = "win_gnome";
static mut DELAY: bool = false;
static mut LASTX: i32 = 0;
static mut LASTY: i32 = 0;

enum CornerAction {
    open_menu,
    open_wintab,
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

fn delay_next(ms: u64) {
    unsafe {
        DELAY = true;
        std::thread::spawn(move || {
            let dur = Duration::from_millis(ms);
            thread::sleep(dur);
            DELAY = false;
            mouse_move(LASTX, LASTY);
        });
    }
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
            if desktop.is_bottom_left(x, y) {
                if !desktop.full_screen_program() {
                    desktop.tray.show();
                    desktop.open_start_menu();
                    delay_next(300);
                } else if desktop.update_desktop() {
                    println!("Desktop handle was invalid. Got new one and trying again");
                    mouse_move(x, y); // try again
                }
            }
        } else {
            if !desktop.is_tray_region(y) && !desktop.is_tray_open() {
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
winevent_hook! {
    pub fn fg_hook(context: &mut FgWinEvent){
        let hwnd = context.get_hwnd();
        unsafe{
            desktop.last_window = hwnd;
            if hwnd == desktop.tray.start_menu{
                desktop.tray.show();
            }
            desktop._debug_window(hwnd);
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