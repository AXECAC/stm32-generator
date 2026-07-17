pub mod start;
pub mod peripherals;
pub mod pins;
pub mod spi;
pub mod run;

#[derive(PartialEq, Clone, Copy)]
pub enum Page {
    Start,
    Peripherals,
    Pins,
    Spi,
    Run,
}
