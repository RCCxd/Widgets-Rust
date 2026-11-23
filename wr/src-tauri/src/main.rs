#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::os::windows::process::CommandExt;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager as _, WebviewUrl, WebviewWindowBuilder,
};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionMediaProperties as Props,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP};

#[derive(Serialize, Default, Clone)]
struct Payload {
    active: bool,
    key: String,
    title: String,
    artist: String,
    playing: bool,
    pos: f64,
    dur: f64,
    app: String,
    thumb: Option<String>,
}

fn pick(mgr: &Manager) -> Option<Session> {
    if let Ok(list) = mgr.GetSessions() {
        for s in list {
            if let Ok(i) = s.GetPlaybackInfo() {
                if i.PlaybackStatus().ok() == Some(Status::Playing) {
                    return Some(s);
                }
            }
        }
    }
    mgr.GetCurrentSession().ok()
}

fn thumb(p: &Props) -> Option<String> {
    let st = p.Thumbnail().ok()?.OpenReadAsync().ok()?.get().ok()?;
    let size = st.Size().ok()? as u32;
    if size == 0 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&st.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.get().ok()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(buf)))
}

fn read(mgr: &Manager, thumb_key: &mut String) -> windows::core::Result<Option<Payload>> {
    let Some(s) = pick(mgr) else { return Ok(None) };
    let p = s.TryGetMediaPropertiesAsync()?.get()?;
    let title = p.Title()?.to_string();
    let artist = p.Artist()?.to_string();
    let key = format!("{title}|{artist}");
    let mut th = None;
    if *thumb_key != key {
        th = thumb(&p);
        if th.is_some() {
            *thumb_key = key.clone();
        }
    }
    let tl = s.GetTimelineProperties()?;
    let playing = s.GetPlaybackInfo()?.PlaybackStatus()? == Status::Playing;
    let dur = (tl.EndTime()?.Duration - tl.StartTime()?.Duration) as f64 / 1e7;
    let mut pos = tl.Position()?.Duration as f64 / 1e7;
    // o Windows so informa a posicao de vez em quando: soma o tempo desde a ultima atualizacao
    let last = tl.LastUpdatedTime()?.UniversalTime as f64;
    let now = (SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
        + 11_644_473_600.0)
        * 1e7;
    let el = (now - last) / 1e7;
    if playing && last > 0.0 && el > 0.0 && el < 86_400.0 {
        pos += el;
    }
    if dur > 0.0 {
        pos = pos.min(dur);
    }
    Ok(Some(Payload {
        active: true,
        key,
        title,
        artist,
        playing,
        pos,
        dur,
        app: s.SourceAppUserModelId()?.to_string(),
        thumb: th,
    }))
}

fn media_key(vk: u8) {
    unsafe {
        keybd_event(vk, 0, KEYBD_EVENT_FLAGS(0), 0);
        keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
    }
}

#[tauri::command]
fn media_cmd(action: String) {
    thread::spawn(move || {
        let run = || -> windows::core::Result<bool> {
            let mgr = Manager::RequestAsync()?.get()?;
            let Some(s) = pick(&mgr) else { return Ok(false) };
            match action.as_str() {
                "toggle" => s.TryTogglePlayPauseAsync()?.get(),
                "next" => s.TrySkipNextAsync()?.get(),
                "prev" => s.TrySkipPreviousAsync()?.get(),
                _ => Ok(true),
            }
        };
        // se o comando direto falhar, usa as teclas de midia do Windows
        if !run().unwrap_or(false) {
            match action.as_str() {
                "toggle" => media_key(0xB3),
                "next" => media_key(0xB0),
                "prev" => media_key(0xB1),
                _ => {}
            }
        }
    });
}

fn sfile(app: &tauri::AppHandle, name: &str) -> std::path::PathBuf {
    let d = app.path().app_data_dir().unwrap_or_default();
    let _ = std::fs::create_dir_all(&d);
    d.join(name)
}
#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> String {
    std::fs::read_to_string(sfile(&app, "settings.json")).unwrap_or_else(|_| "{}".into())
}
#[tauri::command]
fn set_settings(app: tauri::AppHandle, json: String) {
    let _ = std::fs::write(sfile(&app, "settings.json"), &json);
    let _ = app.emit("settings", json);
}
#[tauri::command]
fn get_events(app: tauri::AppHandle) -> String {
    std::fs::read_to_string(sfile(&app, "events.json")).unwrap_or_else(|_| "[]".into())
}
#[tauri::command]
fn set_events(app: tauri::AppHandle, json: String) {
    let _ = std::fs::write(sfile(&app, "events.json"), &json);
    let _ = app.emit("events", json);
}
const RUN: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[tauri::command]
fn get_autostart() -> bool {
    std::process::Command::new("reg").args(["query", RUN, "/v", "Widgets"]).creation_flags(0x08000000)
        .status().map(|s| s.success()).unwrap_or(false)
}
#[tauri::command]
fn set_autostart(on: bool) {
    let mut c = std::process::Command::new("reg");
    let exe = format!("\"{}\"", std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default());
    if on {
        c.args(["add", RUN, "/v", "Widgets", "/t", "REG_SZ", "/d", exe.as_str(), "/f"]);
    } else {
        c.args(["delete", RUN, "/v", "Widgets", "/f"]);
    }
    let _ = c.creation_flags(0x08000000).status();
}
#[tauri::command]
fn media_seek(secs: f64) {
    thread::spawn(move || {
        let _ = (|| -> windows::core::Result<()> {
            let m = Manager::RequestAsync()?.get()?;
            if let Some(s) = pick(&m) {
                s.TryChangePlaybackPositionAsync((secs * 1e7) as i64)?.get()?;
            }
            Ok(())
        })();
    });
}
fn open_cfg(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Widgets — Configurações").inner_size(900.0, 620.0).center().build();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![media_cmd, media_seek, get_settings, set_settings, get_events, set_events, get_autostart, set_autostart])
        .setup(|app| {
            // 3 janelas independentes, empilhadas no canto superior direito
            let lw = match app.primary_monitor() {
                Ok(Some(m)) => m.size().width as f64 / m.scale_factor(),
                _ => 1920.0,
            };
            let x = lw - 300.0 - 12.0;
            let mut y = 12.0;
            for (label, h) in [("clock", 150.0), ("cal", 290.0), ("music", 92.0)] {
                WebviewWindowBuilder::new(app, label, WebviewUrl::default())
                    .title("Widgets")
                    .inner_size(300.0, h)
                    .position(x, y)
                    .decorations(false)
                    .transparent(true)
                    .resizable(false)
                    .skip_taskbar(true)
                    .shadow(false)
                    .always_on_bottom(true)
                    .focused(false)
                    .build()?;
                y += h + 12.0;
            }
            // bandeja com "Sair"
            let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let cfg = MenuItem::with_id(app, "cfg", "Configurações", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&cfg, &quit])?;
            let mut tray = TrayIconBuilder::new().menu(&menu).on_menu_event(|app, e| match e.id.as_ref() {
                "quit" => app.exit(0),
                "cfg" => open_cfg(app),
                _ => {}
            });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            // leitura de midia a cada 1s
            let h = app.handle().clone();
            thread::spawn(move || {
                let mut tk = String::new();
                let mgr = loop {
                    if let Ok(m) = Manager::RequestAsync().and_then(|o| o.get()) {
                        break m;
                    }
                    thread::sleep(Duration::from_secs(2));
                };
                loop {
                    let p = read(&mgr, &mut tk).ok().flatten().unwrap_or_default();
                    let _ = h.emit("media", &p);
                    thread::sleep(Duration::from_millis(1000));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar");
}
