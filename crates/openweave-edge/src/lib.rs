#![cfg_attr(feature = "edge", no_std)]

#[cfg(feature = "edge")]
extern crate alloc;

pub mod allocator;
pub mod wasm_target;