use std::fmt::Display;

use eframe::egui;
use egui::{Pos2, Rect, Rounding, Widget};

use crate::{default_time_format, worker};

use super::{logs::Logs, App};

pub struct MainScreen<'a> {
    app: &'a App,
}

impl<'a> MainScreen<'a> {
    pub fn new(app: &'a App) -> Self {
        MainScreen { app }
    }

    pub fn draw(&self, ctx: &egui::Context) {
        let app = self.app;
        let assets = &app.assets;
        let w_handle = &app.w_handle;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Sides::new().show(
                ui,
                |ui| {
                    let response = egui::ImageButton::new(assets.reload_icon.clone())
                        .uv(Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)).shrink(0.1))
                        .rounding(Rounding::same(5.0))
                        .ui(ui);
                    if response.clicked() {
                        w_handle.command(worker::Command::Reconnect);
                    }
                    egui_theme_switch::global_theme_switch(ui);
                },
                |ui| {
                    let response = egui::ImageButton::new(assets.settings_icon.clone())
                        .uv(Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)).shrink(0.1))
                        .rounding(Rounding::same(5.0))
                        .ui(ui);
                    if response.clicked() {
                        app.ui_state_mut().settings_state.show = true;
                    }
                },
            );
            egui::ScrollArea::vertical().show(ui, |ui| {
                table_ui(ui, false, &app);
            });

            if app.ui_state_mut().settings_state.show {
                SettingsWindow::new(app).draw(ctx);
            }
        });
    }
}

pub struct SettingsWindow<'a> {
    app: &'a App,
}

impl<'a> SettingsWindow<'a> {
    pub fn new(app: &'a App) -> Self {
        SettingsWindow { app }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        let state = &mut self.app.ui_state_mut().settings_state;
        let win_close = &mut self.app.ui_state_mut().settings_state.show;
        egui::Window::new("Settings")
            .open(win_close)
            .show(ctx, |ui| {
                ui.label("RabbitMQ host");
                egui::TextEdit::singleline(&mut state.host)
                    .hint_text("myhost.com")
                    .show(ui);
                ui.label("RabbitMQ virtual host");
                egui::TextEdit::singleline(&mut state.vhost)
                    .hint_text("demo-vhost")
                    .show(ui);
                ui.label("RabbitMQ port");
                egui::TextEdit::singleline(&mut state.port)
                    .hint_text("5672")
                    .show(ui);
                ui.label("RabbitMQ user");
                egui::TextEdit::singleline(&mut state.username)
                    .hint_text("rmquser")
                    .show(ui);
                ui.label("RabbitMQ password");
                egui::TextEdit::singleline(&mut state.password)
                    .password(true)
                    .show(ui);
                let resp = ui.button("Submit");
                if resp.clicked() {
                    self.app.apply_settings();
                    state.show = false;
                }
            });
    }
}

pub fn table_ui(ui: &mut egui::Ui, reset: bool, app: &App) {
    use egui_extras::{Column, TableBuilder};
    let logs = &app.logs;

    let text_height = egui::TextStyle::Body
        .resolve(ui.style())
        .size
        .max(ui.spacing().interact_size.y);

    let available_height = ui.available_height();
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true))
        .column(Column::auto()) // Idx
        .column(
            Column::remainder() // Timestamp
                .at_least(40.0)
                .clip(true)
                .resizable(true),
        )
        .column(
            Column::initial(50.0) // Level
                .at_least(50.0)
                .clip(true)
                .resizable(true),
        )
        .column(
            Column::remainder() // Source
                .at_least(70.0)
                .clip(true)
                .resizable(true),
        )
        .column(
            Column::initial(200.0) // Message
                .at_least(120.0)
                .clip(true)
                .resizable(true),
        )
        .column(
            Column::remainder() // Fields
                .at_least(70.0)
                .clip(true)
                .resizable(true),
        )
        .min_scrolled_height(0.0)
        .max_scroll_height(available_height);

    if true {
        table = table.sense(egui::Sense::click());
    }

    if reset {
        table.reset();
    }

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Idx");
            });
            header.col(|ui| {
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        ui.strong("Timestamp");
                    },
                    |ui| {
                        let as_countdown = app.ui_state_mut().table_state.as_countdown;
                        app.ui_state_mut().table_state.as_countdown ^=
                            ui.button(if as_countdown { "⬆" } else { "⬇" }).clicked();
                    },
                );
            });
            header.col(|ui| {
                ui.strong("Level");
            });
            header.col(|ui| {
                ui.strong("Source");
            });
            header.col(|ui| {
                ui.strong("Message");
            });
            header.col(|ui| {
                ui.strong("Fields");
            });
        })
        .body(|body| {
            body.rows(text_height, logs.entries().len(), |mut row| {
                let row_index = row.index();
                let entry = &logs.entries()[row_index];
                row.col(|ui| {
                    ui.label(row_index.to_string());
                });
                row.col(|ui| {
                    let as_countdown = app.ui_state_mut().table_state.as_countdown;

                    if as_countdown {
                        let duration = std::time::Duration::from_secs(
                            (time::OffsetDateTime::now_utc() - entry.timestamp).whole_seconds()
                                as u64,
                        );
                        ui.label(format!(
                            "{} ago",
                            humantime::format_duration(duration).to_string()
                        ));
                    } else {
                        ui.label(format!(
                            "{}",
                            entry.timestamp.format(default_time_format()).unwrap()
                        ));
                    }
                });
                row.col(|ui| {
                    ui.colored_label(entry.level.color(), entry.level.to_string());
                });
                row.col(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("{}:{}", entry.source.file, entry.source.line));
                    });
                });
                row.col(|ui| {
                    ui.label(&entry.message);
                });
                row.col(|ui| {
                    for (k, v) in &entry.fields {
                        ui.label(k);
                        ui.label(serde_json::to_string_pretty(v).expect("Failed to seriaze value"));
                    }
                });
            })
        });
}
