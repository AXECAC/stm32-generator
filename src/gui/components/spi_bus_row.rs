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

#[relm4::factory(pub)]
impl FactoryComponent for SpiBusRowModel {
    /// Исходная конфигурация SPI-шины для создания строки.
    type Init = SpiConfig;
    /// Входящие сообщения factory-строки.
    type Input = ();
    /// Исходящие сообщения factory-строки.
    type Output = SpiBusRowOutput;
    /// Асинхронные команды factory-строкой не используются.
    type CommandOutput = ();
    /// Родительский GTK-контейнер, в который добавляются строки.
    type ParentWidget = adw::PreferencesGroup;

    view! {
        adw::ActionRow {
            set_title: self.title.as_str(),
            set_subtitle: self.subtitle.as_str(),

            add_suffix = &gtk::Button {
                set_label: "Удалить",
                add_css_class: "destructive-action",
                set_valign: gtk::Align::Center,

                connect_clicked[sender, bus = self.bus] => move |_| {
                    if let Err(e) = sender.output(SpiBusRowOutput::Remove(bus)) {
                        log::error!("Не удалось отправить Remove из SpiBusRowModel: {:?}", e);
                    }
                }
            }
        }
    }

    /// Создаёт модель строки из доменной конфигурации SPI.
    fn init_model(
        init: Self::Init,
        _idx: &relm4::factory::DynamicIndex,
        _sender: relm4::factory::FactorySender<Self>,
    ) -> Self {
        let title = format!("Шина {}", init.bus.variant_name());
        let subtitle = Self::subtitle(&init);

        Self {
            bus: init.bus,
            title,
            subtitle,
        }
    }
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
