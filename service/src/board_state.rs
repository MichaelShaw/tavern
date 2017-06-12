use tavern_core::game::santorini::{Move, State};


// it's it's own thing because we're gonna ship it across the wire as essential state
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct BoardState { 
    pub moves: Vec<Move>, // basically a replay
    pub state: State,
}


impl BoardState {
    pub fn new(state: State) -> BoardState {
        BoardState { // this is the core
            moves: Vec::new(),
            state: state,
        }
    }
}
