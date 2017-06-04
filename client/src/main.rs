#![allow(dead_code)]

extern crate jam;
extern crate rand;
extern crate tavern_core;
extern crate time;
extern crate tavern_client;

fn main() {
    match tavern_client::app::run_app() {
    	Ok(()) => (),
    	Err(e) => println!("Error launching -> {:?}", e),
    }
}