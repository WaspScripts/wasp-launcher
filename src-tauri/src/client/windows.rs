use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetClassNameW, GetWindowRect, GetWindowThreadProcessId,
};

struct WindowMatch {
    pid: u32,
    hwnd: HWND,
}

struct EnumContext {
    target_pid: u32,
    matches: Vec<WindowMatch>,
}

pub fn list_processes() -> windows::core::Result<()> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let mut context = EnumContext {
                    target_pid: entry.th32ProcessID,
                    matches: Vec::new(),
                };

                let _ = EnumWindows(
                    Some(enum_window_callback),
                    LPARAM(&mut context as *mut EnumContext as isize),
                );

                for window in context.matches {
                    println!("MATCH -> PID: {} | HWND: {:?}", window.pid, window.hwnd);
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        CloseHandle(snapshot)?;
        Ok(())
    }
}

extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut EnumContext);
        let mut process_id = 0;

        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == context.target_pid {
            // 1. Check if the top-level window itself is the canvas
            check_and_add_if_match(hwnd, context);

            // 2. Scan all children of this window
            let _ = EnumChildWindows(
                Some(hwnd),
                Some(enum_child_callback),
                LPARAM(context as *mut EnumContext as isize),
            );
        }
    }
    BOOL(1)
}

extern "system" fn enum_child_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut EnumContext);
        check_and_add_if_match(hwnd, context);
    }
    BOOL(1)
}

unsafe fn check_and_add_if_match(hwnd: HWND, context: &mut EnumContext) {
    let mut class_name = [0u16; 256];
    let len = GetClassNameW(hwnd, &mut class_name);

    if len == 0 {
        return;
    }

    let name = String::from_utf16_lossy(&class_name[..len as usize]);
    if name != "SunAwtCanvas" {
        return;
    }

    println!("{:?}", hwnd);
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return;
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    if width <= 100 || height <= 100 {
        return;
    }

    context.matches.push(WindowMatch {
        pid: context.target_pid,
        hwnd,
    });
}
