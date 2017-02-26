use super::*;


use HashSet;

pub struct MoveBuilder {
    pub current_state: State,
    pub board: StandardBoard,
    pub proposed_state: State,
    pub next_moves : Vec<Move>,
    pub legal_slots : HashSet<Slot>,
    pub tentative_move_count : usize,
}

impl MoveBuilder {
    pub fn new(base_state: &State, board: &StandardBoard, positions : &Vec<Slot>, tentative: Option<Slot>) -> MoveBuilder {
        let mut next_moves = Vec::new();
        board.next_moves(base_state, &mut next_moves);

        let legal_moves_as_slots : Vec<_> = next_moves.iter().flat_map(|m| m.to_slots()).filter(|sl| {
            sl.starts_with(&positions)
        }).collect();

        let mut with_tentative : Vec<_> = positions.clone();
        let mut tentative_move_count = 0;

        if let Some(slot) = tentative {
            with_tentative.push(slot);
            tentative_move_count = legal_moves_as_slots.iter().filter(|slots| slots.starts_with(&with_tentative)).count();
            if tentative_move_count == 0 {
                with_tentative.pop();
            }
        }
        let new_state = modify_state(base_state, &with_tentative);

        let mut legal_slots : HashSet<Slot> = HashSet::default();
        
        for slots in &legal_moves_as_slots {
            let next_slot_idx = positions.len() as usize;
            if next_slot_idx < slots.len() {
                legal_slots.insert(slots[next_slot_idx]);
            }
        }

         legal_moves_as_slots.iter().filter(|slots| slots.starts_with(&with_tentative)).count();

        MoveBuilder {
            current_state: base_state.clone(),
            board: board.clone(),
            proposed_state: new_state,
            next_moves: next_moves,
            legal_slots: legal_slots,
            tentative_move_count: tentative_move_count,
        }
    }
}

pub fn modify_state(base_state:&State, slots:&Vec<Slot>) -> State {
    let mut new_state = base_state.clone();
    if new_state.builders_to_place() {
        for (i, slot) in slots.iter().enumerate() {
            new_state.builder_locations[new_state.to_move.0 as usize][i] = *slot;
        }
    } else {
        for i in 0..2 {
            let builder_location = new_state.builder_locations[new_state.to_move.0 as usize][i];
            if let Some(from) = slots.get(0) {
                if builder_location == *from {
                    if let Some(to) = slots.get(1) {
                        new_state.builder_locations[new_state.to_move.0 as usize][i] = *to;
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
