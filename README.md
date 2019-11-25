# win-gnome
Hotcorner + hides taskbar when not active. Written in Rust. Replicates gnome-shell behavior for Windows 10, but with a few differences:
* By default, the hot corner opens the **start menu and taskbar**, not the window selector. This is because my preference is for the search, as opposed to the window selector. To use the window selector instead, run with the `--selector` argument
* Hides taskbar when not in focus, and is meant to be used with auto-hide. The hot corner is not active while the taskbar is open. This is because in Windows 10, the entire taskbar acts like a hotcorner; is easy to accidentally activate and steals precious display space.

# Installation & Removal
1. Download `win-gnome.zip` at https://github.com/x0a/win-gnome/releases
2. Extract to directory
3. Run install.ps1 with powershell

Uninstallation can be done with `uninstall.ps1`



# Best use

## Taskbar Settings

![Right-click Taskbar -> Taskbar Settings -> Automatically hide the taskbar in desktop mode](/shared/taskbar_settings.png?raw=true "Right-click Taskbar -> Taskbar Settings -> Automatically hide the taskbar in desktop mode")

Right-click Taskbar -> Taskbar Settings -> Automatically hide the taskbar in desktop mode

**Required**. Frees up display real estate. win-gnome will hide the taskbar until you activate the hot corner or open the start menu. Hot corner is not active when the taskbar or start menu is in use.

## Performance Options

![Start Menu -> \"adjust performance\" -> Uncheck \"Fade or slide menus into view\"](/shared/performance_window.png?raw=true "Start Menu -> \"adjust performance\" -> Uncheck \"Fade or slide menus into view\"")


Start Menu -> "adjust performance" -> Uncheck "Fade or slide menus into view"

Reduces delay between leaving the start menu and the taskbar being hidden.
