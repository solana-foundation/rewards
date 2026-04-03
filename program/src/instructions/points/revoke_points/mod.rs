pub mod accounts;
pub mod data;
pub mod processor;

pub use crate::instructions::impl_instructions::RevokePoints;
pub use accounts::*;
pub use data::*;
pub use processor::*;
