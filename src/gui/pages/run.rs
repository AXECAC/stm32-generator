use eframe::egui;
use std::path::PathBuf;
use crossbeam_channel::Receiver;

use crate::core::{
    config::Config,
    worker::{WorkerMessage, start_generation},
};

pub struct RunState {
    pub receiver: Option<Receiver<WorkerMessage>>,
    pub gen_status: String,
    pub gen_percent: f32,
    pub gen_finished: bool,
    pub gen_error: Option<String>,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            receiver: None,
            gen_status: "".to_string(),
            gen_percent: 0.0,
            gen_finished: false,
            gen_error: None,
        }
    }
}

impl RunState {
    pub fn render(&mut self, ui: &mut egui::Ui, config: &Config, output_path: &mut String) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(ui.available_height() / 4.0);
            
            ui.allocate_ui_with_layout(egui::vec2(500.0, ui.available_height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::Frame::group(ui.style())
                    .show(ui, |ui| {
                        ui.add_space(10.0);
                        ui.heading(egui::RichText::new("Генерация проекта").size(20.0));
                        ui.add_space(15.0);

                        ui.horizontal(|ui| {
                            ui.label("Путь для сохранения:");
                            ui.add(egui::TextEdit::singleline(output_path).desired_width(f32::INFINITY));
                        });

                    ui.add_space(20.0);

                    if self.receiver.is_none() && !self.gen_finished && ui.button(egui::RichText::new("🚀 Начать генерацию").size(16.0)).clicked() {
                        let path = PathBuf::from(output_path.clone());
                        self.receiver = Some(start_generation(config.clone(), path));
                        self.gen_status = "Запуск...".to_string();
                        self.gen_percent = 0.0;
                        self.gen_error = None;
                        self.gen_finished = false;
                    }

                    if let Some(rx) = &self.receiver {
                        while let Ok(msg) = rx.try_recv() {
                            match msg {
                                WorkerMessage::Progress { percent, status } => {
                                    self.gen_percent = percent as f32;
                                    self.gen_status = status;
                                }
                                WorkerMessage::Done { output_dir } => {
                                    self.gen_percent = 100.0;
                                    self.gen_status = format!("Успешно! Сохранено в {:?}", output_dir);
                                    self.gen_finished = true;
                                    self.receiver = None;
                                    break;
                                }
                                WorkerMessage::Error { message } => {
                                    self.gen_status = "Ошибка генерации".to_string();
                                    self.gen_error = Some(format!("{:?}", message));
                                    self.gen_finished = true;
                                    self.receiver = None;
                                    break;
                                }
                            }
                        }
                    }

                    if self.receiver.is_some() || self.gen_finished {
                        ui.add_space(15.0);
                        ui.label(format!("Статус: {}", self.gen_status));
                        ui.add_space(5.0);
                        ui.add(egui::ProgressBar::new(self.gen_percent / 100.0).show_percentage());

                        if let Some(err) = &self.gen_error {
                            ui.add_space(10.0);
                            ui.colored_label(egui::Color32::RED, format!("Ошибка: {}", err));
                        }
                    }
                });
            }); // close allocate_ui_with_layout
        });
    }
}
