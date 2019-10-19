/*
    Adapted from: https://github.com/CasualX/external/tree/master/src/hook
*/

use std::ptr::null_mut;
use super::{WinContext, InvokeWin, WinHook};
use winapi::um::winuser::{
	SetWinEventHook, 
    EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS
};
use winapi::shared::windef::HWND;
use crate::errors::ErrorCode;

pub struct FgWinEvent(WinContext);

impl FgWinEvent {
    pub fn get_hwnd(&self) -> HWND {
        return self.0.hwnd as HWND;
    }
}


pub trait CallFg: InvokeWin {
    fn callback(arg: &mut FgWinEvent);
    /// Registers the low-level mouse hook.
    fn register() -> Result<WinHook, ErrorCode> {
        unsafe {
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                null_mut(),
                Some(Self::thunk),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );
            if hook.is_null() {
                Err(ErrorCode::last())
            } else {
                Ok(WinHook(hook))
            }
        }
    }
}