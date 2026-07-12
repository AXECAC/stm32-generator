pub mod w5500;

/// Mac аддрес устройства
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacAddr(pub [u8; 6]);
