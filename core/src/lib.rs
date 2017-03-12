#![crate_name="tavern_core"]
#![allow(dead_code)]

extern crate rand;
extern crate pad;
extern crate fnv;
extern crate colored;

pub mod game;

pub use game::util::{Slot, Player, Position, Packed};

use fnv::FnvHasher;
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};
use std::hash::BuildHasherDefault;


pub type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;
pub type HashSet<V> = StdHashSet<V, BuildHasherDefault<FnvHasher>>;
