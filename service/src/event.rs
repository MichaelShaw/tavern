

use tavern_core::Slot;
use tavern_core::game::santorini::*;

use game::InteractionState;
use ai::StateAnalysis;
// this is your starting state

// someone made a move

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