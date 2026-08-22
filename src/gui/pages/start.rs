use std::sync::{Arc, RwLock};

use adw::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, adw, gtk};

use crate::core::board::{TargetBoard, TargetBoardId};
use crate::core::config::Config;
use crate::core::gpio::TargetMcu;
use crate::gui::components::forms::ComboField;

/// Состояние страницы выбора стартовой платы.
pub struct StartPageModel {
    /// Глобальная конфигурация приложения.
    pub(crate) config: Arc<RwLock<Config>>,
    /// Типизированное состояние выпадающего списка плат.
    board: ComboField<TargetBoard>,
}

/// Входящие сообщения стартовой страницы.
#[derive(Debug)]
pub enum StartPageInput {
    /// Вкладка стала активной; нужно перечитать свежий [`Config`].
    UpdateConfig,
    /// Пользователь выбрал плату по индексу.
    BoardSelected(usize),
}

#[relm4::component(pub)]
impl SimpleComponent for StartPageModel {
    type Init = Arc<RwLock<Config>>;
    type Input = StartPageInput;
    type Output = ();

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
                            set_model: Some(&model.board.model),

                            #[watch]
                            set_selected: model.board.selected_idx as u32,

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
        let black_pill = TargetBoard::try_new(TargetBoardId::BlackPill, TargetMcu::StmF401)
            .expect("Black Pill / STM32F401 must be a supported platform");
        let board_items = vec![black_pill];
        let board_labels = board_items
            .iter()
            .map(TargetBoard::name)
            .collect::<Vec<_>>();
        let board_label_refs = board_labels.iter().map(String::as_str).collect::<Vec<_>>();

        let mut board = ComboField::new(board_items, &board_label_refs);
        let current_board = init.read().unwrap().board;
        board.selected_idx = board
            .items
            .iter()
            .position(|board| *board == current_board)
            .unwrap_or(0);

        let model = StartPageModel {
            config: init,
            board,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            StartPageInput::UpdateConfig => {
                let current_board = self.config.read().unwrap().board;
                self.board.selected_idx = self
                    .board
                    .items
                    .iter()
                    .position(|b| *b == current_board)
                    .unwrap_or(0);
            }
            StartPageInput::BoardSelected(idx) => {
                if self.board.selected_idx == idx {
                    return;
                }

                let Some(board) = self.board.items.get(idx).copied() else {
                    self.board.clamp_selected();
                    return;
                };

                self.board.selected_idx = idx;

                let mut config = self.config.write().unwrap();
                if board != config.board {
                    // Сбрасываем конфиг при смене платы
                    *config = Config::new(board);
                }
            }
        }
    }
}
