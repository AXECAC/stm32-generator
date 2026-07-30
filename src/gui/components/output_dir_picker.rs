use std::path::PathBuf;

use adw::prelude::*;
use gtk::gio;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};

/// Модель компонента выбора директории для генерации проекта.
pub(crate) struct OutputDirPickerModel {
    /// Текущая выбранная директория назначения.
    output_dir: PathBuf,
    /// Доступен ли выбор директории пользователю.
    is_sensitive: bool,
    /// Сообщение об ошибке выбора директории.
    error: Option<String>,
    /// Открытый системный диалог выбора директории.
    active_dialog: Option<gtk::FileChooserNative>,
}

/// Входящие сообщения компонента выбора директории.
#[derive(Debug)]
pub(crate) enum OutputDirPickerInput {
    /// Пользователь запросил выбор директории через файловый менеджер.
    ChooseOutputDir,
    /// Пользователь выбрал директорию.
    OutputDirSelected(PathBuf),
    /// Обновить доступность выбора директории.
    SetSensitive(bool),
    /// Показать ошибку выбора директории.
    SetError(String),
    /// Открытый диалог выбора директории был закрыт.
    DialogClosed,
}

/// Исходящие сообщения компонента выбора директории.
#[derive(Debug)]
pub(crate) enum OutputDirPickerOutput {
    /// Выбранная директория изменилась.
    DirectoryChanged(PathBuf),
}

#[relm4::component(pub(crate))]
impl SimpleComponent for OutputDirPickerModel {
    type Init = PathBuf;
    type Input = OutputDirPickerInput;
    type Output = OutputDirPickerOutput;

    view! {
        adw::PreferencesGroup {
            set_title: "Параметры запуска",
            adw::ActionRow {
                set_title: "Директория проекта",
                #[watch]
                set_subtitle: model.output_dir_display().as_str(),

                add_suffix = &gtk::Button {
                    set_label: "Выбрать…",
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_sensitive: model.is_sensitive,

                    connect_clicked[sender] => move |_| {
                        sender.input(OutputDirPickerInput::ChooseOutputDir);
                    }
                }
            },

            gtk::Label {
                #[watch]
                set_label: model.error.as_deref().unwrap_or(""),
                #[watch]
                set_visible: model.error.is_some(),
                add_css_class: "error",
                set_wrap: true,
                set_xalign: 0.0,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = OutputDirPickerModel {
            output_dir: init,
            is_sensitive: true,
            error: None,
            active_dialog: None,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            OutputDirPickerInput::ChooseOutputDir => self.choose_output_dir(sender),
            OutputDirPickerInput::OutputDirSelected(path) => {
                if self.is_sensitive {
                    self.set_output_dir(path, sender);
                }
            }
            OutputDirPickerInput::SetSensitive(is_sensitive) => self.is_sensitive = is_sensitive,
            OutputDirPickerInput::SetError(message) => self.set_error(message),
            OutputDirPickerInput::DialogClosed => self.active_dialog = None,
        }
    }
}

impl OutputDirPickerModel {
    /// Возвращает путь директории назначения для отображения в UI.
    fn output_dir_display(&self) -> String {
        self.output_dir.display().to_string()
    }

    /// Открывает системный выбор директории.
    fn choose_output_dir(&mut self, sender: ComponentSender<Self>) {
        if !self.is_sensitive || self.active_dialog.is_some() {
            return;
        }

        let initial_folder = gio::File::for_path(&self.output_dir);
        let dialog = gtk::FileChooserNative::builder()
            .title("Выберите директорию для генерации")
            .action(gtk::FileChooserAction::SelectFolder)
            .accept_label("Выбрать")
            .cancel_label("Отмена")
            .modal(true)
            .build();

        if let Err(e) = dialog.set_file(&initial_folder) {
            log::error!(
                "Не удалось установить стартовую директорию файлового менеджера: {}",
                e
            );
        }

        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept
                && let Some(file) = dialog.file()
            {
                if let Some(path) = file.path() {
                    sender.input(OutputDirPickerInput::OutputDirSelected(path));
                } else {
                    sender.input(OutputDirPickerInput::SetError(
                        "Файловый менеджер вернул директорию без локального пути".to_string(),
                    ));
                }
            }

            dialog.destroy();
            sender.input(OutputDirPickerInput::DialogClosed);
        });

        self.active_dialog = Some(dialog.clone());
        dialog.show();
    }

    /// Обновляет выбранную директорию и сообщает родительскому компоненту.
    fn set_output_dir(&mut self, path: PathBuf, sender: ComponentSender<Self>) {
        self.output_dir = path.clone();
        self.error = None;

        if let Err(e) = sender.output(OutputDirPickerOutput::DirectoryChanged(path)) {
            log::error!(
                "Не удалось отправить DirectoryChanged из компонента выбора директории: {:?}",
                e
            );
        }
    }

    /// Устанавливает сообщение об ошибке выбора директории.
    fn set_error(&mut self, message: String) {
        log::error!("Ошибка выбора директории: {}", message);
        self.error = Some(message);
    }
}
