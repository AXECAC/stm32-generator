use crate::core::board::TargetBoard;
use crate::core::config::Config;
use crate::core::gpio::TargetMcu;
use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};

pub struct StartPageModel {
    pub(crate) config: Config,
    pub(crate) boards: Vec<TargetBoard>,
}

#[derive(Debug)]
pub enum StartPageInput {
    UpdateConfig(Config),
    BoardSelected(usize),
}

#[derive(Debug)]
pub enum StartPageOutput {
    ConfigChanged(Config),
}

#[relm4::component(pub)]
impl SimpleComponent for StartPageModel {
    type Init = Config;
    type Input = StartPageInput;
    type Output = StartPageOutput;

    view! {
        adw::StatusPage {
            set_title: "STM32 Generator",
            set_description: Some("Генератор прошивок для микроконтроллеров STM32.\nВыберите вашу платформу для начала работы."),
            set_icon_name: Some("emblem-system-symbolic"),
            set_vexpand: true,

            #[wrap(Some)]
            set_child = &adw::Clamp {
                set_maximum_size: 450,
                set_tightening_threshold: 300,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,

                    adw::PreferencesGroup {
                        set_title: "Выбор платформы",
                        set_description: Some("Внимание: при смене микроконтроллера вся текущая конфигурация будет сброшена!"),

                        #[name = "board_combo"]
                        adw::ComboRow {
                            set_title: "Отладочная плата / MCU",
                            set_subtitle: "Выберите чип для вашего проекта",

                            set_model: Some(&{
                                let names: Vec<String> = model.boards.iter().map(|b| b.name()).collect();
                                gtk::StringList::new(&names.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                            }),

                            #[watch]
                            set_selected: model.boards.iter().position(|b| *b == model.config.board).unwrap_or(0) as u32,

                            connect_selected_notify[sender] => move |row| {
                                sender.input(StartPageInput::BoardSelected(row.selected() as usize));
                            }
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_halign: gtk::Align::Center,
                        set_spacing: 12,

                        // Можно будет добавить кнопку "Продолжить" для переключения таба
                        gtk::Label {
                            set_label: "Перейдите на вкладку «Пины» на панели сверху для настройки.",
                            add_css_class: "dim-label",
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let boards = vec![TargetBoard::BlackPill(TargetMcu::StmF401)];

        let model = StartPageModel {
            config: init,
            boards,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            StartPageInput::UpdateConfig(cfg) => {
                self.config = cfg;
            }
            StartPageInput::BoardSelected(idx) => {
                if let Some(board) = self.boards.get(idx)
                    && *board != self.config.board
                {
                    // Сбрасываем конфиг при смене платы
                    let new_config = Config::new(*board);
                    self.config = new_config.clone();
                    if let Err(e) = sender.output(StartPageOutput::ConfigChanged(new_config)) {
                        log::error!("Не удалось отправить ConfigChanged из StartPageModel: {:?}", e);
                    }
                }
            }
        }
    }
}
