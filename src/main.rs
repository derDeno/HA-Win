#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod discovery;
mod mqtt;
mod platform;
mod settings;
mod ui;
mod updates;

use native_windows_gui as nwg;
use nwg::NativeUi;
use std::sync::mpsc;

pub enum UiEvent {
    Connection(bool, String),
    Notification(String, String),
    Error(String),
    Test(Result<(), String>),
    Update(Result<Option<updates::Update>, String>, bool),
    Installed(Result<(), String>),
}

#[derive(Clone)]
pub struct UiSender {
    tx: mpsc::Sender<UiEvent>,
    notice: Option<nwg::NoticeSender>,
}

impl UiSender {
    fn send(&self, event: UiEvent) {
        if self.tx.send(event).is_ok() {
            if let Some(notice) = &self.notice {
                notice.notice();
            }
        }
    }
}

fn main() {
    if let Err(error) = run() {
        nwg::error_message("HA Win", &format!("{error:#}"));
    }
}

fn run() -> anyhow::Result<()> {
    use windows::Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS},
        System::{
            Threading::CreateMutexW,
            WinRT::{RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED},
        },
    };
    // Keep one MQTT client and one tray icon per Windows session.
    let mutex = unsafe { CreateMutexW(None, false, windows::core::w!("Local\\HA-Win-Rust"))? };
    struct Instance(windows::Win32::Foundation::HANDLE);
    impl Drop for Instance {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let _instance = Instance(mutex);
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        nwg::simple_message(
            "HA Win",
            "HA Win is already running. Open Settings from its tray icon.",
        );
        return Ok(());
    }
    let winrt = unsafe { RoInitialize(RO_INIT_SINGLETHREADED) }.is_ok();
    nwg::init()?;
    nwg::Font::set_global_family("Segoe UI")?;
    let path = settings::settings_path()?;
    // A malformed file is reported instead of silently replacing user credentials.
    let mut settings = settings::load(&path)?;
    settings.auto_start = platform::auto_start_enabled();
    let app = ui::App::build_ui(ui::App::new(settings, path))?;
    nwg::dispatch_thread_events();
    drop(app);
    if winrt {
        unsafe {
            RoUninitialize();
        }
    }
    Ok(())
}
