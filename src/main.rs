#![windows_subsystem = "windows"]
extern crate winapi;

use std::ptr::null_mut;
use std::thread;
use std::time::Duration;

use winapi::um::winuser::{
    MessageBoxW, RegisterHotKey, MB_ICONEXCLAMATION, MB_OK, MOD_WIN, VK_ESCAPE, WM_MOUSEMOVE,
};
mod desktop;
use desktop::{Desktop, Tray, TrayOrientation};
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
    StartMenu,
    DesktopSelector,
}

#[allow(non_upper_case_globals)]
static mut corner_action: CornerAction = CornerAction::StartMenu;
#[allow(non_upper_case_globals)]
static mut desktop: Desktop = Desktop {
    height: 0,
    width: 0,
    enabled: true,
    last_window: null_mut(),
    shell_window: null_mut(),
    shell_parent: null_mut(),
    tray: Tray{
        orientation: TrayOrientation::Bottom,
        bar: null_mut(),
        start_button: null_mut(),
        start_menu: null_mut(),
        icon_overflow: null_mut(),
        parent_width: 0,
        parent_height: 0,
        start_width: 0,
        start_height: 0,
        showing: true,
        overflow_showing: false,
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
        // desktop._debug_window(desktop.tray.icon_overflow);
        if !desktop.tray.showing {
            if desktop.tray.is_hot_corner(x, y) {
                if !desktop.full_screen_program() {
                    desktop.tray.show();
                    match corner_action {
                        CornerAction::StartMenu => desktop.open_start_menu(),
                        CornerAction::DesktopSelector => desktop.open_desktop_selector(),
                    }
                    delay_next(300);
                } else if desktop.update_desktop() {
                    println!("Desktop handle was invalid. Got new one and trying again");
                    mouse_move(x, y); // try again
                }
            }
        } else {
            if !desktop.tray.overflow_showing 
                && !desktop.tray.is_tray_region(x, y) 
                && !desktop.tray.is_tray_open() {
                if !desktop.tray.hide(){
                    if desktop.update_desktop() {
                        println!("Desktop handle was invalid. Got new one and trying again");
                    }
                }
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
            desktop.foreground_changed(hwnd);
            desktop._debug_window(hwnd)
        };
    }
}
fn main() {
    for (i, arg) in std::env::args().enumerate() {
        if i == 0 {
            continue;
        }
        if arg == "--selector" {
            unsafe {
                corner_action = CornerAction::DesktopSelector;
            }
        } else if arg == "--help" {
            println!("WinGnome 0.1");
            println!(
                "\t--selector\tOpens Desktop selector on hot corner as opposed to opening menu"
            );
            return ();
        } else {
            println!(
                "Invalid argument \"{}\". Use --help for a list of parameters.",
                arg
            );
            return ();
        }
    }
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