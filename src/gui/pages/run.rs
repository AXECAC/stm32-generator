use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use adw::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, adw, gtk,
};

use crate::core::config::Config;
use crate::core::worker::{WorkerMessage, start_generation};
use crate::gui::components::output_dir_picker::{
    OutputDirPickerInput, OutputDirPickerModel, OutputDirPickerOutput,
};

/// Модель страницы запуска генерации проекта.
pub struct RunPageModel {
    /// Глобальная конфигурация приложения.
    pub(crate) config: Arc<RwLock<Config>>,
    /// Директория, в которую будет записан сгенерированный проект.
    output_dir: PathBuf,
    /// Компонент выбора директории назначения.
    output_dir_picker: Controller<OutputDirPickerModel>,
    /// Текстовый статус текущей операции.
    status: String,
    /// Прогресс генерации в диапазоне `0.0..=1.0`.
    progress: f64,
    /// Сообщение об ошибке генерации или выбора директории.
    error: Option<String>,
    /// Путь к директории последней успешной генерации.
    done_dir: Option<PathBuf>,
    /// Флаг активной генерации.
    is_generating: bool,
}

/// Входящие сообщения страницы запуска генерации.
#[derive(Debug)]
pub enum RunPageInput {
    /// Вкладка стала активной; нужно перечитать свежий [`Config`].
    UpdateConfig,
    /// Компонент выбора директории сообщил новый путь.
    OutputDirSelected(PathBuf),
    /// Пользователь нажал кнопку генерации.
    StartGeneration,
    /// Worker прислал очередное сообщение.
    WorkerMessage(WorkerMessage),
}

#[relm4::component(pub)]
impl SimpleComponent for RunPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = RunPageInput;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,

            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,
                    set_margin_all: 32,
                    set_valign: gtk::Align::Start,

                    adw::StatusPage {
                        set_title: "Генерация проекта",
                        set_description: Some("Выберите директорию назначения и запустите генерацию Cargo-проекта."),
                        set_icon_name: Some("media-playback-start-symbolic"),
                    },

                    #[local_ref]
                    output_dir_picker_widget -> adw::PreferencesGroup {},

                    adw::PreferencesGroup {
                        set_title: "Статус",

                        adw::ActionRow {
                            set_title: "Текущий этап",
                            #[watch]
                            set_subtitle: model.status.as_str(),
                        },

                        gtk::ProgressBar {
                            #[watch]
                            set_fraction: model.progress,
                            #[watch]
                            set_show_text: model.is_generating || model.progress > 0.0,
                            #[watch]
                            set_text: Some(&format!("{:.0}%", model.progress * 100.0)),
                        },

                        gtk::Label {
                            #[watch]
                            set_label: model.error.as_deref().unwrap_or(""),
                            #[watch]
                            set_visible: model.error.is_some(),
                            add_css_class: "error",
                            set_wrap: true,
                            set_xalign: 0.0,
                        },

                        gtk::Label {
                            #[watch]
                            set_label: model.done_message().as_deref().unwrap_or(""),
                            #[watch]
                            set_visible: model.done_dir.is_some(),
                            add_css_class: "success",
                            set_wrap: true,
                            set_xalign: 0.0,
                        }
                    },

                    gtk::Button {
                        #[watch]
                        set_label: if model.is_generating { "Генерация…" } else { "Сгенерировать проект" },
                        add_css_class: "suggested-action",
                        add_css_class: "pill",
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_sensitive: !model.is_generating,

                        connect_clicked[sender] => move |_| {
                            sender.input(RunPageInput::StartGeneration);
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let output_dir = std::env::current_dir().unwrap_or_else(|e| {
            log::error!("Не удалось получить текущую директорию: {}", e);
            PathBuf::from(".")
        });

        let output_dir_picker = OutputDirPickerModel::builder()
            .launch(output_dir.clone())
            .forward(sender.input_sender(), |output| match output {
                OutputDirPickerOutput::DirectoryChanged(path) => {
                    RunPageInput::OutputDirSelected(path)
                }
            });

        let model = RunPageModel {
            config: init,
            output_dir,
            output_dir_picker,
            status: "Готово к генерации".to_string(),
            progress: 0.0,
            error: None,
            done_dir: None,
            is_generating: false,
        };

        let output_dir_picker_widget = model.output_dir_picker.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            RunPageInput::UpdateConfig => {}
            RunPageInput::OutputDirSelected(path) => self.set_output_dir(path),
            RunPageInput::StartGeneration => self.start_generation(sender),
            RunPageInput::WorkerMessage(message) => self.handle_worker_message(message),
        }
    }
}

impl RunPageModel {
    /// Возвращает сообщение об успешной генерации.
    fn done_message(&self) -> Option<String> {
        self.done_dir
            .as_ref()
            .map(|path| format!("Проект успешно сгенерирован: {}", path.display()))
    }

    /// Обновляет директорию назначения.
    fn set_output_dir(&mut self, path: PathBuf) {
        self.output_dir = path;
        self.error = None;
        self.done_dir = None;
        self.status = "Готово к генерации".to_string();
        self.progress = 0.0;
    }

    /// Запускает генерацию проекта в worker-потоке.
    fn start_generation(&mut self, sender: ComponentSender<Self>) {
        if self.is_generating {
            return;
        }

        let config = self.config.read().unwrap().clone();
        let output_dir = self.output_dir.clone();

        self.error = None;
        self.done_dir = None;
        self.status = "Запуск генерации...".to_string();
        self.progress = 0.0;
        self.is_generating = true;
        self.set_output_dir_picker_sensitive(false);

        let receiver = start_generation(config, output_dir);
        let input_sender = sender.input_sender().clone();
        std::thread::spawn(move || {
            for message in receiver {
                if input_sender
                    .send(RunPageInput::WorkerMessage(message))
                    .is_err()
                {
                    log::error!("Не удалось отправить WorkerMessage в RunPageModel");
                    break;
                }
            }
        });
    }

    /// Обрабатывает сообщение worker-а.
    fn handle_worker_message(&mut self, message: WorkerMessage) {
        match message {
            WorkerMessage::Progress { percent, status } => {
                self.progress = (percent as f64 / 100.0).clamp(0.0, 1.0);
                self.status = status;
            }
            WorkerMessage::Done { output_dir } => {
                self.progress = 1.0;
                self.status = "Генерация завершена".to_string();
                self.done_dir = Some(output_dir);
                self.error = None;
                self.is_generating = false;
                self.set_output_dir_picker_sensitive(true);
            }
            WorkerMessage::Error { message } => {
                self.status = "Генерация завершилась ошибкой".to_string();
                self.error = Some(message.to_string());
                self.done_dir = None;
                self.is_generating = false;
                self.set_output_dir_picker_sensitive(true);
            }
        }
    }

    /// Обновляет доступность компонента выбора директории.
    fn set_output_dir_picker_sensitive(&self, is_sensitive: bool) {
        if let Err(e) = self
            .output_dir_picker
            .sender()
            .send(OutputDirPickerInput::SetSensitive(is_sensitive))
        {
            log::error!(
                "Не удалось отправить SetSensitive в компонент выбора директории: {:?}",
                e
            );
        }
    }
}
