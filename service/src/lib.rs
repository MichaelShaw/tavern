#![crate_name="tavern_service"]
#![allow(dead_code)]

extern crate aphid;
extern crate tavern_core;

extern crate time;
extern crate rand;

extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod ai;
pub mod server;
pub mod game;
pub mod board_state;
pub mod tentative;
pub mod event;