use super::*;
use game::*;

use HashSet;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchStatus {
    ToMove(Player),
    Won(Player),
}

#[derive(Debug, Clone)]
pub struct BoardState { 
    pub moves: Vec<Move>, // basically a replay
    pub state: State,
    pub board: StandardBoard,
    pub next_moves : Vec<Move>,
}

impl BoardState {
    pub fn new(board : StandardBoard, state: State) -> BoardState {
        let mut next_moves = Vec::new();
        board.next_moves_for_player(&state, &mut next_moves);

        BoardState { // this is the core
            moves: Vec::new(),
            state: state,
            board: board,
            next_moves : next_moves,
        }
    }


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
            self.board.next_moves_for_player(&self.state, &mut self.next_moves);    
        }
        
        if is_winning_move {
            MatchStatus::Won(self.state.next_player())
        } else if self.next_moves.is_empty() {
            MatchStatus::Won(self.state.next_player())
        } else {
            MatchStatus::ToMove(self.state.to_move)
        }
    }

    pub fn tentative(&self, positions : &Vec<Slot>, tentative: Option<Slot>) -> TentativeState {
        let legal_moves_as_slots : Vec<_> = self.next_moves.iter().map(|m| m.to_slots()).filter(|sl| {
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

        TentativeState {
            proposed_state: new_state,
            matching_slots: matching_slots,
            move_count: tentative_move_count,
        }
    }
}

pub struct TentativeState {
    pub proposed_state: State,
    pub matching_slots : HashSet<Slot>,
    pub move_count : usize,
}



// fuck, I don't really get this at all :-/
fn modify_state(base_state:&State, slots:&Vec<Slot>) -> State {
    let mut new_state = base_state.clone();

    // println!("modify state with slots {:?}", slots);

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
