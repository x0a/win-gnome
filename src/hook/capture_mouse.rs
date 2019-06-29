
use std::ptr::null_mut;
use super::{WinContext, InvokeWin, WinHook};
use winapi::um::winuser::{
	SetWinEventHook, 
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, EVENT_SYSTEM_CAPTURESTART, EVENT_SYSTEM_CAPTUREEND
};
use winapi::shared::windef::HWND;
use crate::errors::ErrorCode;

pub struct MouseCaptureEvent(WinContext);

impl MouseCaptureEvent{
    pub fn has_focus(&self) -> bool{
        return self.0.dwEvent == EVENT_SYSTEM_CAPTURESTART
    }
    pub fn get_hwnd(&self) -> HWND{
        return self.0.hwnd as HWND
    }
}

pub trait CallCapture: InvokeWin{
    fn callback(arg: &mut MouseCaptureEvent);
    fn register() -> Result<WinHook, ErrorCode>{
        unsafe {
            let hook = SetWinEventHook(
                EVENT_SYSTEM_CAPTURESTART,
                EVENT_SYSTEM_CAPTUREEND,
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