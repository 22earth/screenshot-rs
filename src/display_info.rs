const CCHDEVICENAME: usize = 32;
use bindings::windows::win32::menu_rc::{MONITORINFO, GetMonitorInfoW, EnumDisplayMonitors};
use bindings::windows::win32::backup::RECT;

#[derive(Clone)]
pub struct DisplayInfo {
    pub handle: isize,
    pub display_name: String,
}

impl DisplayInfo {
    pub fn new(monitor_handle: isize) -> Self {
        #[repr(C)]
        struct MonitorInfoExW {
            _base: MONITORINFO,
            sz_device: [u16; CCHDEVICENAME],
        }

        let mut info = MonitorInfoExW {
            _base: MONITORINFO {
                cbSize: std::mem::size_of::<MonitorInfoExW>() as u32,
                rcMonitor: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                rcWork: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
                dwFlags: 0,
            },
            sz_device: [0u16; CCHDEVICENAME],
        };

        unsafe {
            let result = GetMonitorInfoW(monitor_handle, &mut info as *mut _ as *mut _);
            if result == 0 {
                panic!("GetMonitorInfoW failed!");
            }
        }

        let display_name = String::from_utf16_lossy(&info.sz_device)
            .trim_matches(char::from(0))
            .to_string();

        Self {
            handle: monitor_handle,
            display_name,
        }
    }
}

pub fn enumerate_displays() -> Box<Vec<DisplayInfo>> {
    unsafe {
        let displays = Box::into_raw(Box::new(Vec::<DisplayInfo>::new()));
        EnumDisplayMonitors(0, std::ptr::null_mut(), Some(enum_monitor), displays as isize);
        Box::from_raw(displays)
    }
}

extern "system" fn enum_monitor(
    monitor: isize,
    _: isize,
    _: *mut RECT,
    state: isize,
) -> i32 {
    unsafe {
        let state = Box::leak(Box::from_raw(state as *mut Vec<DisplayInfo>));

        let display_info = DisplayInfo::new(monitor);
        state.push(display_info);
    }
    1
}
