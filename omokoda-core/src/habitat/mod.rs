pub mod topology;
pub mod types;

pub use topology::{can_operate, can_reach, is_available, is_safe};
pub use types::{Area, HabitatAddress, PhysicalResource};
