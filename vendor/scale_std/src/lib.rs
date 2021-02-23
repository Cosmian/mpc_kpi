#![feature(min_const_generics)]
#![no_std]

//! Some convenience datastructures and functions that are not necessary
//! for working with scasm, but make life easier and require no unsafe
//! code on the user side.

pub mod array;
pub mod heap;
pub mod matrix;
pub mod slice;
pub mod circuits;
pub mod local_functions;
pub mod bit_protocols;
pub mod secret_ieee;
pub mod integer;

pub use heap::Box;
