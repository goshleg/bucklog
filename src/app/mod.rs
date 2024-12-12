use assets::Assets;
use config::{RabbitMQSettings, Settings};
use eframe::egui;
use egui_notify::Toasts;
use logs::Logs;

use crate::worker::{Worker, WorkerHandle};

mod assets;
pub mod config;
mod logs;
mod uis;

struct SettingsState {
    show: bool,
    host: String,
    vhost: String,
    port: String,
    username: String,
    password: String,
}

impl SettingsState {
    fn load(settings: &RabbitMQSettings) -> SettingsState {
        SettingsState {
            show: false,
            host: settings.host.clone(),
            vhost: settings.vhost.clone(),
            port: settings.port.to_string(),
            username: settings.username.clone(),
            password: settings.password.clone(),
        }
    }
}

#[derive(Default)]
struct TableState {
    as_countdown: bool,
}

struct UiState {
    settings_state: SettingsState,
    table_state: TableState,
    toasts: Toasts,
}

impl UiState {
    fn load(setttings: &Settings) -> Self {
        UiState {
            settings_state: SettingsState::load(&setttings.rabbit_mq),
            table_state: Default::default(),
            toasts: Toasts::default(),
        }
    }
}

pub struct App {
    assets: Assets,
    state: std::cell::Cell<UiState>,
    logs: Logs,
    w_handle: WorkerHandle,
    settings: std::cell::Cell<Settings>,
}

impl App {
    pub fn new(ctx: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        // Start async worker
        let egui_ctx = ctx.egui_ctx.clone();
        let w_handle = Worker::new(settings.rabbit_mq.clone(), egui_ctx).start();

        egui_extras::install_image_loaders(&ctx.egui_ctx);
        App {
            assets: Assets::load(),
            state: std::cell::Cell::new(UiState::load(&settings)),
            logs: Default::default(),
            w_handle,
            settings: std::cell::Cell::new(settings),
        }
    }

    fn draw(&mut self, ctx: &egui::Context) {
        uis::MainScreen::new(self).draw(ctx);
    }

    fn update_state(&mut self) {
        use crate::worker::Notification;
        let notifications = self.w_handle.get_notifications();
        if !notifications.is_empty() {
            for n in notifications {
                match n {
                    Notification::LogEntry(log_entry) => {
                        self.ui_state_mut()
                            .toasts
                            .info(format!("One more log: {}", log_entry.level.to_string()));
                        self.logs.append(log_entry);
                    }
                    Notification::ConnectionStatusChanged { status: Err(e) }
                    | Notification::Error(e) => {
                        self.ui_state_mut().toasts.info(format!("Error: {}", e));
                    }
                    Notification::ConnectionStatusChanged { status: Ok(()) } => {
                        self.ui_state_mut().toasts.info("Connected!");
                    }
                }
            }
        }
    }

    /// Drawing loop is one-line execution, so we can trust that
    fn ui_state_mut(&self) -> &mut UiState {
        unsafe { self.state.as_ptr().as_mut().unwrap() }
    }

    /// And that too
    fn settings_mut(&self) -> &mut Settings {
        unsafe { self.settings.as_ptr().as_mut().unwrap() }
    }

    fn apply_settings(&self) {
        let settings = &mut self.settings_mut().rabbit_mq;
        let settings_ui = &mut self.ui_state_mut().settings_state;
        settings.host = settings_ui.host.clone();
        settings.vhost = settings_ui.vhost.clone();
        settings.port = settings_ui.port.parse().expect("Failed to parse port");
        settings.username = settings_ui.username.clone();
        settings.password = settings_ui.password.clone();
        self.settings_mut()
            .write_configuration()
            .expect("Failed to write configuration");
        self.w_handle
            .command(crate::worker::Command::UpdateConfig(settings.clone()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.update_state();
        self.draw(ctx);
        self.ui_state_mut().toasts.show(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.logs.store().expect("Failed to store logs!");
    }
}
