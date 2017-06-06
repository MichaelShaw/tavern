#![crate_name="tavern_core"]
#![allow(dead_code)]

extern crate aphid;

extern crate rand;
extern crate pad;
extern crate colored;
extern crate time;

extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod game;

pub use game::util::{Slot, Player, Position, Packed};
pub use game::packed::{Packed1};







