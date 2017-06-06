use tavern_core::game::santorini::{Move, State};
use tavern_core::{Slot, Packed1};

use aphid::HashSet;


pub struct TentativeState {
    pub matching_slots : HashSet<Slot>, // matching positions for highlights
    pub move_count : usize,
    pub proposed_state: State, 
}

fn produce_tentative(state:&State, legal_moves: &Vec<Move>, current_slots: &Vec<Slot>, tentative_slot: Option<Slot>) -> TentativeState {
    let legal_moves_as_slots : Vec<_> = legal_moves.iter().map(|m| m.to_slots()).filter(|sl| {
        sl.starts_with(&current_slots)
    }).collect();

    let mut with_tentative : Vec<_> = current_slots.clone();
    let mut tentative_move_count = 0;

    if let Some(slot) = tentative_slot {
        with_tentative.push(slot);
        tentative_move_count = legal_moves_as_slots.iter().filter(|slots| slots.starts_with(&with_tentative)).count();
        if tentative_move_count == 0 {
            with_tentative.pop();
        }
    }
    let new_state = modify_state(state, &with_tentative);

    let mut matching_slots : HashSet<Slot> = HashSet::default();
    
    for slots in &legal_moves_as_slots {
        let next_slot_idx = current_slots.len() as usize;
        if next_slot_idx < slots.len() {
            matching_slots.insert(slots[next_slot_idx]);
        }
    }

    TentativeState {
        proposed_state: new_state,
        matching_slots: matching_slots,
        move_count: tentative_move_count,
    }
}


// applies predicted slot moves to some state to produce a new state (used in tentative production)
fn modify_state(base_state:&State, slots:&Vec<Slot>) -> State {
    let mut new_state = base_state.clone();

    let current_builders = new_state.current_builders();

    if !current_builders.any() {
        for slot in slots {
            new_state.builders[new_state.to_move.0 as usize].0 |= 1 << slot.0;
        }
    } else {
        for bl in new_state.builders[new_state.to_move.0 as usize].iter() {
            if let Some(from) = slots.get(0) {
                if bl == *from {
                    if let Some(to) = slots.get(1) {
                        let movement_mask = Packed1((1 << from.0) | (1 << to.0));

                        new_state.builders[new_state.to_move.0 as usize] ^= movement_mask;

                        if let Some(build) = slots.get(2) {
                            new_state.build_at(*build);
                        }
                    }
                }
            }
        }
    }

    new_state
}