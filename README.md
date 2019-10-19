# win-gnome
Hotcorner + hides taskbar when not in use. Written in Rust. Replicates gnome-shell behavior for Windows 10, but with a few differences:
* By default, the hot corner opens the **start menu**, not the window selector. This is because my preference is for the search, as opposed to window selection. To use the window selector instead, run with the `--selector` argument
* Hides taskbar when not in focus, and is meant to be used with auto-hide. The hot corner is not active while the taskbar is open.

# Installation & Removal
1. Download `win-gnomev1.0.zip` at https://github.com/x0a/win-gnome/releases/tag/v1.0
2. Extract to directory
3. Run install.ps1 with powershell

Uninstallation can be done with `uninstall.ps1`



# Best use

## Taskbar Settings

![Right-click Taskbar -> Taskbar Settings -> Automatically hide the taskbar in desktop mode](/shared/taskbar_settings.png?raw=true "Right-click Taskbar -> Taskbar Settings -> Automatically hide the taskbar in desktop mode")

Right-click Taskbar -> Taskbar Settings -> Automatically hide the taskbar in desktop mode

Required. Frees up display real estate, win-gnome will hide the taskbar until you activate the hot corner or open the start menu. Hot corner is not active when the taskbar or start menu is in use.

## Performance Options

![Start Menu -> \"adjust performance\" -> Uncheck \"Fade or slide menus into view\"](/shared/performance_window.png?raw=true "Start Menu -> \"adjust performance\" -> Uncheck \"Fade or slide menus into view\"")


Start Menu -> "adjust performance" -> Uncheck "Fade or slide menus into view"

Reduces delay between leaving start menu and taskbar being hidden.