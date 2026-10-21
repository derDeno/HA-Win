use crate::{
    i18n::{self, Language},
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
    connection: RefCell<mqtt::ConnectionStatus>,
    downloading: Cell<bool>,

    #[nwg_resource]
    resources: nwg::EmbedResource,
    #[nwg_resource(source_embed: Some(&data.resources), source_embed_id: 1)]
    icon: nwg::Icon,
    #[nwg_resource(family: "Segoe UI", weight: 700)]
    status_font: nwg::Font,
    #[nwg_resource(family: "Segoe UI", size_absolute: 12)]
    hint_font: nwg::Font,

    #[nwg_control(size: (520, 480), center: true, flags: "WINDOW|MINIMIZE_BOX", icon: Some(&data.icon))]
    #[nwg_events(OnInit: [App::initialize], OnWindowClose: [App::hide])]
    window: nwg::Window,

    #[nwg_control(parent: window, size: (496, 412), position: (12, 12))]
    tabs: nwg::TabsContainer,
    #[nwg_control(parent: tabs)]
    status_tab: nwg::Tab,
    #[nwg_control(parent: status_tab, position: (16, 18), size: (450, 22), background_color: Some([255, 255, 255]))]
    connection_caption: nwg::Label,
    #[nwg_control(parent: status_tab, position: (16, 44), size: (450, 80), background_color: Some([255, 255, 255]), font: Some(&data.status_font))]
    connection_status: nwg::Label,
    #[nwg_control(parent: status_tab, position: (16, 140), size: (450, 22), background_color: Some([255, 255, 255]))]
    machine_caption: nwg::Label,
    #[nwg_control(parent: status_tab, position: (16, 166), size: (450, 22), background_color: Some([255, 255, 255]))]
    machine: nwg::Label,
    #[nwg_control(parent: status_tab, position: (16, 212), size: (450, 22), background_color: Some([255, 255, 255]))]
    version_caption: nwg::Label,
    #[nwg_control(parent: status_tab, text: env!("CARGO_PKG_VERSION"), position: (16, 238), size: (450, 22), background_color: Some([255, 255, 255]))]
    version: nwg::Label,

    #[nwg_control(parent: tabs, text: "MQTT")]
    mqtt_tab: nwg::Tab,
    #[nwg_control(parent: mqtt_tab, position: (12, 14), size: (130, 22), background_color: Some([255, 255, 255]))]
    host_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 10), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    host: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (12, 48), size: (130, 22), background_color: Some([255, 255, 255]))]
    port_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 44), size: (320, 26), limit: 5)]
    #[nwg_events(OnTextInput: [App::changed])]
    port: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (12, 94), size: (130, 22), background_color: Some([255, 255, 255]))]
    namespace_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 90), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    namespace: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (12, 140), size: (130, 22), background_color: Some([255, 255, 255]))]
    username_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 136), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    username: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (12, 174), size: (130, 22), background_color: Some([255, 255, 255]))]
    password_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 170), size: (320, 26), password: Some('●'))]
    #[nwg_events(OnTextInput: [App::changed])]
    password: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (12, 220), size: (130, 22), background_color: Some([255, 255, 255]))]
    client_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 216), size: (320, 26))]
    #[nwg_events(OnTextInput: [App::changed])]
    client_id: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (12, 252), size: (150, 24), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    tls: nwg::CheckBox,
    #[nwg_control(parent: mqtt_tab, position: (12, 298), size: (130, 22), background_color: Some([255, 255, 255]))]
    notify_label: nwg::Label,
    #[nwg_control(parent: mqtt_tab, position: (148, 294), size: (320, 26), readonly: true)]
    notify: nwg::TextInput,
    #[nwg_control(parent: mqtt_tab, position: (258, 338), size: (210, 30))]
    #[nwg_events(OnButtonClick: [App::test])]
    test_button: nwg::Button,

    #[nwg_control(parent: tabs)]
    app_tab: nwg::Tab,
    #[nwg_control(parent: app_tab, position: (16, 18), size: (450, 24), background_color: Some([255, 255, 255]))]
    app_caption: nwg::Label,
    #[nwg_control(parent: app_tab, position: (16, 58), size: (100, 22), background_color: Some([255, 255, 255]))]
    language_label: nwg::Label,
    #[nwg_control(parent: app_tab, position: (124, 54), size: (342, 100), collection: Vec::<String>::new())]
    #[nwg_events(OnComboxBoxSelection: [App::changed])]
    language: nwg::ComboBox<String>,
    #[nwg_control(parent: app_tab, position: (124, 88), size: (342, 32), flags: "VISIBLE|DISABLED", background_color: Some([255, 255, 255]), font: Some(&data.hint_font))]
    language_hint: nwg::Label,

    #[nwg_control(parent: app_tab, position: (16, 134), size: (450, 26), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    auto_start: nwg::CheckBox,
    #[nwg_control(parent: app_tab, position: (16, 168), size: (450, 26), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    show_window: nwg::CheckBox,
    #[nwg_control(parent: app_tab, position: (16, 202), size: (450, 26), background_color: Some([255, 255, 255]))]
    #[nwg_events(OnButtonClick: [App::changed])]
    auto_updates: nwg::CheckBox,
    #[nwg_control(parent: app_tab, position: (16, 238), size: (240, 30))]
    #[nwg_events(OnButtonClick: [App::check_updates])]
    update_button: nwg::Button,
    #[nwg_control(parent: app_tab, position: (16, 290), size: (140, 30))]
    #[nwg_events(OnButtonClick: [App::github])]
    github_button: nwg::Button,

    #[nwg_control(parent: window, position: (310, 438), size: (100, 30), enabled: false)]
    #[nwg_events(OnButtonClick: [App::save])]
    save_button: nwg::Button,
    #[nwg_control(parent: window, position: (418, 438), size: (90, 30))]
    #[nwg_events(OnButtonClick: [App::hide])]
    hide_button: nwg::Button,

    #[nwg_control(parent: window, icon: Some(&data.icon), tip: Some("HA Win"))]
    #[nwg_events(MousePressLeftUp: [App::show], OnContextMenu: [App::tray_menu])]
    tray: nwg::TrayNotification,
    #[nwg_control(parent: window, popup: true)]
    menu: nwg::Menu,
    #[nwg_control(parent: menu)]
    #[nwg_events(OnMenuItemSelected: [App::show])]
    open_item: nwg::MenuItem,
    #[nwg_control(parent: menu)]
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
        self.apply_language();
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
                Some(&i18n::text().notification_registration_failed),
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

    fn apply_language(&self) {
        let t = i18n::text();
        self.window.set_text(&t.window_title);
        self.status_tab.set_text(&t.status_tab);
        self.connection_caption.set_text(&t.connection_caption);
        self.machine_caption.set_text(&t.machine_caption);
        self.version_caption.set_text(&t.version_caption);
        self.app_tab.set_text(&t.app_tab);
        self.host_label.set_text(&t.host_label);
        self.port_label.set_text(&t.port_label);
        self.namespace_label.set_text(&t.namespace_label);
        self.username_label.set_text(&t.username_label);
        self.password_label.set_text(&t.password_label);
        self.client_label.set_text(&t.client_label);
        self.tls.set_text(&t.tls);
        self.notify_label.set_text(&t.notify_label);
        self.app_caption.set_text(&t.app_caption);
        self.auto_start.set_text(&t.auto_start);
        self.show_window.set_text(&t.show_window);
        self.auto_updates.set_text(&t.auto_updates);
        self.github_button.set_text(&t.github_button);
        self.save_button.set_text(&t.save_button);
        self.hide_button.set_text(&t.hide_button);
        set_menu_text(&self.open_item, &t.open_item);
        set_menu_text(&self.exit_item, &t.exit_item);
        self.language_label.set_text(&t.language_label);
        self.language_hint.set_text(&t.language_hint);
        self.test_button.set_text(if self.test_button.enabled() {
            &t.test_button
        } else {
            &t.testing
        });
        self.update_button.set_text(if self.downloading.get() {
            &t.downloading
        } else if self.update_button.enabled() {
            &t.update_button
        } else {
            &t.checking
        });
        let selection = self.language.selection().unwrap_or_else(|| {
            Language::ALL
                .iter()
                .position(|l| *l == self.saved.borrow().language)
                .unwrap()
        });
        self.language.set_collection(vec![
            t.automatic.clone(),
            "English".into(),
            "Deutsch".into(),
        ]);
        self.language.set_selection(Some(selection));
        self.show_connection();
    }

    fn show_connection(&self) {
        self.connection_status
            .set_text(&self.connection.borrow().text());
        let t = i18n::text();
        self.tray.set_tip(if self.connected.get() {
            &t.tray_connected
        } else {
            &t.tray_disconnected
        });
    }

    fn read(&self) -> Result<Settings> {
        let language = Language::ALL[self.language.selection().unwrap_or(0)];
        let settings = Settings {
            broker_host: self.host.text().trim().into(),
            broker_port: self
                .port
                .text()
                .trim()
                .parse()
                .context(&language.strings().invalid_port)?,
            namespace: self.namespace.text(),
            username: self.username.text(),
            password: self.password.text(),
            client_id: self.client_id.text(),
            use_tls: checked(&self.tls),
            auto_start: checked(&self.auto_start),
            show_window_on_startup: checked(&self.show_window),
            auto_check_updates: checked(&self.auto_updates),
            language,
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
            let auto_start_changed = settings.auto_start != self.saved.borrow().auto_start;
            if auto_start_changed {
                platform::set_auto_start(settings.auto_start)
                    .context(&i18n::text().auto_start_failed)?;
            }
            if let Err(e) = settings::save(&self.path, &settings) {
                if auto_start_changed {
                    let _ = platform::set_auto_start(self.saved.borrow().auto_start);
                }
                return Err(e);
            }
            if let Some(service) = self.service.borrow().as_ref() {
                service.apply(settings.clone());
            }
            i18n::set_language(settings.language);
            *self.saved.borrow_mut() = settings;
            self.apply_language();
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
                self.test_button.set_text(&i18n::text().testing);
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
        self.update_button.set_text(&i18n::text().checking);
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
                UiEvent::Connection(status) => {
                    self.connected.set(status.connected());
                    self.connection.replace(status);
                    self.show_connection();
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
                    self.test_button.set_text(&i18n::text().test_button);
                    match result {
                        Ok(()) => {
                            nwg::modal_info_message(
                                &self.window,
                                &i18n::text().test_title,
                                &i18n::text().test_success,
                            );
                        }
                        Err(e) => {
                            self.error(&i18n::text().connection_failed.replace("{error}", &e))
                        }
                    }
                }
                UiEvent::Update(result, silent) => {
                    self.update_button.set_enabled(true);
                    self.update_button.set_text(&i18n::text().update_button);
                    match result {
                        Ok(Some(update)) => {
                            self.show();
                            let choice = nwg::modal_message(
                                &self.window,
                                &nwg::MessageParams {
                                    title: &i18n::text().update_available,
                                    content: &i18n::text()
                                        .update_prompt
                                        .replace("{version}", &update.version.to_string()),
                                    buttons: nwg::MessageButtons::YesNo,
                                    icons: nwg::MessageIcons::Info,
                                },
                            );
                            if choice == nwg::MessageChoice::Yes {
                                self.update_button.set_enabled(false);
                                self.downloading.set(true);
                                self.update_button.set_text(&i18n::text().downloading);
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
                                &i18n::text().update_check,
                                &i18n::text().up_to_date,
                            );
                        }
                        Err(e) if !silent => {
                            self.error(&i18n::text().update_check_failed.replace("{error}", &e))
                        }
                        _ => {}
                    }
                }
                UiEvent::Installed(result) => {
                    self.downloading.set(false);
                    self.update_button.set_enabled(true);
                    self.update_button.set_text(&i18n::text().update_button);
                    match result {
                        Ok(()) => self.exit(),
                        Err(e) => self.error(&i18n::text().update_failed.replace("{error}", &e)),
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

fn set_menu_text(item: &nwg::MenuItem, text: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{ModifyMenuW, HMENU, MF_BYCOMMAND, MF_STRING};
    let (menu, id) = item.handle.hmenu_item().unwrap();
    // NWG has no setter for menu item text; preserve the ID used by its event handler.
    unsafe {
        ModifyMenuW(
            HMENU(menu.cast()),
            id,
            MF_BYCOMMAND | MF_STRING,
            id as usize,
            &windows::core::HSTRING::from(text),
        )
        .expect("Could not translate tray menu");
    }
}
fn check_state(value: bool) -> nwg::CheckBoxState {
    if value {
        nwg::CheckBoxState::Checked
    } else {
        nwg::CheckBoxState::Unchecked
    }
}
