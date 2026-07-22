use serde::Serialize;
pub mod f4;

/// Предоставляет абстракцию для графического интерфейса над специфичными для МК режимами пинов.
///
/// Трейт позволяет пользовательскому интерфейсу (UI) динамически отрисовывать доступные
/// для выбора режимы и их дополнительные свойства, ничего не зная о внутреннем устройстве
/// конкретного микроконтроллера (например, какие именно скорости или виды подтяжки у него есть).
pub trait PinModeUiInfo {
    /// Возвращает список базовых режимов пина (например, `["Input", "Output"]`).
    ///
    /// Используется графическим интерфейсом для формирования основного списка выбора
    /// направления/функции работы вывода.
    fn mode_variants(&self) -> Vec<&'static str>;

    /// Возвращает индекс текущего базового режима из списка [`mode_variants`](Self::mode_variants).
    ///
    /// Позволяет инициализировать элементы управления актуальным состоянием.
    fn current_mode_index(&self) -> usize;

    /// Устанавливает новый базовый режим по его индексу из [`mode_variants`](Self::mode_variants).
    ///
    /// Изменение базового режима может полностью изменить набор доступных
    /// дополнительных свойств, возвращаемых методом [`properties`](Self::properties).
    fn set_mode_index(&mut self, idx: usize);

    /// Возвращает список зависимых свойств для детальной настройки текущего режима.
    ///
    /// Каждое свойство описывается кортежем:
    /// - Название параметра (например, `"Скорость"`, `"Тип выхода"`).
    /// - Список возможных значений (например, `["Low", "Medium", "High"]`).
    /// - Индекс текущего выбранного значения.
    fn properties(&self) -> Vec<(&'static str, Vec<&'static str>, usize)>;

    /// Изменяет значение одного из дополнительных свойств.
    ///
    /// Аргумент `prop_idx` соответствует индексу свойства в векторе, возвращаемом
    /// методом [`properties`](Self::properties), а `variant_idx` - индексу нового значения
    /// в списке вариантов данного свойства.
    fn set_property(&mut self, prop_idx: usize, variant_idx: usize);
}

macro_rules! define_mcus {
    (
        $(
            $variant:ident {
                pin_type: $pin_type:ty,
                mode_type: $mode_type:ty,
                spi_bus_type: $spi_bus_type:ty,
                family: $family:expr,
                hal_version: $hal_version:expr,
                feature: $feature:expr $(,)?
            }
        ),* $(,)?
    ) => {
        /// Идентификатор микроконтроллера
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
        pub enum TargetMcu {
            $( $variant ),*
        }

        impl TargetMcu {
            /// Получить все пины для данного микроконтроллера
            pub fn all_pins(&self) -> Vec<ChosenPin> {
                match self {
                    $(
                        Self::$variant => {
                            <$pin_type as strum::VariantNames>::VARIANTS.iter().map(|v| {
                                ChosenPin::$variant(<$pin_type as std::str::FromStr>::from_str(v).unwrap())
                            }).collect()
                        }
                    ),*
                }
            }
        }

        /// Пин
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::IntoStaticStr, Serialize)]
        pub enum ChosenPin {
            $( $variant($pin_type) ),*
        }

        impl ChosenPin {
            pub fn mcu_family(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $family ),*
                }
            }

            pub fn hal_version(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $hal_version ),*
                }
            }

            pub fn hal_feature(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $feature ),*
                }
            }

            pub fn variant_name(&self) -> &'static str {
                match self {
                    $( Self::$variant(p) => p.into() ),*
                }
            }

            pub fn default_mode(&self) -> ChosenPinWithMode {
                match self {
                    $( Self::$variant(p) => ChosenPinWithMode::$variant(*p, <$mode_type>::default()) ),*
                }
            }
        }

        impl From<ChosenPinWithMode> for ChosenPin {
            fn from(cur_pin: ChosenPinWithMode) -> Self {
                match cur_pin {
                    $( ChosenPinWithMode::$variant(pin, _) => Self::$variant(pin) ),*
                }
            }
        }

        /// Пин + режим
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::IntoStaticStr, Serialize)]
        pub enum ChosenPinWithMode {
            $( $variant($pin_type, $mode_type) ),*
        }

        impl ChosenPinWithMode {
            pub fn pin(&self) -> ChosenPin {
                match self {
                    $( Self::$variant(pin, _) => ChosenPin::$variant(*pin) ),*
                }
            }

            pub fn template_vars(&self) -> (&'static str, bool, Option<&'static str>) {
                match self {
                    $( Self::$variant(_, mode) => mode.template_vars() ),*
                }
            }
        }

        impl PinModeUiInfo for ChosenPinWithMode {
            fn mode_variants(&self) -> Vec<&'static str> {
                match self {
                    $( Self::$variant(_, mode) => mode.mode_variants() ),*
                }
            }

            fn current_mode_index(&self) -> usize {
                match self {
                    $( Self::$variant(_, mode) => mode.current_mode_index() ),*
                }
            }

            fn set_mode_index(&mut self, idx: usize) {
                match self {
                    $( Self::$variant(_, mode) => mode.set_mode_index(idx) ),*
                }
            }

            fn properties(&self) -> Vec<(&'static str, Vec<&'static str>, usize)> {
                match self {
                    $( Self::$variant(_, mode) => mode.properties() ),*
                }
            }

            fn set_property(&mut self, prop_idx: usize, variant_idx: usize) {
                match self {
                    $( Self::$variant(_, mode) => mode.set_property(prop_idx, variant_idx) ),*
                }
            }
        }

        /// Шина SPI
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::IntoStaticStr, Serialize)]
        pub enum ChosenSpiBus {
            $( $variant($spi_bus_type) ),*
        }
    };
}

define_mcus! {
    StmF401 {
        pin_type: crate::core::gpio::f4::f401::StmF401Pin,
        mode_type: crate::core::gpio::f4::StmF4PinMode,
        spi_bus_type: crate::core::gpio::f4::f401::StmF401SpiBus,
        family: "stm32f4",
        hal_version: "0.23.0",
        feature: "stm32f401",
    },
}
