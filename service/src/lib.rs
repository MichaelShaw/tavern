#![crate_name="tavern_service"]
#![allow(dead_code)]

extern crate aphid;
extern crate tavern_core;

extern crate time;
extern crate rand;

extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate psyk;

pub mod ai;
pub mod server;
pub mod game;
pub mod board_state;
pub mod tentative;
pub mod event;

use psyk::event::{to_server, to_client};
use event::{GameEvent, GameDetails};

pub type ToServerEvent = to_server::Event<GameEvent>;
pub type ToClientEvent = to_client::Event<GameEvent, GameDetails>;
