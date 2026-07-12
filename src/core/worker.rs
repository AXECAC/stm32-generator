use crossbeam_channel::Receiver;

use crate::core::{config::Config, errors::GeneratorError};
use std::path::PathBuf;

/// Сообщения, в GUI о процессе генерации проекта
#[derive(Debug, Clone)]
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
        // --- ЗАГЛУШКА ДЛЯ РАЗРАБОТКИ GUI ---

        let _ = sender.send(WorkerMessage::Progress {
            percent: 10,
            status: "Подготовка шаблонов...".to_string(),
        });
        std::thread::sleep(std::time::Duration::from_millis(500));

        let _ = sender.send(WorkerMessage::Progress {
            percent: 50,
            status: "Генерация кода (main.rs)...".to_string(),
        });
        std::thread::sleep(std::time::Duration::from_millis(500));

        let _ = sender.send(WorkerMessage::Progress {
            percent: 90,
            status: "Сохранение файлов на диск...".to_string(),
        });
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Симуляция успешного завершения
        let _ = sender.send(WorkerMessage::Done { output_dir });

        // Симуляция ошибки (для теста в GUI):
        // let _ = sender.send(WorkerMessage::Error {
        //     message: "Отказано в доступе при записи в папку".to_string()
        // });
    });

    receiver
}
