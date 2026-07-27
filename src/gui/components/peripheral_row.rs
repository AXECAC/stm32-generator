use crate::core::config::PeripheralId;
use crate::core::peripherals::Peripheral;
use crate::core::peripherals::ethernet::w5500::SocketMode;
use adw::prelude::*;
use relm4::factory::FactoryComponent;
use relm4::{adw, gtk};

/// Factory-модель строки сконфигурированной периферии.
///
/// Используется для компактного отображения уже добавленных периферийных
/// устройств и отправки запроса на удаление выбранного устройства.
#[derive(Debug)]
pub struct PeripheralRowModel {
    /// Идентификатор периферии в глобальной конфигурации.
    id: PeripheralId,
    /// Заголовок строки, отображаемый в UI.
    title: String,
    /// Краткое описание настроек периферии.
    subtitle: String,
}

/// Исходящие сообщения строки периферии.
#[derive(Debug)]
pub enum PeripheralRowOutput {
    /// Пользователь нажал кнопку удаления для указанной периферии.
    Remove(PeripheralId),
}

#[relm4::factory(pub)]
impl FactoryComponent for PeripheralRowModel {
    /// Исходная конфигурация периферии для создания строки.
    type Init = (PeripheralId, Peripheral);
    /// Входящие сообщения factory-строки не используются.
    type Input = ();
    /// Исходящие сообщения factory-строки.
    type Output = PeripheralRowOutput;
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

                connect_clicked[sender, id = self.id] => move |_| {
                    if let Err(e) = sender.output(PeripheralRowOutput::Remove(id)) {
                        log::error!("Не удалось отправить Remove из PeripheralRowModel: {:?}", e);
                    }
                }
            }
        }
    }

    /// Создаёт модель строки из доменной конфигурации периферии.
    fn init_model(
        init: Self::Init,
        _idx: &relm4::factory::DynamicIndex,
        _sender: relm4::factory::FactorySender<Self>,
    ) -> Self {
        let (id, peripheral) = init;
        let title = Self::title(id, &peripheral);
        let subtitle = Self::subtitle(&peripheral);

        Self {
            id,
            title,
            subtitle,
        }
    }
}

impl PeripheralRowModel {
    /// Формирует заголовок строки периферии.
    fn title(id: PeripheralId, peripheral: &Peripheral) -> String {
        match peripheral {
            Peripheral::W5500(_) => format!("W5500 #{}", id.get()),
        }
    }

    /// Формирует человекочитаемое описание параметров периферии.
    fn subtitle(peripheral: &Peripheral) -> String {
        match peripheral {
            Peripheral::W5500(w5500) => {
                let socket = match w5500.socket_mode {
                    SocketMode::TcpServer { port, socket_num } => {
                        format!("TCP server, port {}, socket {}", port, socket_num)
                    }
                };

                format!(
                    "SPI {}, CS {}, RST {}; MAC {}, IP {}; {}",
                    w5500.spi_bus.variant_name(),
                    w5500.cs.variant_name(),
                    w5500.rst.variant_name(),
                    w5500.network.mac,
                    w5500.network.ip,
                    socket
                )
            }
        }
    }
}
