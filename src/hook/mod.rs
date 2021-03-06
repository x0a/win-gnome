/*! Adapted from: https://github.com/CasualX/external/tree/master/src/hook
Windows hooks.

The most important thing to know is that the callbacks are context-less.

You cannot pass a `self` of any kind to communicate to the outside world, the only way to get information out is through global mutable state. This is an API design limitation of `SetWindowsHookEx` itself.

# Examples

Use the provided `windows_hook!` to create the hooks instead of implementing the traits manually.

This complexity is needed due to the lack of context pointer making the `Fn*` traits not usable.

```
# #[macro_use] extern crate external; fn main() {
windows_hook! {
	/// A function with the given name which takes no arguments is created.
	///
	/// This function registers the hook and returns the registration result.
	///
	/// Doc comments, other attributes and optional `pub` will be applied to this function.
	pub fn my_callback(context: &mut KeyboardLL) {
		// The callback type is defined by the argument identifier:
		//
		// * `KeyboardLL` means this is a low level keyboard hook.
		// * `MouseLL` means this is a low level mouse hook.
		//
		// It is not necessary to use any items from this module, the macro will resolve them for you.
		println!("{:?}", context);
	}
}
# }
```

Generates the following code:

```
/// A function with the given name which takes no arguments is created.
///
/// This function registers the hook and returns the registration result.
///
/// Doc comments, other attributes and optional `pub` will be applied to this function.
pub fn my_callback() -> Result<external::hook::Hook, external::error::ErrorCode> {
	enum T {}
	impl external::hook::Invoke for T {
		unsafe fn invoke(context: &mut external::hook::Context) {
			<T as external::hook::CallKeyboardLL>::callback(std::mem::transmute(context));
		}
	}
	impl external::hook::CallKeyboardLL for T {
		fn callback(context: &mut external::hook::KeyboardLL) {
			println!("{:?}", context);
		}
	}
	impl T {
		/// Registers the hook.
		pub fn register() -> Result<external::hook::Hook, external::error::ErrorCode> {
			<T as external::hook::CallKeyboardLL>::register()
		}
	}
	T::register()
}
```

Register the hook by simply calling the defined function and unwrapping it.
!*/

use crate::winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, WPARAM};
use crate::winapi::shared::windef::{HHOOK, HWINEVENTHOOK, HWND};
use crate::winapi::um::winuser::{CallNextHookEx, UnhookWinEvent, UnhookWindowsHookEx};
use std::{panic, ptr};

/// Raw context for hook callbacks.
///
/// You will not need to use this directly.
#[allow(non_snake_case)]
pub struct Context {
	pub code: i32,
	pub wParam: WPARAM,
	pub lParam: LPARAM,
	pub result: LRESULT,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct WinContext {
	dwEvent: u32,
	dwEventThread: u32,
	dwmsEventTime: u32,
	idObject: i32,
	idChild: i32,
	hWinEventHook: u64,
	hwnd: u64,
}

/// Thunks the system's `HOOKPROC`.
pub trait Invoke {
	/// Cast the raw context to something more sensible for the specific hook type and invoke the real callback handler.
	unsafe fn invoke(arg: &mut Context);

	#[allow(non_snake_case)]
	#[doc(hidden)]
	unsafe extern "system" fn thunk(code: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
		if code >= 0 {
			let result = panic::catch_unwind(|| {
				let mut context = Context {
					code: code,
					wParam: wParam,
					lParam: lParam,
					result: 0,
				};
				Self::invoke(&mut context);
				context
			});
			if let Ok(context) = result {
				return if context.result != 0 {
					context.result
				} else {
					CallNextHookEx(
						ptr::null_mut(),
						context.code,
						context.wParam,
						context.lParam,
					)
				};
			}
		}
		CallNextHookEx(ptr::null_mut(), code, wParam, lParam)
	}

}

pub trait InvokeWin {
	unsafe fn invoke(arg: &mut WinContext);
	
	#[allow(non_snake_case)]
	unsafe extern "system" fn thunk(
		hWinEventHook: HWINEVENTHOOK,
		dwEvent: DWORD,
		hwnd: HWND,
		idObject: i32,
		idChild: i32,
		dwEventThread: DWORD,
		dwmsEventTime: DWORD,
	) {

		let mut context = WinContext {
			hWinEventHook: hWinEventHook as u64,
			hwnd: hwnd as u64,
			dwEvent,
			idObject,
			idChild,
			dwEventThread,
			dwmsEventTime,
		};
		Self::invoke(&mut context);
	}
}

/// Setup a windows hook callback.
///
/// See the [hook module](hook/index.html)'s documentation for more information.
///

#[macro_export]
macro_rules! winevent_hook {
	(@parse $prefix:tt [fn $name:ident($arg:ident: &mut FgWinEvent) $body:tt]) => {
		winevent_hook!(@emit $prefix CallFg [fn $name($arg: &mut FgWinEvent) $body]);
	};
	(@parse $prefix:tt [fn $name:ident($arg:ident: &mut MouseCaptureEvent) $body:tt]) => {
		winevent_hook!(@emit $prefix CallCapture [fn $name($arg: &mut MouseCaptureEvent) $body]);
	};
	(@parse $prefix:tt [fn $($tail:tt)*]) => {
		env!("Unsupported argument fn: expected `&mut KeyboardLL` or `&mut MouseLL`. Check spelling?");
	};
	(@parse $prefix:tt [type $name:ident($arg:ident: &mut FgWinEvent) $body:tt]) => {
		winevent_hook!(@emit $prefix CallFg [type $name($arg: &mut FgWinEvent) $body]);
	};
	(@parse $prefix:tt [type $name:ident($arg:ident: &mut MouseCaptureEvent) $body:tt]) => {
		winevent_hook!(@emit $prefix CallCapture [type $name($arg: &mut MouseCaptureEvent) $body]);
	};

	(@parse [$($prefix:tt)*] [$head:tt $($tail:tt)*]) => {
		winevent_hook!(@parse [$($prefix)* $head] [$($tail)*]);
	};
	// Emits the given name as a registration function.
	(@emit [$($prefix:tt)*] $call:ident [fn $name:ident($arg:ident: &mut $ty:ident) $body:tt]) => {
		$($prefix)*
		fn $name() -> Result<$crate::hook::WinHook, $crate::errors::ErrorCode> {
			winevent_hook!(@emit [] $call [type T($arg: &mut $ty) $body]);
			T::register()
		}
	};

	// Emits the given name as a registration type.
	(@emit [$($prefix:tt)*] $call:ident [type $name:ident($arg:ident: &mut $ty:ident) $body:tt]) => {
		$($prefix)*
		enum $name {}
		impl $crate::hook::InvokeWin for $name {
			unsafe fn invoke(context: &mut $crate::hook::WinContext) {
				<Self as $crate::hook::$call>::callback(std::mem::transmute(context));
			}
		}
		impl $crate::hook::$call for $name {
			fn callback($arg: &mut $crate::hook::$ty) $body
		}
		impl $name {
			/// Registers the hook.
			pub fn register() -> Result<$crate::hook::WinHook, $crate::errors::ErrorCode> {
				<$name as $crate::hook::$call>::register()
			}
		}
	};

	//----------------------------------------------------------------
	// Macro entry point

	($($tail:tt)*) => {
		winevent_hook!(@parse [] [$($tail)*]);
	};
}
macro_rules! windows_hook {
	//----------------------------------------------------------------
	// Parser rules

	// Match the `KeyboardLL` callback fn
	(@parse $prefix:tt [fn $name:ident($arg:ident: &mut KeyboardLL) $body:tt]) => {
		windows_hook!(@emit $prefix CallKeyboardLL [fn $name($arg: &mut KeyboardLL) $body]);
	};
	// Match the `MouseLL` callback fn
	(@parse $prefix:tt [fn $name:ident($arg:ident: &mut MouseLL) $body:tt]) => {
		windows_hook!(@emit $prefix CallMouseLL [fn $name($arg: &mut MouseLL) $body]);
	};

	// Match every other callback fn
	(@parse $prefix:tt [fn $($tail:tt)*]) => {
		//env!("Unsupported argument fn: expected `&mut KeyboardLL` or `&mut MouseLL`. Check spelling?");
	};

	// Match the `KeyboardLL` callback type
	(@parse $prefix:tt [type $name:ident($arg:ident: &mut KeyboardLL) $body:tt]) => {
		windows_hook!(@emit $prefix CallKeyboardLL [type $name($arg: &mut KeyboardLL) $body]);
	};
	// Match the `MouseLL` callback type
	(@parse $prefix:tt [type $name:ident($arg:ident: &mut MouseLL) $body:tt]) => {
		windows_hook!(@emit $prefix CallMouseLL [type $name($arg: &mut MouseLL) $body]);
	};
	// Match every other callback type
	(@parse $prefix:tt [type $($tail:tt)*]) => {
		env!("Unsupported argument type: expected `&mut KeyboardLL` or `&mut MouseLL`. Check spelling?");
	};

	// TT muncher pealing off prefixes
	(@parse [$($prefix:tt)*] [$head:tt $($tail:tt)*]) => {
		windows_hook!(@parse [$($prefix)* $head] [$($tail)*]);
	};
	// Catches the case where no `fn` or `type` token is found
	(@parse $prefix:tt []) => {
		env!("Invalid syntax: expected an `fn` or `type` token.");
	};

	//----------------------------------------------------------------
	// Emits the finalized code

	// Emits the given name as a registration function.
	(@emit [$($prefix:tt)*] $call:ident [fn $name:ident($arg:ident: &mut $ty:ident) $body:tt]) => {
		$($prefix)*
		fn $name() -> Result<$crate::hook::Hook, $crate::errors::ErrorCode> {
			windows_hook!(@emit [] $call [type T($arg: &mut $ty) $body]);
			T::register()
		}
	};

	// Emits the given name as a registration type.
	(@emit [$($prefix:tt)*] $call:ident [type $name:ident($arg:ident: &mut $ty:ident) $body:tt]) => {
		$($prefix)*
		enum $name {}
		impl $crate::hook::Invoke for $name {
			unsafe fn invoke(context: &mut $crate::hook::Context) {
				<Self as $crate::hook::$call>::callback(std::mem::transmute(context));
			}
		}
		impl $crate::hook::$call for $name {
			fn callback($arg: &mut $crate::hook::$ty) $body
		}
		impl $name {
			/// Registers the hook.
			pub fn register() -> Result<$crate::hook::Hook, $crate::errors::ErrorCode> {
				<$name as $crate::hook::$call>::register()
			}
		}
	};

	//----------------------------------------------------------------
	// Macro entry point

	($($tail:tt)*) => {
		windows_hook!(@parse [] [$($tail)*]);
	};
}

/// The hook registration, unhooked when this goes out of scope.
pub struct Hook(HHOOK);
impl Drop for Hook {
	fn drop(&mut self) {
		unsafe {
			UnhookWindowsHookEx(self.0);
		}
	}
}
pub struct WinHook(HWINEVENTHOOK);
impl Drop for WinHook {
	fn drop(&mut self) {
		unsafe {
			UnhookWinEvent(self.0);
		}
	}
}

mod mouse_ll;
pub use self::mouse_ll::*;
mod fg_changed;
pub use self::fg_changed::*;
mod capture_mouse;
pub use self::capture_mouse::*;