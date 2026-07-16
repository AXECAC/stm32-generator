pub mod w5500;

use serde::Serialize;

/// Mac аддрес устройства
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacAddr(pub [u8; 6]);
