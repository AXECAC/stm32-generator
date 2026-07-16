use crossbeam_channel::Receiver;

use crate::core::{config::Config, errors::GeneratorError};
use std::path::PathBuf;

use crate::core::generator::{context::TemplateContext, render, writer::create_project};

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
        let _ = sender.send(WorkerMessage::Progress {
            percent: 10,
            status: "Сборка контекста шаблона...".to_string(),
        });

        let context = match TemplateContext::from_config(&config) {
            Ok(ctx) => ctx,
            Err(e) => {
                let _ = sender.send(WorkerMessage::Error { message: e });
                return;
            }
        };

        let _ = sender.send(WorkerMessage::Progress {
            percent: 50,
            status: "Рендеринг шаблонов Jinja...".to_string(),
        });

        let files = match render(&context) {
            Ok(f) => f,
            Err(e) => {
                let _ = sender.send(WorkerMessage::Error { message: e });
                return;
            }
        };

        let _ = sender.send(WorkerMessage::Progress {
            percent: 90,
            status: "Запись файлов на диск...".to_string(),
        });

        if let Err(e) = create_project(&output_dir, files) {
            let _ = sender.send(WorkerMessage::Error { message: e });
            return;
        }

        // Успешное завершение
        let _ = sender.send(WorkerMessage::Done { output_dir });
    });

    receiver
}
