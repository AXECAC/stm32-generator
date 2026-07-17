#![no_std]
#![no_main]

// Обработчик паники. В случае фатальной ошибки исполнения программа войдет в бесконечный цикл.
use panic_halt as _;
use cortex_m_rt::entry;

// Инициализация базовых импортов для выбранного семейства МК
use cortex_m::delay::Delay;
use stm32f4xx_hal::{
    pac,
    prelude::*,
    spi::{Mode, Phase, Polarity, Spi},
};

// Импорты для модуля W5500
use w5500_hl::ll::{Registers, Sn, SocketStatus};
use w5500_hl::Tcp;
use w5500_ll::eh1::vdm::W5500;
use w5500_ll::net::{Eui48Addr, Ipv4Addr};

#[entry]
fn main() -> ! {
    // 1. Базовая инициализация МК (получение Peripherals, настройка RCC, SysTick/Delay)
        // Обобщенная низкоуровневая периферия
    let cp = cortex_m::Peripherals::take().unwrap();
    // Периферия stm32f4
    let dp = pac::Peripherals::take().unwrap();

    // Инициализация задержек (базовая тактовая частота 16MHz)
    let mut delay = Delay::new(cp.SYST, 16_000_000);

    // Инициализация модуля Reset and Clock Control (RCC)
    let mut rcc = dp.RCC.constrain();

    // 2. Инициализация портов (включение тактирования портов, настройка GPIO и SPI)
        // Включение тактирования (RCC) и разбивка используемых GPIO портов
    let gpioa = dp.GPIOA.split(&mut rcc);

    // Настройка пользовательских GPIO пинов

    // Настройка шин SPI
    // Пины для шины SPI1
    let sck_spi1 = gpioa.pa0.into_alternate();
    let miso_spi1 = gpioa.pa0.into_alternate();
    let mosi_spi1 = gpioa.pa0.into_alternate();

    let spi_mode_spi1 = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let spi1 = Spi::new(
        dp.SPI1,
        (Some(sck_spi1), Some(miso_spi1), Some(mosi_spi1)),
        spi_mode_spi1,
        10.MHz(),
        &mut rcc,
    );

    // 3. Инициализация периферии
        // Инициализация W5500 #0
    // Пины аппаратного сброса и Chip Select
    let mut w5500_rst_0 = gpioa.pa1.into_push_pull_output();
    let mut w5500_cs_0 = gpioa.pa1.into_push_pull_output();
    w5500_cs_0.set_high(); // Изначально CS в высоком уровне (неактивен)

    // Ручная перезагрузка чипа W5500
    w5500_rst_0.set_low();
    delay.delay_ms(1_u32);
    w5500_rst_0.set_high();
    delay.delay_ms(1_u32);

    let spi_device_0 = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(
        spi1,
        w5500_cs_0
    ).unwrap();
    let mut w5500_0 = W5500::new(spi_device_0);

    // Установка сетевых параметров
    let mac_addr_0 = Eui48Addr::new(0, 8, 220, 171, 205, 239);
    w5500_0.set_shar(&mac_addr_0).unwrap();

    let ip_addr_0 = Ipv4Addr::new(192, 168, 1, 100);
    w5500_0.set_sipr(&ip_addr_0).unwrap();

    let subnet_mask_0 = Ipv4Addr::new(255, 255, 255, 0);
    w5500_0.set_subr(&subnet_mask_0).unwrap();

    let gateway_ip_0 = Ipv4Addr::new(192, 168, 1, 29);
    w5500_0.set_gar(&gateway_ip_0).unwrap();


    // Выделяем сокет
    const TCP_SOCKET_0: Sn = Sn::Sn0;
    const SERVER_PORT_0: u16 = 8079;

    macro_rules! close_socket_0 {
        () => {
            w5500_0
                .set_sn_cr(TCP_SOCKET_0, w5500_ll::SocketCommand::Disconnect)
                .unwrap();
            delay.delay_ms(100_u32);
            w5500_0
                .set_sn_cr(TCP_SOCKET_0, w5500_ll::SocketCommand::Close)
                .unwrap();
        };
    }


    loop {
        // 4. Основной цикл (опрос периферии, обработка прерываний стейт-машины)
        
        
        // Чтение аппаратно-формируемого статуса сокета W5500 #0
        let status_0 = w5500_0
            .sn_sr(TCP_SOCKET_0)
            .unwrap()
            .unwrap_or(SocketStatus::Closed);

        match status_0 {
            // Состояние покоя. Сокет должен быть переведен в режим прослушивания порта.
            SocketStatus::Closed => {
                w5500_0.tcp_listen(TCP_SOCKET_0, SERVER_PORT_0).unwrap();
            }

            // Обработка запроса по tcp
            SocketStatus::Established | SocketStatus::CloseWait => {
                let mut buf = [0u8; 64];
                if let Ok(bytes_read) = w5500_0.tcp_read(TCP_SOCKET_0, &mut buf) {
                    if bytes_read > 0 {
                        // TODO: Добавьте вашу бизнес-логику обработки входящих данных здесь
                        
                        /* Пример обработки данных:
                        for i in 0..bytes_read {
                            match buf[i as usize] {
                                b'0' => {
                                    // my_pin.set_low();
                                    let _ = w5500_0.tcp_write(TCP_SOCKET_0, b"LED ON\n");
                                }
                                b'1' => {
                                    // my_pin.set_high();
                                    let _ = w5500_0.tcp_write(TCP_SOCKET_0, b"LED OFF\n");
                                }
                                _ => {}
                            }
                        }
                        */
                    }
                }

                // Если во время обработки одного из запросов пришел еще один
                // (или если клиент быстро закрыл соединение)
                if status_0 == SocketStatus::CloseWait {
                    close_socket_0!();
                }
            }

            SocketStatus::TimeWait => {
                close_socket_0!();
            }

            // Прочие транзитные состояния (Listen, SynSent, SynRecv) не требуют
            // программного вмешательства - аппаратура отрабатывает их самостоятельно.
            _ => {}
        }
    }
}