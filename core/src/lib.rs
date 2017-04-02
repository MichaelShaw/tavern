#![crate_name="tavern_core"]
#![allow(dead_code)]

extern crate aphid;

extern crate rand;
extern crate pad;
extern crate fnv;
extern crate colored;
extern crate time;

pub mod game;

pub use game::util::{Slot, Player, Position, Packed};






