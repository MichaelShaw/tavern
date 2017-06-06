

use tavern_core::game::santorini::{Move, State, StandardBoard};
use tavern_core::{Player};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState { 
    pub moves: Vec<Move>, // basically a replay
    pub state: State,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatchStatus {
    ToMove(Player),
    Won(Player),
}

impl BoardState {
    pub fn new(state: State) -> BoardState {
        BoardState { // this is the core
            moves: Vec::new(),
            state: state,
        }
    }

    pub fn make_move(&mut self, board:&StandardBoard, mve:Move, legal_moves: &Vec<Move>) -> MatchStatus {
        let is_winning_move = board.ascension_winning_move(&self.state, mve);

        let is_valid = legal_moves.iter().any(|m| *m == mve);
        if !is_valid {
            panic!("move wasnt valid -> {:?}", mve);
        }

        self.moves.push(mve);
        self.state = board.apply(mve, &self.state);
        
        let mut next_moves = Vec::new();
        board.next_moves_for_player(&self.state, &mut next_moves);

        if is_winning_move {
            MatchStatus::Won(self.state.next_player())
        } else if next_moves.is_empty() {
            MatchStatus::Won(self.state.next_player())
        } else {
            MatchStatus::ToMove(self.state.to_move)
        }
    }
}
