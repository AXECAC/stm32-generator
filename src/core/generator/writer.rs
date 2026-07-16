use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::errors::GeneratorError;
use crate::core::generator::{Code, ProjectPath};

/// Записывает отрендеренные файлы проекта на жесткий диск.
///
/// - `target_dir` - путь к корневой директории генерируемого проекта.
/// - `files` - словарь, где ключ это путь к файлу в проекте, а значение - код файла.
pub fn create_project(
    target_dir: &Path,
    files: HashMap<ProjectPath, Code>,
) -> Result<(), GeneratorError> {
    for (file_path, content) in files {
        // Формируем полный путь к файлу
        let full_path = target_dir.join(&file_path);

        // Создаем директорию, если её нет
        if let Some(parent) = full_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        // Записываем исходный код в файл
        fs::write(full_path, content)?;
    }

    Ok(())
}
