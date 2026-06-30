pub mod f401;

pub enum StmF4PinMode {
    InputMode(StmF4InputMode),
}

pub enum StmF4InputMode {
    Analog,
    Floating,
    PullUp,
    PullDown,
}
