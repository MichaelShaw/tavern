use super::*;


use HashSet;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchStatus {
    ToMove(Player),
    Won(Player),
}

#[derive(Debug, Clone)]
pub struct CoreGame { 
    pub moves: Vec<Move>,
    pub state: State,
    pub board: StandardBoard,
    pub next_moves : Vec<Move>,
}

impl CoreGame {
    pub fn make_move(&mut self, mve:Move) -> MatchStatus {
        let is_winning_move = self.board.ascension_winning_move(&self.state, mve);

        let is_valid = self.next_moves.iter().any(|m| *m == mve);
        if !is_valid {
            panic!("move wasnt valid -> {:?}", mve);
        }
        self.moves.push(mve);
        self.state = self.board.apply(mve, &self.state);
        self.next_moves.clear();
        if !is_winning_move {
            self.board.next_moves(&self.state, &mut self.next_moves);    
        }
        
        if is_winning_move {
            MatchStatus::Won(self.state.next_player())
        } else if self.next_moves.is_empty() {
            MatchStatus::Won(self.state.next_player())
        } else {
            MatchStatus::ToMove(self.state.to_move)
        }
    }

    pub fn new(board : StandardBoard, state: State) -> CoreGame {
        let mut next_moves = Vec::new();
        board.next_moves(&state, &mut next_moves);

        CoreGame { // this is the core
            moves: Vec::new(),
            state: state,
            board: board,
            next_moves : next_moves,
        }
    }

    pub fn tentative(&self, positions : &Vec<Slot>, tentative: Option<Slot>) -> TentativeGame {
        let legal_moves_as_slots : Vec<_> = self.next_moves.iter().flat_map(|m| m.to_slots()).filter(|sl| {
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
        let new_state = modify_state(&self.state, &with_tentative);

        let mut matching_slots : HashSet<Slot> = HashSet::default();
        
        for slots in &legal_moves_as_slots {
            let next_slot_idx = positions.len() as usize;
            if next_slot_idx < slots.len() {
                matching_slots.insert(slots[next_slot_idx]);
            }
        }

        TentativeGame {
            proposed_state: new_state,
            matching_slots: matching_slots,
            move_count: tentative_move_count,
        }
    }
}

pub struct TentativeGame {
    pub proposed_state: State,
    pub matching_slots : HashSet<Slot>,
    pub move_count : usize,
}


fn modify_state(base_state:&State, slots:&Vec<Slot>) -> State {
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
