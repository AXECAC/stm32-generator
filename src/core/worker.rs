use std::path::PathBuf;
use crate::core::errors::GeneratorError;

/// Сообщения, в GUI о процессе генерации проекта
#[derive(Debug, Clone)]
pub enum WorkerMessage {
    /// Промежуточный прогресс, от 0 до 100, и текстовое описание этапа
    Progress { percent: u32, status: String },
    /// Генерация успешно завершена
    Done { output_dir: PathBuf },
    /// Произошла фатальная ошибка
    Error {
        message: GeneratorError,
    },
}
