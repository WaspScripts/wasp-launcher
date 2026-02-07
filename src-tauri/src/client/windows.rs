use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowThreadProcessId,
};

pub fn list_processes() -> windows::core::Result<()> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(&entry.szExeFile)
                    .trim_matches('\0')
                    .to_string();

                println!("PID: {} | Name: {}", entry.th32ProcessID, name);

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        CloseHandle(snapshot)?;
        Ok(())
    }
}

pub fn list_windows(target_pid: u32) -> windows::core::Result<()> {
    unsafe {
        EnumWindows(Some(enum_window_callback), LPARAM(target_pid as isize))?;
    }
    Ok(())
}

extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let target_pid = lparam.0 as u32;
        let mut process_id = 0;

        // Get the PID of the process that owns this window
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == target_pid {
            // Buffer for the class name (256 is usually plenty)
            let mut class_name = [0u16; 256];
            let len = GetClassNameW(hwnd, &mut class_name);

            if len > 0 {
                let name = String::from_utf16_lossy(&class_name[..len as usize]);
                println!("Window Handle: {:?} | Class Name: {}", hwnd, name);
            }
        }
    }
    BOOL(1) // Return 1 to continue enumerating other windows
}
