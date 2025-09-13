#![doc = include_str!("../README.md")]
pub mod core;
pub mod helpers;
mod kinda_sorta_dangling;

pub use core::{Mutable, Output, Yoke, Yokeable};
pub use helpers::CovariantYokeable;
pub type YokeMut<Y, C> = Yoke<Y, C, Mutable>;
