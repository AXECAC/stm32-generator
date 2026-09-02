# План добавления Blue Pill и генерации TCP-проекта W5500

Дата анализа: 2026-08-15

Документ составлен по текущему состоянию `stm32-generator`, всем локальным
`.md`-документам и первому TCP-коммиту репозитория
[`AXECAC/blue_pill_ethernet`](https://github.com/AXECAC/blue_pill_ethernet/commit/b2e0f4a2c2b296f256c81d96b092601daaea0cdb).

Из внешнего репозитория использован именно коммит `b2e0f4a` (`Init commit:
blue-pill TCP`). Следующий коммит `70f7fb9` меняет только `src/main.rs`, заменяя
TCP на UDP, поэтому UDP в этот план не входит.

Код проекта пока не менялся. Единственное новое изменение, предусмотренное
этим запросом, — настоящий файл с планом.

## 1. Краткий вывод

Абстракции в текущем проекте в целом подходят для добавления STM32F1:

- `Config` не привязан к F4 и уже хранит пины, SPI и W5500 через обобщённые
  `ChosenPin`, `ChosenSpiBus` и `Peripheral`;
- GUI получает возможности MCU через `TargetBoard`, `TargetMcu` и
  `PinModeUiInfo`, а не через прямые проверки F4 в каждой форме;
- генератор уже разделён на контекст, регистрацию шаблонов, шаблоны и writer;
- TCP-модель W5500 уже есть в core и в GUI.

Главная работа будет не в переделке GUI, а в корректном расширении core и
генератора. При этом недостаточно просто добавить `f1/*.rs`: нужно также сделать
параметрами генератора target triple, chip для probe-rs, карту памяти, API GPIO,
инициализацию RCC/AFIO и способ сборки SPI.

Минимальная оценка GUI: **три файла с небольшими изменениями** —
`src/gui/pages/start.rs`, `src/gui/pages/pins.rs` и
`src/gui/components/chip_canvas.rs`. В `app.rs`, формах SPI/W5500, страницах
SPI/периферии и странице генерации архитектурных изменений не требуется.

Отдельный функциональный вопрос — прикладная логика TCP. Эталонный проект не
просто поднимает TCP-сервер: он управляет LED по байтам `0`/`1` и отправляет
ответ. Текущая модель конфигурации этого поведения не описывает, а текущий
шаблон оставляет его в виде TODO — это нельзя считать решённым добавлением
одних шаблонов F1.

## 2. Что делает эталонный TCP-проект

Первый коммит внешнего проекта — уже готовый `no_std`-проект с единственным
`src/main.rs`, а не генератор. Его аппаратная схема:

| Назначение | Blue Pill STM32F103 |
|---|---|
| W5500 reset | `PA3` |
| W5500 CS | `PA4` |
| SPI1 SCK | `PA5` |
| SPI1 MISO | `PA6` |
| SPI1 MOSI | `PA7` |
| LED | `PB2` |
| TCP socket | `Sn0` |
| TCP port | `8080` |

Логика цикла в `b2e0f4a`:

1. Получить `cortex_m::Peripherals` и `stm32f1xx_hal::pac::Peripherals`.
2. Настроить RCC, заморозить clocks через `FLASH`, создать `Delay`.
3. Разделить GPIOA и GPIOB через `rcc.apb2`.
4. Настроить LED, reset и CS как push-pull output.
5. Создать SPI1 в Mode 0 на частоте `2 MHz`.
6. Выполнить аппаратный reset W5500 с задержками по 50 ms.
7. Создать `embedded_hal_bus::spi::ExclusiveDevice`.
8. Записать MAC, IP, subnet и gateway.
9. При `Closed` вызвать `tcp_listen`.
10. При `Established`/`CloseWait` прочитать до 64 байт:
    - `0` выключает LED и отвечает `LED OFF\n`;
    - `1` включает LED и отвечает `LED ON\n`.
11. При `CloseWait` и `TimeWait` закрыть сокет через макрос.

В репозитории есть детали, которые нельзя копировать бездумно:

- комментарий говорит о 10 MHz, а код использует 2 MHz;
- сетевой интерфейс в `justfile` жёстко зашит под конкретный компьютер;
- прошивка выполняется через `stm32flash` и serial bootloader;
- LED и пины W5500 захардкожены прямо в `main.rs`.

В генераторе частота, пины, сеть и сокет должны браться из `Config`. Поведение
LED должно быть либо явно частью модели приложения, либо оставаться честным
пользовательским extension point, но не появляться как скрытая магия по имени
пина.

## 3. Что уже есть в текущем проекте

Фактический путь генерации сейчас такой:

```text
Config
  -> TemplateContext::from_config
  -> minijinja Environment
  -> render(main.rs.j2 и инфраструктурных файлов)
  -> writer::create_project
  -> worker::start_generation
  -> RunPageModel
```

В `TemplateContext` уже есть данные для семейства MCU/HAL, используемых портов,
пользовательских GPIO, SPI с optional MISO/MOSI, нескольких W5500 с уникальными
ID, TCP server, MAC/IP/subnet/gateway/socket и feature-флагов W5500.

В текущем F4-пути корректно разделены общие и специфичные части:

- `main.rs.j2` — orchestration;
- `blocks/mcu/stm32f4/*` — PAC/HAL/GPIO/SPI;
- `blocks/peripherals/W5500/*` — W5500;
- `context.rs` — преобразование доменной модели в простой Jinja-контекст.

Фактические F4-зависимости пока находятся в следующих местах:

1. `main.rs.j2` включает только `blocks/mcu/stm32f4/*`.
2. `generator/mod.rs` регистрирует только F4-шаблоны.
3. `templates.rs` не знает о F1-шаблонах.
4. `.cargo/config.toml.j2` имеет defaults для `thumbv7em` и STM32F401.
5. `memory.x.j2` имеет defaults `256K/64K`, не подходящие F103C8T6.
6. `justfile.j2` использует Black Pill/DFU-сценарий.
7. `GpioPinCtx` не содержит `CRL/CRH` для F1.
8. `PinCtx` не содержит F1-регистр конфигурации.
9. `build_w5500_ctx` содержит match только для `ChosenSpiBus::StmF401`.
10. `logic_single.rs.j2` не содержит LED-обработку из эталона, а оставляет TODO.

Локальный коммит `2fe0813` уже показывает один набросок F1-поддержки. Его
полезно учитывать, но не принимать автоматически: там target-specific значения
и `crl/crh` добавлены прямо в текущий контекст, а часть решений нужно проверить
на реальном API HAL и на сгенерированном проекте.

## 4. Рекомендуемая последовательность реализации

### Этап 0. Зафиксировать минимальный сценарий приёмки

До изменений зафиксировать fixture, соответствующий эталонному проекту:

- `TargetBoard::BluePill(TargetMcu::StmF103)`;
- LED `PB2` как пользовательский output;
- SPI1: SCK `PA5`, MISO `PA6`, MOSI `PA7`, Mode 0, 2 MHz;
- W5500: CS `PA4`, RST `PA3`;
- MAC `02:00:00:11:22:33`, IP `192.168.1.50`, subnet `/24`, gateway
  `192.168.1.1`;
- TCP server на socket 0, port 8080.

Fixture нужна как golden test для контекста и как основа ручной проверки
сгенерированного `main.rs`. Она не должна означать, что эти пины навсегда
зашиваются в генератор.

### Этап 1. Добавить доменную модель STM32F1

Создать `src/core/gpio/f1/` по аналогии с F4:

- `f103.rs` — `StmF103Pin` и `StmF103SpiBus`;
- `mod.rs` — `StmF1PinMode`, input/output modes и скорости.

Для первого целевого проекта достаточно выставить SPI1. SPI2 можно добавить
только при наличии и проверке отдельного шаблонного пути. Нельзя объявлять шину
доступной в GUI, если генератор ещё не умеет её собрать.

`StmF1PinMode` должен реализовать тот же `PinModeUiInfo`, что и F4. Отличия F1:

- GPIO использует `CRL/CRH`, а не F4 MODER/OSPEEDR;
- скорости — 2, 10 и 50 MHz;
- AFIO/remap относится к периферии и не должен становиться ручным GPIO-режимом;
- SPI-пины должны настраиваться F1-специфичным SPI-шаблоном.

Затем добавить `StmF103` в macro `define_mcus!`. Благодаря этому автоматически
появятся `ChosenPin::StmF103`, `ChosenSpiBus::StmF103`, `all_pins()` и
`all_spi_buses()`.

### Этап 2. Добавить описание Blue Pill в `core/board.rs`

Добавить `TargetBoard::BluePill(TargetMcu::StmF103)` и реализовать для него
`mcu()`, имя, label чипа и точный список доступных GPIO/power-пинов Blue Pill.

Список пинов нужно сверить с корпусом STM32F103C8T6 и реальной распиновкой
платы. Нельзя просто переиспользовать Black Pill-порядок: canvas выводит пины в
порядке, который возвращает `build_pins()`.

Метаданные платы/MCU лучше централизовать одним источником истины:

```text
family       = stm32f1
hal_version  = 0.11.0
hal_feature  = stm32f103
target       = thumbv7m-none-eabi
probe_chip   = STM32F103C8T6
flash        = 64K
ram          = 20K
```

Сейчас эти значения можно разложить по match в `context.rs`, но для будущих
MCU лучше дать `TargetMcu`/`TargetBoard` метод вроде `toolchain_info()`.

### Этап 3. Расширить контракт `TemplateContext`

Контекст должен строиться от `config.board.mcu()`, а не определять семейство по
первому занятому пину. Последний вариант ломается на пустом конфиге и скрывает
связь с выбранной платой.

Добавить или централизовать:

- `target` — `thumbv7m-none-eabi` для F103;
- `chip` — `STM32F103C8T6`;
- `flash_origin`, `flash_length`, `ram_origin`, `ram_length`;
- `cr_reg` для каждого GPIO/SPI-пина (`crl` для 0..7, `crh` для 8..15);
- имя конструктора SPI (`spi1`, позднее `spi2`);
- F1-аргументы (`clocks`, `afio.mapr`, типы NoMiso/NoMosi).

Jinja не должен вычислять эти значения из строк. Rust-контекст должен выдавать
уже готовые значения, например:

```text
gpio pin:  { port: "a", pin_num: "3", cr_reg: "crl" }
spi bus:   { pac_bus: "SPI1", constructor: "spi1", ... }
mcu:       { family: "stm32f1", target: "thumbv7m-none-eabi", ... }
```

Важно убрать match `ChosenSpiBus::StmF401` из W5500-контекста и использовать
общий `variant_name()`/метод контекста. W5500 не должен знать, F1 или F4 лежит
под его SPI-шиной.

### Этап 4. Зарегистрировать F1-шаблоны и сделать выбор MCU динамическим

Добавить в `assets/templates/blocks/mcu/stm32f1/`:

- `imports.rs.j2`;
- `init.rs.j2`;
- `gpio.rs.j2`.

В `templates.rs` добавить `include_str!`, а в `generator/mod.rs` — регистрацию
всех трёх шаблонов.

В `main.rs.j2` использовать динамический include по семейству:

```text
blocks/mcu/{{ mcu_family }}/imports.rs.j2
blocks/mcu/{{ mcu_family }}/init.rs.j2
blocks/mcu/{{ mcu_family }}/gpio.rs.j2
```

Такой выбор лучше длинного `if mcu_family == ...` в каждом месте: основной
шаблон остаётся orchestration-слоем, а добавление новых семейств не разрастается
в один монолитный файл. Неизвестное семейство должно давать ошибку Rust до
записи проекта, а не пустой `main.rs`.

### Этап 5. Реализовать F1 init/GPIO/SPI

`stm32f1/init.rs.j2` должен повторять проверенную последовательность из
эталона:

1. `cp`/`dp`;
2. `dp.RCC.constrain()`;
3. `dp.FLASH.constrain()`;
4. `rcc.cfgr.freeze(&mut flash.acr)`;
5. `Delay::new(..., clocks.sysclk().raw())`;
6. `dp.AFIO.constrain()`.

В `gpio.rs.j2` нужно:

- разделять порты через `rcc.apb2`;
- передавать `&mut gpioX.crl` или `&mut gpioX.crh` в GPIO-конструкторы;
- создавать пользовательские GPIO с учётом F1 method name;
- создавать SPI-пины в нужных F1 режимах;
- передавать `&mut afio.mapr`, mode, частоту и `clocks` в `Spi::spi1`.

Для optional MISO/MOSI нельзя полагаться на F4-кортеж `(Some(...), None, ...)`.
Нужно сгенерировать типы, которые ожидает F1 HAL (`NoMiso`/`NoMosi`), либо
ограничить первую версию полным SPI1-подключением, как в эталоне. Это должно
быть решено в контексте, а не сложными условиями в Jinja.

### Этап 6. Разделить MCU-specific pin setup W5500 и общую TCP-логику

Сетевые операции W5500 и TCP state machine общие для F1/F4. Отличается прежде
всего настройка CS/RST:

- F4: `into_push_pull_output()`;
- F1: `into_push_pull_output(&mut gpioX.crl/crh)`.

Рекомендуемый вариант — оставить общий W5500 init, но вынести создание CS/RST
в MCU hook, например отдельный `blocks/mcu/<family>/w5500_pins.rs.j2`. Это
лучше, чем добавлять много условий `if mcu_family == ...` в
`blocks/peripherals/W5500/init.rs.j2`.

Общими оставить `ExclusiveDevice`, MAC/IP/subnet/gateway, TCP constants,
`close_socket`, импорты и `logic_single`/`logic_bridge`.

Проверить нужно также время reset. Эталон использует 50 ms, текущий шаблон —
1 ms. Для первого порта я бы выбрал консервативное именованное значение 50 ms
или вынес его в metadata W5500; копировать разные магические литералы в F1 и F4
не стоит.

### Этап 7. Сделать инфраструктурные шаблоны target-aware

Без этого сгенерированный F1 `main.rs` может быть правильным, но проект не
соберётся или будет запускаться не тем runner'ом.

Изменения нужны в:

- `.cargo/config.toml.j2` — target и chip из контекста;
- `memory.x.j2` — 64K Flash и 20K RAM для STM32F103C8T6;
- `Cargo.toml.j2` — `stm32f1xx-hal`, feature `stm32f103`, версия HAL;
- `justfile.j2` — board-specific flash recipe.

Для `justfile` нужно заранее выбрать стратегию:

1. Сохранить поведение эталона для Blue Pill — `stm32flash` через serial
   bootloader.
2. Унифицировать все платы через probe-rs/ST-Link.

Я бы оставил runner в `.cargo/config.toml` через probe-rs для единообразного
debug/run, а рецепт прошивки Blue Pill сделал отдельным и явно подписанным
`stm32flash`. Имя serial-порта и сетевого интерфейса нельзя зашивать как
рабочую гарантию: их лучше вынести в переменные/комментарии настройки.

### Этап 8. Отдельно решить прикладную TCP-логику

Текущая `SocketMode::TcpServer { port, socket_num }` описывает транспорт, но
не описывает, что делать с полученными байтами. Поэтому есть два корректных
варианта.

#### Вариант A — первый релиз оставляет hook

Сохранить текущий подход `logic_single.rs.j2`: создать рабочий TCP server и
оставить в loop понятный пользовательский блок. Тогда генератор поддерживает
транспорт эталона, но не обещает автоматически повторить LED-поведение.

Плюс: почти не меняются core и GUI.

Минус: сгенерированный проект не является поведенчески точной копией первого
коммита.

#### Вариант B — явно добавить описание приложения

Если цель — получать проект с управлением LED, добавить в модель отдельную
прикладную конфигурацию, например:

```text
TcpApplication::LedControl {
    pin: ChosenPin,
    off_command: u8,
    on_command: u8,
}
```

`W5500Config`/`SocketMode` тогда хранит не только port/socket, но и обработчик.
GUI должен дать выбрать GPIO output и команды. В контекст попадают имя
переменной GPIO и команды, а шаблон генерирует код из данных.

Это архитектурно правильно, если приложение будет развиваться, но это уже не
«минимальная добавка Blue Pill»: потребуются core-конфигурация, форма,
валидация, отображение и тесты.

#### Что не следует делать

Не следует молча искать GPIO с alias `led` и считать, что это всегда PB2. Такая
эвристика случайно повторит demo на одной конфигурации, но сломает принцип
явной конфигурации и будет непредсказуемой для пользователя.

Для первого этапа я рекомендую вариант A, а точную LED-логику оформить отдельной
задачей после стабилизации F1/TCP transport. Golden fixture всё равно можно
использовать для ручной проверки и для последующего варианта B.

## 5. Оценка изменений в GUI

### Обязательные изменения

| Файл | Что поменять | Почему |
|---|---|---|
| `src/gui/pages/start.rs` | Добавить `TargetBoard::BluePill(TargetMcu::StmF103)` в `board_items` | Иначе Blue Pill нельзя выбрать |
| `src/gui/pages/pins.rs` | При `UpdateConfig` отправлять в canvas новый `board.chip_label()` | Список пинов обновится, но label старого чипа может остаться |
| `src/gui/components/chip_canvas.rs` | Добавить `UpdateChipLabel(String)` и обновление `DrawingState.chip_label` | Canvas живёт дольше страницы и не знает о смене board сам |

Это примерно 10–25 строк изменения, если не делать параллельный рефакторинг.
В `pins.rs` менять логику режимов GPIO, блокировок или echo-защиты не нужно.

### Что, скорее всего, менять не нужно

- `src/gui/app.rs`: общий `Arc<RwLock<Config>>` уже подходит; Black Pill можно
  оставить default.
- `src/gui/components/forms/pin_mode.rs`: работает через `PinModeUiInfo`.
- `src/gui/components/forms/spi.rs`: использует `ChosenPin`/`ChosenSpiBus`.
- `src/gui/pages/spi.rs`: список шин берётся из `board.mcu().all_spi_buses()`.
- `src/gui/components/forms/w5500.rs`: выбирает общие `ChosenPin` и SPI.
- `src/gui/pages/peripherals.rs`: конфигурация W5500 уже board-neutral.
- `src/gui/pages/run.rs`: генератор получает клон всего `Config`.
- `src/gui/components/spi_bus_row.rs` и `peripheral_row.rs`: используют
  `variant_name()`.

### Проверка canvas

Текущий canvas вычисляет число пинов динамически через `build_pins()` и
`div_ceil(4)`, поэтому отдельная геометрия для Blue Pill не обязательна.
Однако порядок power-пинов и компактность Blue Pill нужно визуально проверить.
Реалистичная распиновка по сторонам — отдельное улучшение canvas, не обязательная
часть F1/TCP-порта.

`RULES_CONTEXT.md` запрещает менять `pins.rs` без прямого указания. Поэтому при
фактической реализации это должен быть отдельный минимальный патч только для
синхронизации label canvas, без затрагивания остальной эталонной логики.

## 6. Что делать по-другому по сравнению с прямым копированием эталона

1. Не копировать весь `src/main.rs` в один Jinja-файл. Разнести MCU init, GPIO,
   SPI и W5500 hook по существующей структуре.
2. Не захардкодить PA3/PA4/PA5/PA6/PA7/PB2. Они должны прийти из `Config`;
   fixture использует эти значения только для проверки.
3. Не выводить `family` из первого занятого пина. Выбранная плата уже есть в
   `Config.board`.
4. Не помещать `CRL/CRH`, AFIO и `Spi::spi1` в core-конфиг как Rust-код. Core
   хранит физические сущности, context — подготовленные безопасные значения.
5. Не смешивать TCP и UDP на первом этапе. UDP-коммит — только будущий источник
   требований.
6. Не переносить в генератор hardcoded host network interface из `justfile`.
7. Не делать глобальные GUI-уведомления ради Blue Pill: сохраняются
   `Arc<RwLock<Config>>` и ленивый `UpdateConfig`.
8. Не добавлять новый GUI-путь для F1 GPIO. Общий `PinModeUiInfo` должен скрыть
   различия F1/F4.
9. Не объявлять поддержку SPI2, пока для него не проверены pin modes, AFIO,
   API HAL и generated build.

## 7. План тестирования и проверки

### Core

- `StmF103Pin::variant_name()` и `all_pins()` возвращают ожидаемые пины;
- `TargetMcu::StmF103.all_spi_buses()` соответствует реально поддержанным
  шаблонам;
- F1 mode UI корректно переключает Input/Output и свойства скорости/типа;
- конфликт пинов работает одинаково для F4 и F1;
- W5500 и запрет удаления занятой SPI-шины не зависят от семейства MCU.

### Контекст и Minijinja

- fixture выдаёт `stm32f1`, `thumbv7m-none-eabi`, F103 chip, `64K/20K`;
- `crl` получается для PA3..PA7 и PB2;
- `used_ports` содержит только реально используемые `a` и `b`;
- SPI-контекст содержит `SPI1`, Mode 0, частоту из Config и F1 constructor;
- W5500 не падает на `ChosenSpiBus::StmF103`;
- в F1-проекте не остаются F4 defaults.

### Сгенерированный проект

1. Сгенерировать fixture во временную директорию.
2. Проверить `Cargo.toml`, `.cargo/config.toml`, `memory.x`, `justfile`.
3. Проверить отсутствие F4 PAC/HAL в F1 `main.rs`.
4. Установить `thumbv7m-none-eabi` и выполнить `cargo check`/сборку generated
   проекта с нужным target.
5. На плате проверить reset, SPI1, W5500, TCP listen, CloseWait и TimeWait.
6. Отдельно проверить выбранную прикладную семантику: hook или LED-control.

### GUI

- выбор Blue Pill меняет board в общем `Config`;
- после перехода на вкладку пинов отображаются правильные пины и label F103;
- SPI показывает только шины нового MCU;
- W5500 видит SPI и свободные пины F1;
- переходы вкладок не вызывают echo-loop после обновления ComboRow;
- генерация использует выбранный Blue Pill, а не стартовый Black Pill.

## 8. Итоговая последовательность коммитов

Я бы разбил работу так:

1. `core: add STM32F1/F103 pin modes and SPI identity`.
2. `core: add BluePill board layout and MCU toolchain metadata`.
3. `generator: make template context board-aware`.
4. `generator: add STM32F1 imports/init/GPIO/SPI templates`.
5. `generator: add F1-specific W5500 pin setup`.
6. `generator: make main/infrastructure templates target-aware`.
7. `gui: expose Blue Pill and refresh canvas chip label`.
8. `tests: add Blue Pill TCP golden fixture and generated-project checks`.
9. Отдельно, после решения прикладной модели: `generator/gui: add TCP LED
   application behavior`.

## 9. Вопрос перед реализацией

Нужно подтвердить один функциональный выбор:

- первый этап генерирует универсальный TCP server с пользовательским hook,
  как сейчас задумано в шаблонах;
- или уже первый этап должен генерировать точно LED-поведение из `b2e0f4a`, для
  чего потребуется явно расширить core/GUI модель прикладной логикой.

Сам порт STM32F1, TCP transport, SPI1, W5500 и минимальные GUI-изменения от
этого выбора не меняются; меняется только объём последнего прикладного этапа.
