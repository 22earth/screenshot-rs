use crate::dwmapi;
use crate::user;
use crate::window_info::WindowInfo;
use bindings::windows::win32 as win32;

pub fn enumerate_capturable_windows() -> Box<Vec<WindowInfo>> {
    unsafe {
        let windows = Box::into_raw(Box::new(Vec::<WindowInfo>::new()));
        win32::EnumWindows(Some(enum_window), windows as isize);
        Box::from_raw(windows)
    }
}

extern "system" fn enum_window(window: isize, state: isize) -> i32 {
    unsafe {
        let state = Box::leak(Box::from_raw(state as *mut Vec<WindowInfo>));

        let window_info = WindowInfo::new(window);
        if window_info.is_capturable_window() {
            state.push(window_info);
        }
    }
    1
}

pub trait CaptureWindowCandidate {
    fn is_capturable_window(&self) -> bool;
}

impl CaptureWindowCandidate for WindowInfo {
    fn is_capturable_window(&self) -> bool {
        unsafe {
            if self.title.is_empty()
                || self.handle == win32::GetShellWindow()
                || win32::IsWindowVisible(self.handle) == 0
                || win32::GetAncestor(self.handle, user::GA_ROOT) != self.handle
            {
                return false;
            }

            let style = win32::GetWindowLongW(self.handle, user::GWL_STYLE);
            if style & user::WS_DISABLED == 1 {
                return false;
            }

            // No tooltips
            let ex_style = win32::GetWindowLongW(self.handle, user::GWL_EXSTYLE);
            if ex_style & user::WS_EX_TOOLWINDOW == 1 {
                return false;
            }

            // Check to see if the self is cloaked if it's a UWP
            if self.class_name == "Windows.UI.Core.CoreWindow"
                || self.class_name == "ApplicationFrameWindow"
            {
                let mut cloaked: u32 = 0;
                if dwmapi::DwmGetWindowAttribute(
                    self.handle,
                    dwmapi::DWMWA_CLOAKED,
                    &mut cloaked as *mut _ as *mut _,
                    std::mem::size_of::<u32>() as u32,
                ) == 0
                    && cloaked == dwmapi::DWM_CLOAKED_SHELL
                {
                    return false;
                }
            }

            // Unfortunate work-around. Not sure how to avoid this.
            if is_known_blocked_window(self) {
                return false;
            }
        }
        true
    }
}

fn is_known_blocked_window(window_info: &WindowInfo) -> bool {
    // Task View
    window_info.matches_title_and_class_name("Task View", "Windows.UI.Core.CoreWindow") ||
    // XAML Islands
    window_info.matches_title_and_class_name("DesktopWindowXamlSource", "Windows.UI.Core.CoreWindow") ||
    // XAML Popups
    window_info.matches_title_and_class_name("PopupHost", "Xaml_WindowedPopupClass")
}
