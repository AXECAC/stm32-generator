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

impl SpiBusRowModel {
    /// Формирует человекочитаемое описание параметров SPI-шины.
    fn subtitle(spi: &SpiConfig) -> String {
        let mode: &'static str = spi.mode.into();
        let miso = spi
            .miso
            .map(|p| p.variant_name().to_string())
            .unwrap_or_else(|| "не используется".to_string());
        let mosi = spi
            .mosi
            .map(|p| p.variant_name().to_string())
            .unwrap_or_else(|| "не используется".to_string());

        format!(
            "{} МГц, {}; SCK {}, MISO {}, MOSI {}",
            spi.frequency_mhz,
            mode,
            spi.sck.variant_name(),
            miso,
            mosi
        )
    }
}
