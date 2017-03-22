#![crate_name="tavern_core"]
#![allow(dead_code)]


extern crate rand;
extern crate pad;
extern crate fnv;
extern crate colored;
extern crate time;

pub mod game;

pub use game::util::{Slot, Player, Position, Packed};

use fnv::FnvHasher;
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::collections::hash_map::Entry::*;

pub type HashMap<K, V> = StdHashMap<K, V, BuildHasherDefault<FnvHasher>>;
pub type HashSet<V> = StdHashSet<V, BuildHasherDefault<FnvHasher>>;

// let bullshit = vec![1,3,2,5,6,1,23,5,6,72];
// let as_a_map = group_by(bullshit, |e| e % 2);
pub fn group_by<T, K, F>(items: Vec<T>, f: F) -> HashMap<K, Vec<T>> where F : Fn(&T) -> K, K : Eq + Hash {
	let mut map : HashMap<K, Vec<T>> = HashMap::default();

	for item in items.into_iter() {
		let k = f(&item);
		match map.entry(k) {
			Occupied(mut cl) => {
                (cl.get_mut()).push(item);
            },
            Vacant(ve) => { 
            	ve.insert(vec![item]);
            },
		}
	}

	map
}

pub fn contains<T, F>(opt: Option<T>, f: F) -> bool where F: Fn(&T) -> bool {
	opt.iter().any(f)
}
