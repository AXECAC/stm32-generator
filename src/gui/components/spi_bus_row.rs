use crate::core::config::SpiConfig;
use crate::core::gpio::ChosenSpiBus;
use adw::prelude::*;
use relm4::factory::FactoryComponent;
use relm4::{adw, gtk};

/// Factory-модель строки сконфигурированной SPI-шины.
///
/// Используется для компактного отображения уже добавленных SPI шин и отправки
/// запроса на удаление выбранной шины.
#[derive(Debug)]
pub struct SpiBusRowModel {
    /// Идентификатор SPI-шины, к которой относится строка.
    bus: ChosenSpiBus,
    /// Заголовок строки, отображаемый в UI.
    title: String,
    /// Краткое описание параметров SPI-шины.
    subtitle: String,
}

/// Исходящие сообщения строки SPI.
#[derive(Debug)]
pub enum SpiBusRowOutput {
    /// Пользователь нажал кнопку удаления для указанной SPI-шины.
    Remove(ChosenSpiBus),
}
