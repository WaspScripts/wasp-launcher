use std::{error::Error, fs};

use serde::{Deserialize, Serialize};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, Window},
    rust_connection::RustConnection,
    CURRENT_TIME,
};

// Every AWT toplevel on X11 gets a tiny focus proxy child window with this class.
const FOCUS_PROXY_CLASS: &str = "FocusProxy";
const CANVAS_PEER_NAME: &str = "sun-awt-X11-XCanvasPeer";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowMatch {
    pid: u32,
    pub(crate) hwnd: isize,
    name: String,
}

pub fn list_processes() -> Result<Vec<WindowMatch>, String> {
    find_canvases().map_err(|e| e.to_string())
}

fn find_canvases() -> Result<Vec<WindowMatch>, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;
    let net_wm_pid = intern_atom(&conn, b"_NET_WM_PID")?;

    let mut matches = Vec::new();
    walk_tree(&conn, root, net_wm_pid, &mut matches)?;
    Ok(matches)
}

fn walk_tree(
    conn: &RustConnection,
    window: Window,
    net_wm_pid: u32,
    matches: &mut Vec<WindowMatch>,
) -> Result<(), Box<dyn Error>> {
    for child in conn.query_tree(window)?.reply()?.children {
        let Some(pid) = get_pid(conn, child, net_wm_pid) else {
            // Not a client window (e.g. a window manager frame), look inside it.
            // Windows can vanish mid-walk, that shouldn't abort the whole listing.
            let _ = walk_tree(conn, child, net_wm_pid, matches);
            continue;
        };

        if matches.iter().any(|m| m.pid == pid) || !is_java_window(conn, child) {
            continue;
        }

        if let Some(canvas) = find_canvas(conn, child) {
            matches.push(WindowMatch {
                pid,
                hwnd: canvas as isize,
                name: process_name(pid),
            });
        }
    }
    Ok(())
}

fn is_java_window(conn: &RustConnection, window: Window) -> bool {
    let Ok(Ok(tree)) = conn.query_tree(window).map(|c| c.reply()) else {
        return false;
    };
    tree.children
        .iter()
        .any(|&child| get_wm_class(conn, child).as_deref() == Some(FOCUS_PROXY_CLASS))
}

// Every window of a Java app shares the same WM_CLASS, but AWT names each one after its
// peer class (WM_NAME). The game canvas is the XCanvasPeer, which is the window Simba
// targets and the one remote input needs.
fn find_canvas(conn: &RustConnection, window: Window) -> Option<Window> {
    if get_wm_name(conn, window).as_deref() == Some(CANVAS_PEER_NAME) {
        return Some(window);
    }

    let children = conn.query_tree(window).ok()?.reply().ok()?.children;
    children
        .into_iter()
        .find_map(|child| find_canvas(conn, child))
}

fn get_wm_name(conn: &RustConnection, window: Window) -> Option<String> {
    let reply = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 256)
        .ok()?
        .reply()
        .ok()?;
    Some(String::from_utf8_lossy(&reply.value).into_owned())
}

fn get_wm_class(conn: &RustConnection, window: Window) -> Option<String> {
    let reply = conn
        .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)
        .ok()?
        .reply()
        .ok()?;

    // WM_CLASS is "instance\0class\0".
    let class = reply.value.split(|&b| b == 0).nth(1)?;
    Some(String::from_utf8_lossy(class).into_owned())
}

fn get_pid(conn: &RustConnection, window: Window, net_wm_pid: u32) -> Option<u32> {
    conn.get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?
        .value32()?
        .next()
}

fn process_name(pid: u32) -> String {
    fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| pid.to_string())
}

fn intern_atom(conn: &RustConnection, name: &[u8]) -> Result<u32, Box<dyn Error>> {
    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}

pub fn bring_window_to_top(handle: isize) -> bool {
    activate_window(handle as Window).is_ok()
}

fn activate_window(window: Window) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;
    let wm_state = intern_atom(&conn, b"WM_STATE")?;
    let net_active_window = intern_atom(&conn, b"_NET_ACTIVE_WINDOW")?;

    // Walk up to the client toplevel (the one the window manager put WM_STATE on).
    let mut current = window;
    let mut toplevel = window;
    loop {
        let has_state = conn
            .get_property(false, current, wm_state, AtomEnum::ANY, 0, 0)?
            .reply()?
            .type_
            != u32::from(AtomEnum::NONE);
        if has_state {
            toplevel = current;
        }

        let parent = conn.query_tree(current)?.reply()?.parent;
        if parent == root || parent == x11rb::NONE {
            break;
        }
        current = parent;
    }

    // EWMH activation request, this also de-iconifies the window.
    let event =
        ClientMessageEvent::new(32, toplevel, net_active_window, [1, CURRENT_TIME, 0, 0, 0]);
    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )?;
    conn.flush()?;
    Ok(())
}
