use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

// 0UNDELETED 1DELETED
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteEnum {
    UNDELETED = 0,
    DELETED = 1,
}

// 0ENABLE 1DISABLE
#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEnum {
    ENABLE = 0,
    DISABLE = 1,
    LOCKED = 2,
}

// asc desc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, AsRefStr)]
pub enum OrderEnum {
    #[strum(serialize = "asc")]
    ASC,
    #[strum(serialize = "Desc")]
    DESC,
}
