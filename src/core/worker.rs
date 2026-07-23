use crossbeam_channel::Receiver;

use crate::core::{config::Config, errors::GeneratorError};
use std::path::PathBuf;

use crate::core::generator::{context::TemplateContext, render, writer::create_project};

const FAILED_SEND_ERR_MES: &str = "Worker failed to send progress message (receiver disconnected)";

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
        sender
            .send(WorkerMessage::Progress {
                percent: 10,
                status: "Сборка контекста шаблона...".to_string(),
            })
            .expect(FAILED_SEND_ERR_MES);

        let project_name = output_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let context = match TemplateContext::from_config(&config, project_name) {
            Ok(ctx) => ctx,
            Err(e) => {
                sender
                    .send(WorkerMessage::Error { message: e })
                    .expect(FAILED_SEND_ERR_MES);
                return;
            }
        };

        sender
            .send(WorkerMessage::Progress {
                percent: 50,
                status: "Рендеринг шаблонов Jinja...".to_string(),
            })
            .expect(FAILED_SEND_ERR_MES);

        let files = match render(&context) {
            Ok(f) => f,
            Err(e) => {
                sender
                    .send(WorkerMessage::Error { message: e })
                    .expect(FAILED_SEND_ERR_MES);
                return;
            }
        };

        sender
            .send(WorkerMessage::Progress {
                percent: 90,
                status: "Запись файлов на диск...".to_string(),
            })
            .expect(FAILED_SEND_ERR_MES);

        if let Err(e) = create_project(&output_dir, files) {
            sender
                .send(WorkerMessage::Error { message: e })
                .expect(FAILED_SEND_ERR_MES);
            return;
        }

        // Успешное завершение
        sender
            .send(WorkerMessage::Done { output_dir })
            .expect(FAILED_SEND_ERR_MES);
    });

    receiver
}
