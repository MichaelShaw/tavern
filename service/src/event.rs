

use tavern_core::Slot;
use tavern_core::game::santorini::*;

use game::{InteractionState};

use ai::StateAnalysis;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u64);

pub mod to_server {
    use super::{GameId, Version};
    use game::*;
    use tavern_core::game::santorini::*;
    use board_state::BoardState;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Event {
        pub player: PlayerActual,
        pub payload: Payload,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Payload {
        Hello(Version), // unsure if this will actually work ...
        ListGames,
        NewGame,
        JoinGame(GameId),
        AbandonGame(GameId),
        GameEvent {
            game_id: GameId,
            payload: GameEventPayload,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum GameEventPayload {
        UpdateUI(UIState),
        PlayMove(Move),
    }
}

pub mod to_client {
    use super::GameId;
    use tavern_core::game::santorini::*;
    use game::*;
    use board_state::BoardState;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Event {
        GameListing(Vec<GameDetails>),
        GameCreated(GameDetails),
        GameEvent {
            game_id: GameId,
            payload: GameEventPayload,
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GameDetails {
        game_id: GameId,
        board: BoardState,
        players: Players,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum GameEventPayload {
        PlayMove { // basically an ack of an attempted move
            mve: Move,
            player: PlayerActual,
            winner: Option<PlayerActual>,
        },
        UpdatePlayers(Players), // ui state, or disconnect etc.
    }
}




