use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetAncestor, GetClassNameW, GetWindowRect,
    GetWindowThreadProcessId, IsIconic, SetForegroundWindow, ShowWindow, GA_ROOT, SW_RESTORE,
    SW_SHOW,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowMatch {
    pid: u32,
    pub(crate) hwnd: isize,
    name: String,
}

struct EnumContext {
    names: HashMap<u32, String>,
    pid: u32,
    matches: Vec<WindowMatch>,
}

pub fn list_processes() -> Result<Vec<WindowMatch>, String> {
    let mut context = EnumContext {
        names: process_names()?,
        pid: 0,
        matches: Vec::new(),
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_window_callback),
            LPARAM(&mut context as *mut EnumContext as isize),
        );
    }

    Ok(context.matches)
}

fn process_names() -> Result<HashMap<u32, String>, String> {
    let mut names = HashMap::new();

    unsafe {
        let snapshot =
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| e.to_string())?;

        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                names.insert(entry.th32ProcessID, string_from_u16_slice(&entry.szExeFile));

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    Ok(names)
}

extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut EnumContext);
        let mut process_id = 0;

        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if !context.names.contains_key(&process_id)
            || context.matches.iter().any(|m| m.pid == process_id)
        {
            return BOOL(1);
        }

        context.pid = process_id;
        if !check_and_add_if_match(hwnd, context) {
            let _ = EnumChildWindows(Some(hwnd), Some(enum_child_callback), lparam);
        }
    }
    BOOL(1)
}

extern "system" fn enum_child_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let context = &mut *(lparam.0 as *mut EnumContext);
        if check_and_add_if_match(hwnd, context) {
            return BOOL(0);
        }
    }
    BOOL(1)
}

unsafe fn check_and_add_if_match(hwnd: HWND, context: &mut EnumContext) -> bool {
    let mut class_name = [0u16; 256];
    let len = GetClassNameW(hwnd, &mut class_name);
    if len == 0 {
        return false;
    }

    let name = String::from_utf16_lossy(&class_name[..len as usize]);
    if name != "SunAwtCanvas" {
        return false;
    }

    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return false;
    }

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    if width <= 100 || height <= 100 {
        return false;
    }

    context.matches.push(WindowMatch {
        pid: context.pid,
        hwnd: hwnd.0 as isize,
        name: context.names[&context.pid].clone(),
    });

    true
}

fn string_from_u16_slice(slice: &[u16]) -> String {
    let len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    String::from_utf16_lossy(&slice[..len])
}

pub fn bring_window_to_top(handle: isize) -> bool {
    let hwnd_child = HWND(handle as *mut core::ffi::c_void);

    unsafe {
        let hwnd_root = GetAncestor(hwnd_child, GA_ROOT);
        if hwnd_root.0.is_null() {
            return false;
        }

        if IsIconic(hwnd_root).as_bool() {
            let _ = ShowWindow(hwnd_root, SW_RESTORE);
        } else {
            let _ = ShowWindow(hwnd_root, SW_SHOW);
        }

        SetForegroundWindow(hwnd_root).as_bool()
    }
}
