#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate winapi;
pub mod errors;
#[macro_use]
mod hook;
mod desktop;
mod window;

use std::alloc::System;
use std::ptr::null_mut;
use std::thread;
use std::time::Duration;

#[global_allocator]
static A: System = System;

use winapi::um::winuser::{
    MessageBoxW, RegisterHotKey, MB_ICONEXCLAMATION, MB_OK, MOD_WIN, VK_ESCAPE, WM_MOUSEMOVE
};
use desktop::Desktop;
use window::win32_string;

enum CornerAction {
    StartMenu,
    DesktopSelector,
}

#[derive(PartialEq)]
enum TrayAction {
    Hide,
    NoHide
}

const IDENTIFIER: &str = "win_gnome";
pub static mut SENSITIVITY: i32 = 100; // %
static mut DELAY: bool = false;
static mut LASTX: i32 = 0;
static mut LASTY: i32 = 0;
static mut CORNER_ACTION: CornerAction = CornerAction::StartMenu;
static mut TRAY_ACTION: TrayAction = TrayAction::Hide;
#[allow(non_upper_case_globals)]
pub static mut desktop: Desktop = Desktop::default();

fn delay_next(ms: u64) {
    unsafe {
        DELAY = true;
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            DELAY = false;
            mouse_move(LASTX, LASTY);
        });
    }
}

unsafe fn on_hot_corner() {
    if !desktop.full_screen_program() {
        desktop.hot_active = false;
        
        if TRAY_ACTION == TrayAction::Hide {
            desktop.tray.show();
        }
        match CORNER_ACTION {
            CornerAction::StartMenu => desktop.open_start_menu(),
            CornerAction::DesktopSelector => desktop.open_desktop_selector(),
        }
        delay_next(300);
    } else if desktop.shell_changed() { // full screen program && that full screen program might be new shell
        println!("Desktop handle was invalid. Got new one and trying again");
        on_hot_corner();
    }
}
unsafe fn on_leaving_corner(force: bool){
    if TRAY_ACTION == TrayAction::Hide {
        if force || !desktop.tray.is_tray_open() {
            if !desktop.tray.hide() && desktop.shell_changed() {
                println!("Desktop handle was invalid. Got new one and trying again");
                on_leaving_corner(force);
            } else {
                desktop.hot_active = true;
            }
        }
    } else {
        desktop.hot_active = true;
    }
}
unsafe fn mouse_move(x: i32, y: i32) {
    LASTX = x;
    LASTY = y;

    if DELAY {
        return ();
    }

    if desktop.hot_active {
        if desktop.tray.is_hot_corner(x, y) {
            on_hot_corner();
        }
    } else {
        if desktop.last_window != desktop.tray.start_menu && !desktop.tray.is_tray_region(x, y) {
            on_leaving_corner(false);
        }
    }
}

windows_hook! {
    pub fn mouse_hook(context: &mut MouseLL) {
        if  unsafe { !desktop.enabled } {
            return ();
        }

        if context.message() == WM_MOUSEMOVE {
            unsafe { mouse_move(context.pt_x(), context.pt_y()) };
        }
    }
}
winevent_hook! {
    pub fn fg_hook(context: &mut FgWinEvent) {
        let hwnd = context.get_hwnd();
        
        unsafe{
            if desktop.foreground_changed(hwnd) && !desktop.tray.is_tray_region(LASTX, LASTY) {
                on_leaving_corner(true);
            } 
            #[cfg(debug_assertions)]
            desktop._debug_window(hwnd)
        };
    }
}

fn get_sensitivity(value: &str) -> Result<i32, &'static str>{
    match value.parse::<i32>(){
        Ok(sensitivity) => if sensitivity > 0 && sensitivity <= 100 {
            Ok(sensitivity)
        } else {
            Err("Sensitivity must be between 1-100")
        },
        Err(_) => Err("Expected sensitivity=X, where X is 1-100")
    }
}

fn get_property(argument: String) -> (String, String) {
    let mut parts = argument.split("=");
    (
        match parts.next() {
            Some(arg) => arg.to_string(),
            None => "".to_string()
        },
        match parts.next() {
            Some(arg) => arg.to_string(),
            None => "".to_string()
        }
    )
}

fn main() {
    for (index, (prop, value)) in std::env::args().map(|arg| get_property(arg)).enumerate() {
        match (index, &prop[..], &value[..]) {
            (index, _, _) if index == 0 => continue,
            (_, "--selector", _) => unsafe { CORNER_ACTION = CornerAction::DesktopSelector },
            (_, "--no-hide", _) => unsafe { TRAY_ACTION = TrayAction::NoHide },
            (_, "--sensitivity", sensitivity) => match get_sensitivity(sensitivity){
                Ok(sensitivity) => unsafe { SENSITIVITY = sensitivity },
                Err(error) => {
                    println!("Invalid sensitivity: {}", error);
                    return ();
                }
            },
            (_, "--help", _) => {
                println!("WinGnome 0.1");
                println!(
                    "\t--selector\tOpens Desktop selector on hot corner as opposed to opening menu\n\
                     \t--sensitivity=X\tSpecifies size of hot corner as percent of start button, must be between 1-100\n\
                     \t--no-hide\tDon't hide tray"
                );
                return ();
            },
            _ => {
                println!("Invalid argument \"{}\": Use --help for a list of parameters.", prop);
                return ();
            }
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

        let _window = window::create_hidden_window(IDENTIFIER).unwrap();
        desktop = loop {
            match Desktop::new() {
                Ok(next_desktop) => break next_desktop,
                Err(error) => {
                    println!("Could not fetch desktop: {}\nTrying again in 1 second...", error);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        };

        let hotkey_callback = || {
            let on = !desktop.toggle();
            if TRAY_ACTION == TrayAction::Hide {
                if on {
                    desktop.tray.show();
                } else {
                    desktop.tray.hide();
                }
            }
            true
        };

        let close_callback = || {
            desktop.enabled = false;
            desktop.tray.show();
            false
        };

        let _mhook = mouse_hook()
            .expect("Unable to install system-wide mouse hook");
        let _fhook = fg_hook()
            .expect("Unable to install system-side foreground hook");
        RegisterHotKey(_window.handle, 0, MOD_WIN as u32, VK_ESCAPE as u32);

        if TRAY_ACTION == TrayAction::Hide {
            desktop.tray.hide();
        }

        loop {
            if !window::handle_message(&_window, &hotkey_callback, &close_callback) {
                break;
            }
        }
    }

}