
use tavern_core::{Player, Slot};
use tavern_core::game::santorini::{Move, State};
use aphid::{HashSet, Seconds};


use board_state::BoardState;

use tentative::TentativeState;

use ai::StateAnalysis;

pub struct Game {
	pub board_state: BoardState, // essential state
    pub ui_state : UIState, // transient, tied to this current game
    pub cpu_players : HashSet<Player>,
}

pub struct UIState {
    pub interaction_state: InteractionState, // animation/interactivity really
    pub legal_moves : Vec<Move>, // produced after each move
    pub current_slots : Vec<Slot>, // this is our clicked slots
    pub tentative_slot : Option<Slot>, // mouse over slot
    pub tentative: TentativeState, // predicted moves

    pub state_analysis : Option<StateAnalysis>, // temporary until we get the listen server working
}

// after each move, clear out legal_moves/current_slots/tentative_slot ..... and tentative gets produced every frame ....

// is interaction state a .... local thing .... or a server thing ...
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionState {
    AnimatingMove { prior_state: State, mve:Move, player_type: PlayerType, elapsed : Seconds, winner: Option<Player> }, // player_type is for who's move we're animating ...
    AwaitingInput { player: Player, player_type: PlayerType },
    WaitingVictory { player: Player, elapsed : Seconds },
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerType {
    AI,
    Human,
}


#[derive(Eq, Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Progress {
    pub level: usize,
    pub wins: usize,
}
