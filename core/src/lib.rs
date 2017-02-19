#![crate_name="tavern_core"]
#![allow(dead_code)]

extern crate rand;
extern crate pad;

pub mod game;

pub use game::util::{Slot, Player, Position, Packed};
