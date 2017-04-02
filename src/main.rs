#![allow(dead_code)]

extern crate jam;
extern crate rand;
extern crate tavern_core;
extern crate time;
extern crate tavern;

fn main() {
    match tavern::app::run_app() {
    	Ok(()) => (),
    	Err(e) => println!("Error launching -> {:?}", e),
    }
}