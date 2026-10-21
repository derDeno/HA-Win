use crate::{
    mqtt, platform,
    settings::{self, Settings},
    updates, UiEvent, UiSender,
};
use anyhow::{Context, Result};
use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::mpsc,
    thread,
};

#[derive(Default, NwgUi)]
pub struct App {
    saved: RefCell<Settings>,
    path: PathBuf,
    initialized: Cell<bool>,
    receiver: RefCell<Option<mpsc::Receiver<UiEvent>>>,
    sender: RefCell<Option<UiSender>>,
    service: RefCell<Option<mqtt::Service>>,
    connected: Rc<Cell<bool>>,
    status_style: RefCell<Option<nwg::RawEventHandler>>,

    #[nwg_resource]
    resources: nwg::EmbedResource,
    #[nwg_resource(source_embed: Some(&data.resources), source_embed_id: 1)]
    icon: nwg::Icon,
    #[nwg_resource(family: "Segoe UI", weight: 700)]
    status_font: nwg::Font,

    #[nwg_control(size: (520, 480), center: true, title: "HA Win Settings", flags: "WINDOW|MINIMIZE_BOX", icon: Some(&data.icon))]
    #[nwg_events(OnInit: [App::initialize], OnWindowClose: [App::hide])]
    window: nwg::Window,

    #[nwg_control(parent: window, size: (496, 412), position: (12, 12))]
    tabs: nwg::TabsContainer,
    #[nwg_control(parent: tabs, text: "Status")]
    status_tab: nwg::Tab,
    #[nwg_control(parent: status_tab, text: "MQTT Connection", position: (16, 18), size: (450, 22), background_color: Some([255, 255, 255]))]
    connection_caption: nwg::Label,
    #[nwg_control(parent: status_tab, text: "Not running", position: (16, 44), size: (450, 80), background_color: Some([255, 255, 255]), font: Some(&data.status_font))]
    connection_status: nwg::Label,
    #[nwg_control(parent: status_tab, text: "PC Name", position: (16, 140), size: (450, 22), background_color: Some([255, 255, 255]))]
    machine_caption: nwg::Label,
    #[nwg_control(parent: status_tab, position: (16, 166), size: (450, 22), background_color: Some([255, 255, 255]))]
    machine: nwg::Label,
    #[nwg_control(parent: status_tab, text: "Version", position: (16, 212), size: (450, 22), background_color: Some([255, 255, 255]))]
    version_caption: nwg::Label,
    #[nwg_control(parent: status_tab, text: env!("CARGO_PKG_VERSION"), position: (16, 238), size: (450, 22), background_color: Some([255, 255, 255]))]
    version: nwg::Label,

    #[nwg_control(parent: tabs, text: "MQTT")]
    mqtt_tab: nwg::Tab,
    #[nwg_control(parent: mqtt_tab, text: "Broker Host", position: (12, 14), size: (130, 22), background_color: Some([255, 255, 255]))]
    host_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 10), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    host: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Broker Port", position: (12, 48), size: (130, 22), background_color: Some([255, 255, 255]))]
    port_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 44), size: (320, 26), limit: 5)]
    #[nwg_events(OnTextInput: [App::changed])]
    port: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Namespace", position: (12, 94), size: (130, 22), background_color: Some([255, 255, 255]))]
    namespace_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 90), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    namespace: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Username", position: (12, 140), size: (130, 22), background_color: Some([255, 255, 255]))]
    username_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 136), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    username: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Password", position: (12, 174), size: (130, 22), background_color: Some([255, 255, 255]))]
    password_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 170), size: (320, 26), password: Some('●'))]
    #[nwg_events(OnTextInput: [App::changed])]
    password: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Client ID", position: (12, 220), size: (130, 22), background_color: Some([255, 255, 255]))]
    client_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 216), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    client_id: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Use TLS", position: (12, 252), size: (150, 24), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    tls: nwg::CheckBox,
    #[nwg_control(parent: mqtt_tab, text: "Notify Topic", position: (12, 298), size: (130, 22), background_color: Some([255, 255, 255]))]
    notify_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 294), size: (320, 26), readonly: true)]
    notify: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, text: "Test Connection", position: (318, 338), size: (150, 30))]
    #[nwg_events(OnButtonClick: [App::test])]
    test_button: nwg::Button,

    #[nwg_control(parent: tabs, text: "App")]
    app_tab: nwg::Tab,
    #[nwg_control(parent: app_tab, text: "App Settings", position: (16, 18), size: (450, 24), background_color: Some([255, 255, 255]))]
    app_caption: nwg::Label,
    #[nwg_control(parent: app_tab, text: "Start with Windows", position: (16, 54), size: (450, 26), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    auto_start: nwg::CheckBox,
    #[nwg_control(parent: app_tab, text: "Show window on startup", position: (16, 88), size: (450, 26), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    show_window: nwg::CheckBox,
    #[nwg_control(parent: app_tab, text: "Check for updates on launch", position: (16, 122), size: (450, 26), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    auto_updates: nwg::CheckBox,
    #[nwg_control(parent: app_tab, text: "Check now for updates", position: (16, 158), size: (190, 30))]
    #[nwg_events(OnButtonClick: [App::check_updates])]
    update_button: nwg::Button,
    #[nwg_control(parent: app_tab, text: "Open GitHub", position: (16, 210), size: (140, 30))]
    #[nwg_events(OnButtonClick: [App::github])]
    github_button: nwg::Button,

    #[nwg_control(parent: window, text: "Save", position: (310, 438), size: (100, 30), enabled: false)]
    #[nwg_events(OnButtonClick: [App::save])]
    save_button: nwg::Button,
    #[nwg_control(parent: window, text: "Hide", position: (418, 438), size: (90, 30))]
    #[nwg_events(OnButtonClick: [App::hide])]
    hide_button: nwg::Button,

    #[nwg_control(parent: window, icon: Some(&data.icon), tip: Some("HA Win"))]
    #[nwg_events(MousePressLeftUp: [App::show], OnContextMenu: [App::tray_menu])]
    tray: nwg::TrayNotification,
    #[nwg_control(parent: window, popup: true)]
    menu: nwg::Menu,
    #[nwg_control(parent: menu, text: "Open Settings")]
    #[nwg_events(OnMenuItemSelected: [App::show])]
    open_item: nwg::MenuItem,
    #[nwg_control(parent: menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [App::exit])]
    exit_item: nwg::MenuItem,
    #[nwg_control(parent: window)]
    #[nwg_events(OnNotice: [App::receive])]
    notice: nwg::Notice,
}

impl App {
    pub fn new(settings: Settings, path: PathBuf) -> Self {
        let mut app = Self::default();
        app.saved.replace(settings);
        app.path = path;
        app
    }

    fn initialize(&self) {
        let connected = self.connected.clone();
        let status_hwnd = self.connection_status.handle.hwnd().unwrap();
        // NWG supplies the label's background brush; set its text color on the same paint request.
        *self.status_style.borrow_mut() = Some(
            nwg::bind_raw_event_handler(
                &self.status_tab.handle,
                0x10001,
                move |_, message, dc, child| {
                    if message == windows::Win32::UI::WindowsAndMessaging::WM_CTLCOLORSTATIC
                        && child == status_hwnd as isize
                    {
                        unsafe {
                            windows::Win32::Graphics::Gdi::SetTextColor(
                                windows::Win32::Graphics::Gdi::HDC(dc as *mut _),
                                windows::Win32::Foundation::COLORREF(if connected.get() {
                                    0x008000
                                } else {
                                    0x0000ff
                                }),
                            );
                        }
                    }
                    None
                },
            )
            .expect("Could not style MQTT connection status"),
        );
        let saved = self.saved.borrow();
        self.host.set_text(&saved.broker_host);
        self.port.set_text(&saved.broker_port.to_string());
        self.namespace.set_text(&saved.namespace);
        self.namespace
            .set_placeholder_text(Some(&saved.device_id()));
        self.username.set_text(&saved.username);
        self.password.set_text(&saved.password);
        self.client_id.set_text(&saved.client_id);
        self.tls.set_check_state(check_state(saved.use_tls));
        self.auto_start
            .set_check_state(check_state(saved.auto_start));
        self.show_window
            .set_check_state(check_state(saved.show_window_on_startup));
        self.auto_updates
            .set_check_state(check_state(saved.auto_check_updates));
        self.machine.set_text(&settings::machine_name());
        self.notify
            .set_text(&crate::discovery::notify(&saved.device_id()));
        let (tx, rx) = mpsc::channel();
        let sender = UiSender {
            tx,
            notice: Some(self.notice.sender()),
        };
        *self.receiver.borrow_mut() = Some(rx);
        *self.sender.borrow_mut() = Some(sender.clone());
        *self.service.borrow_mut() = Some(mqtt::Service::new(saved.clone(), sender));
        if let Err(e) = platform::register_notifications() {
            self.tray.show(
                &e.to_string(),
                Some("Notification registration failed"),
                None,
                None,
            );
        }
        self.initialized.set(true);
        self.window.set_visible(saved.show_window_on_startup);
        if saved.auto_check_updates {
            self.start_update_check(true);
        }
    }

    fn read(&self) -> Result<Settings> {
        let settings = Settings {
            broker_host: self.host.text().trim().into(),
            broker_port: self
                .port
                .text()
                .trim()
                .parse()
                .context("Broker port must be between 1 and 65535.")?,
            namespace: self.namespace.text(),
            username: self.username.text(),
            password: self.password.text(),
            client_id: self.client_id.text(),
            use_tls: checked(&self.tls),
            auto_start: checked(&self.auto_start),
            show_window_on_startup: checked(&self.show_window),
            auto_check_updates: checked(&self.auto_updates),
        };
        settings.validate()?;
        Ok(settings)
    }

    fn changed(&self) {
        if !self.initialized.get() {
            return;
        }
        let settings = Settings {
            namespace: self.namespace.text(),
            ..Default::default()
        };
        self.notify
            .set_text(&crate::discovery::notify(&settings.device_id()));
        self.save_button
            .set_enabled(self.read().map_or(true, |s| s != *self.saved.borrow()));
    }

    fn save(&self) {
        let result = (|| -> Result<()> {
            let settings = self.read()?;
            platform::set_auto_start(settings.auto_start)
                .context("Could not update Windows auto-start")?;
            if let Err(e) = settings::save(&self.path, &settings) {
                let _ = platform::set_auto_start(self.saved.borrow().auto_start);
                return Err(e);
            }
            if let Some(service) = self.service.borrow().as_ref() {
                service.apply(settings.clone());
            }
            *self.saved.borrow_mut() = settings;
            self.save_button.set_enabled(false);
            Ok(())
        })();
        if let Err(e) = result {
            self.error(&format!("{e:#}"));
        }
    }

    fn sender(&self) -> UiSender {
        self.sender.borrow().as_ref().unwrap().clone()
    }

    fn test(&self) {
        match self.read() {
            Ok(settings) => {
                self.test_button.set_enabled(false);
                self.test_button.set_text("Testing…");
                let sender = self.sender();
                thread::spawn(move || {
                    sender.send(UiEvent::Test(
                        mqtt::test_connection(&settings).map_err(|e| e.to_string()),
                    ))
                });
            }
            Err(e) => self.error(&e.to_string()),
        }
    }

    fn check_updates(&self) {
        self.start_update_check(false);
    }
    fn start_update_check(&self, silent: bool) {
        self.update_button.set_enabled(false);
        self.update_button.set_text("Checking…");
        let sender = self.sender();
        thread::spawn(move || {
            sender.send(UiEvent::Update(
                updates::check().map_err(|e| e.to_string()),
                silent,
            ))
        });
    }

    fn receive(&self) {
        let events: Vec<_> = self
            .receiver
            .borrow()
            .as_ref()
            .unwrap()
            .try_iter()
            .collect();
        for event in events {
            match event {
                UiEvent::Connection(connected, message) => {
                    self.connected.set(connected);
                    self.connection_status.set_text(&message);
                    self.tray.set_tip(if connected {
                        "HA Win — MQTT connected"
                    } else {
                        "HA Win — MQTT disconnected"
                    });
                }
                UiEvent::Notification(title, message) => {
                    if platform::show_notification(&title, &message).is_err() {
                        self.tray.show(
                            &message,
                            Some(&title),
                            Some(nwg::TrayNotificationFlags::INFO_ICON),
                            None,
                        );
                    }
                }
                UiEvent::Error(message) => {
                    self.tray.show(
                        &message,
                        Some("HA Win"),
                        Some(nwg::TrayNotificationFlags::ERROR_ICON),
                        None,
                    );
                }
                UiEvent::Test(result) => {
                    self.test_button.set_enabled(true);
                    self.test_button.set_text("Test Connection");
                    match result {
                        Ok(()) => {
                            nwg::modal_info_message(
                                &self.window,
                                "MQTT Test",
                                "Connection successful.",
                            );
                        }
                        Err(e) => self.error(&format!("Connection failed: {e}")),
                    }
                }
                UiEvent::Update(result, silent) => {
                    self.update_button.set_enabled(true);
                    self.update_button.set_text("Check now for updates");
                    match result {
                        Ok(Some(update)) => {
                            self.show();
                            let choice = nwg::modal_message(&self.window, &nwg::MessageParams {
                                title: "Update Available", content: &format!("A new version ({}) is available. Download and install it now?", update.version),
                                buttons: nwg::MessageButtons::YesNo, icons: nwg::MessageIcons::Info,
                            });
                            if choice == nwg::MessageChoice::Yes {
                                self.update_button.set_enabled(false);
                                self.update_button.set_text("Downloading…");
                                let sender = self.sender();
                                thread::spawn(move || {
                                    sender.send(UiEvent::Installed(
                                        updates::install(&update).map_err(|e| e.to_string()),
                                    ))
                                });
                            }
                        }
                        Ok(None) if !silent => {
                            nwg::modal_info_message(
                                &self.window,
                                "Update Check",
                                "You are running the latest version.",
                            );
                        }
                        Err(e) if !silent => self.error(&format!("Update check failed: {e}")),
                        _ => {}
                    }
                }
                UiEvent::Installed(result) => {
                    self.update_button.set_enabled(true);
                    self.update_button.set_text("Check now for updates");
                    match result {
                        Ok(()) => self.exit(),
                        Err(e) => self.error(&format!("Update failed: {e}")),
                    }
                }
            }
        }
    }

    fn error(&self, message: &str) {
        nwg::modal_error_message(&self.window, "HA Win", message);
    }
    fn hide(&self) {
        self.window.set_visible(false);
    }
    fn show(&self) {
        self.window.set_visible(true);
        self.window.restore();
        self.window.set_focus();
    }
    fn tray_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.menu.popup(x, y);
    }
    fn github(&self) {
        if let Err(e) = platform::open_url("https://github.com/derDeno/HA-Win") {
            self.error(&e.to_string());
        }
    }
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(handler) = self.status_style.get_mut().take() {
            let _ = nwg::unbind_raw_event_handler(&handler);
        }
    }
}

fn checked(control: &nwg::CheckBox) -> bool {
    control.check_state() == nwg::CheckBoxState::Checked
}
fn check_state(value: bool) -> nwg::CheckBoxState {
    if value {
        nwg::CheckBoxState::Checked
    } else {
        nwg::CheckBoxState::Unchecked
    }
}
