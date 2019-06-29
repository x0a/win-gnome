/*!
Low level mouse hook details.
!*/
use crate::errors::ErrorCode;
use super::{Context, Hook, Invoke};
use crate::winapi::shared::minwindef::{UINT};
use crate::winapi::um::winuser::SetWindowsHookExW;
use crate::winapi::um::winuser::{MSLLHOOKSTRUCT, WH_MOUSE_LL};
use std::ptr::null_mut;

//----------------------------------------------------------------

/// Low level mouse hook callback context.
///
/// See documentation for
/// [LowLevelMouseProc](https://msdn.microsoft.com/en-us/library/windows/desktop/ms644986.aspx)
/// and
/// [MSLLHOOKSTRUCT](https://msdn.microsoft.com/en-us/library/windows/desktop/ms644970(v=vs.85).aspx)
/// for more information.
#[repr(C)]
pub struct MouseLL(Context);
impl MouseLL {
	pub fn message(&self) -> UINT {
		self.0.wParam as UINT
	}

	fn info(&self) -> &MSLLHOOKSTRUCT {
		unsafe { &*(self.0.lParam as *const MSLLHOOKSTRUCT) }
	}

	pub fn pt_x(&self) -> i32 {
		self.info().pt.x
	}
	pub fn pt_y(&self) -> i32 {
		self.info().pt.y
	}
}

/// Low level mouse hook callback.
pub trait CallMouseLL: Invoke {
	fn callback(arg: &mut MouseLL);
	/// Registers the low-level mouse hook.
	fn register() -> Result<Hook, ErrorCode> {
		unsafe {
			let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(Self::thunk), null_mut(), 0);
			if hook.is_null() {
				Err(ErrorCode::last())
			} else {
				Ok(Hook(hook))
			}
		}
	}
}
