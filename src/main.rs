use std::path::PathBuf;

use crate::core::{
    config::Config,
    worker::{WorkerMessage, start_generation},
};

mod core;
mod gui;

fn main() {
    let test_config = Config::new();
    let mut path = PathBuf::new();
    path.push("/home/aragami3070/projects/");
    let reciver = start_generation(test_config, path);
    'main: loop {
        while let Ok(msg) = reciver.try_recv() {
            match msg {
                WorkerMessage::Progress { percent, status } => {
                    println!("Status: {status}, percent: {percent}");
                }
                WorkerMessage::Done { output_dir } => {
                    println!("Успех! Сохранено в {:?}", output_dir);
                    break 'main;
                }
                WorkerMessage::Error { message } => {
                    println!("Ошибка: {}", message);
                    break 'main;
                }
            }
        }
    }
}
