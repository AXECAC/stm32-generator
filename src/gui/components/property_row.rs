use adw::prelude::*;
use relm4::factory::FactoryComponent;
use relm4::{adw, gtk};

/// Абстракция для динамически создаваемой строки настроек конкретного свойства пина или периферии.
///
/// Используется в связке с `relm4::factory::FactoryVecDeque` для отрисовки
/// дополнительных параметров, зависящих от выбранного базового режима работы.
#[derive(Debug)]
pub struct PropertyRowModel {
    pub prop_idx: usize,
    pub title: String,
    pub variants: Vec<String>,
    pub selected: usize,
}

/// Входящие сообщения для строки дополнительных настроек.
///
/// Перечисление намеренно пустое, так как данный фабричный компонент является однонаправленным:
/// он лишь регистрирует выбор пользователя и отправляет результат родительскому компоненту,
/// не принимая никаких команд на обновление извне.
#[derive(Debug)]
pub enum PropertyRowInput {}

/// Исходящие сообщения от строки дополнительных настроек к родительскому компоненту.
#[derive(Debug)]
pub enum PropertyRowOutput {
    /// Отправляется при выборе пользователем нового варианта в выпадающем списке.
    /// Содержит индекс свойства (`prop_idx`) и индекс выбранного значения (`variant_idx`).
    SelectionChanged(usize, usize),
}

#[relm4::factory(pub)]
impl FactoryComponent for PropertyRowModel {
    type Init = (usize, String, Vec<String>, usize);
    type Input = PropertyRowInput;
    type Output = PropertyRowOutput;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        adw::ComboRow {
            set_title: self.title.as_str(),
            set_model: Some(&gtk::StringList::new(
                           &self.variants.iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>(),
                       )),
            set_selected: self.selected as u32,

            connect_selected_notify[sender, prop_idx = self.prop_idx] => move |row| {
                sender
                    .output(PropertyRowOutput::SelectionChanged(prop_idx, row.selected() as usize))
                    .expect("Failed to emit SelectionChanged output message from PropertyRowModel");
            }
        }
    }

    fn init_model(
        init: Self::Init,
        _idx: &relm4::factory::DynamicIndex,
        _sender: relm4::factory::FactorySender<Self>,
    ) -> Self {
        Self {
            prop_idx: init.0,
            title: init.1,
            variants: init.2,
            selected: init.3,
        }
    }
}
