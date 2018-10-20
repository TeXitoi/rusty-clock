#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;
extern crate bitflags;
extern crate embedded_graphics;
extern crate embedded_hal;
extern crate heapless;
extern crate il3820;

pub mod alarm;
pub mod button;
pub mod datetime;
pub mod ui;
