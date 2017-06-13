

use tavern_core::Slot;
use tavern_core::game::santorini::*;

use game::{InteractionState, Players, UIState};
use board_state::BoardState;

use psyk::game::{GameId, Player};

use ai::StateAnalysis;

use psyk::event::{to_server, to_client};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientLocalEvent {
	UpdateTentativeSlot(Option<Slot>),
	PushCurrentSlot(Slot),
	PopCurrentSlot,
	PlayMove(Move, Option<HeuristicValue>),
    PlayerWin,
    PlayerLoss,
    NewInteractionState(InteractionState),
    NewAnalysis(StateAnalysis),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameDetails {
    pub game_id: GameId,
    pub board: BoardState,
    pub players: Players,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    UpdateUI { player: Player, ui: UIState },
    PlayMove { player: Player, mve: Move },
}






