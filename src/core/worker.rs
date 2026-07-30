use crossbeam_channel::Receiver;

use crate::core::{config::Config, errors::GeneratorError};
use std::path::PathBuf;

use crate::core::generator::{context::TemplateContext, render, writer::create_project};

const FAILED_SEND_ERR_MES: &str = "Worker не смог отправить прогресс (receiver отключен)";

/// Сообщения, в GUI о процессе генерации проекта
#[derive(Debug)]
pub enum WorkerMessage {
    /// Промежуточный прогресс, от 0 до 100, и текстовое описание этапа
    Progress { percent: u32, status: String },
    /// Генерация успешно завершена
    Done { output_dir: PathBuf },
    /// Произошла фатальная ошибка
    Error { message: GeneratorError },
}

/// Запускает процесс генерации проекта по собраной конфигурации в отдельном потоке.
///
/// Возвращает канал [`Receiver`], из которого GUI будет читать статусы [`WorkerMessage`].
pub fn start_generation(config: Config, output_dir: PathBuf) -> Receiver<WorkerMessage> {
    let (sender, receiver) = crossbeam_channel::unbounded();

    std::thread::spawn(move || {
        macro_rules! send_or_return {
            ($msg:expr) => {
                if let Err(e) = sender.send($msg) {
                    log::error!("{}: {}", FAILED_SEND_ERR_MES, e);
                    return;
                }
            };
        }

        send_or_return!(WorkerMessage::Progress {
            percent: 10,
            status: "Сборка контекста шаблона...".to_string(),
        });

        let project_name = output_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let context = match TemplateContext::from_config(&config, project_name) {
            Ok(ctx) => ctx,
            Err(e) => {
                send_or_return!(WorkerMessage::Error { message: e });
                return;
            }
        };

        send_or_return!(WorkerMessage::Progress {
            percent: 50,
            status: "Рендеринг шаблонов Jinja...".to_string(),
        });

        let files = match render(&context) {
            Ok(f) => f,
            Err(e) => {
                send_or_return!(WorkerMessage::Error { message: e });
                return;
            }
        };

        send_or_return!(WorkerMessage::Progress {
            percent: 90,
            status: "Запись файлов на диск...".to_string(),
        });

        if let Err(e) = create_project(&output_dir, files) {
            send_or_return!(WorkerMessage::Error { message: e });
            return;
        }

        // Успешное завершение
        send_or_return!(WorkerMessage::Done { output_dir });
    });

    receiver
}
