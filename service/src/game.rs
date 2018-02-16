
use tavern_core::{Slot};
use tavern_core::game::santorini::{Move, State, StandardBoard, AIProfile, Depth, HeuristicName};
use aphid::{Milliseconds};


use board_state::BoardState;

use tentative::TentativeState;

use ai::StateAnalysis;

use psyk::game::{Player, Human};

use tavern_core;

pub type PlayerSlot = tavern_core::Player;


#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum PlayerState {
    Disconnected,
    Abandoned,
    Connected(UIState),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Players {
    pub players: Vec<(Player, PlayerState)>,
    pub ai: AIProfile,
}

impl Players {
    pub fn for_player<'a>(&'a self, player:PlayerSlot) -> &'a Player {
        &self.players[player.0 as usize].0
    }

    pub fn index_of(&self, player:&Player) -> Option<usize> {
        self.players.iter().position(|&(ref pl, _)| { pl == player })
    }

    pub fn first_ai(&self) -> Option<AIProfile> {
        for &(ref player, _) in &self.players {
            match player {
                &Player::Human(_) => (),
                &Player::AI => return Some(self.ai),
            }
        }
        None
    }

    pub fn index_of_human(&self, player:&Human) -> Option<usize> {
        self.players.iter().position(|&(ref pl, _)| {
            match pl {
                &Player::Human(ref play) => play == player,
                &Player::AI { .. } => false,
            }
        })
    }

    pub fn mut_human_ui_state<'a>(&'a mut self, player:&Human) -> Option<&'a mut UIState> {
        if let Some(idx) = self.index_of_human(player) {
            match self.players[idx].1 {
                PlayerState::Connected(ref mut ui) => Some(ui),
                PlayerState::Disconnected | PlayerState::Abandoned => None,
            }
        } else {
            None
        }
    }

    pub fn human_ui_state<'a>(&'a self, player:&Human) -> Option<&'a UIState> {
        if let Some(idx) = self.index_of_human(player) {
            match self.players[idx].1 {
                PlayerState::Connected(ref ui) => Some(ui),
                PlayerState::Disconnected | PlayerState::Abandoned => None,
            }
        } else {
            None
        }
    }

    pub fn mut_ui_state<'a>(&'a mut self, player:&Player) -> Option<&'a mut UIState> {
        if let Some(idx) = self.index_of(player) {
            match self.players[idx].1 {
                PlayerState::Connected(ref mut ui) => Some(ui),
                PlayerState::Disconnected | PlayerState::Abandoned => None,
            }
        } else {
            None
        }
    }
}



pub struct ClientGame {
    pub board: BoardWithMoves, // essential
    
    pub tentative : Option<TentativeState>, // derived from board_state + players/ui

    pub players : Players, // based on slots in board state

    // this is local client state ...
    pub interactivity: InteractionState, // animation/interactivity really

    // this will be removed at some point
    pub analysis : Option<StateAnalysis>, // temporary until we get the listen server working
}

impl ClientGame {
    pub fn waiting_on_ai(&self) -> bool {
        match self.players.for_player(self.board.state().to_move) {
            &Player::AI => true,
            &Player::Human(_) => false,
        }
    }
}



#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MatchStatus {
    ToMove(PlayerSlot),
    Won(PlayerSlot),
}

// what gets sent over the wire

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BoardWithMoves {
    state: BoardState,
    legal_moves : Vec<Move>, // derived
}

impl BoardWithMoves { /// ARRRRGH java style encapsulation to ensure legal_moves 
    pub fn state(&self) -> &State {
        &self.state.state
    }

    pub fn legal_moves(&self) -> &Vec<Move> {
        &self.legal_moves
    }

    pub fn for_board(board:BoardState, standard_board:&StandardBoard) -> BoardWithMoves {
        let mut moves = Vec::new();
        standard_board.next_moves_for_player(&board.state, &mut moves);
        BoardWithMoves {
            state: board,
            legal_moves: moves,
        }
    }

    pub fn make_move(&mut self, board:&StandardBoard, mve:Move) {
        if !self.legal_moves.contains(&mve) {
            panic!("MOVE WASNT VALID -> {:?}", mve);
        }
        self.state.moves.push(mve);
        self.state.state = board.apply(mve, &self.state.state);
        let mut next_moves = Vec::new();
        board.next_moves_for_player(&self.state.state, &mut next_moves);
        self.legal_moves = next_moves;
    }

    pub fn match_status(&self, standard_board:&StandardBoard) -> MatchStatus {
        // handle winning case
        if let Some(&last_move) = self.state.moves.last() {
            if standard_board.ascension_winning_move(&self.state.state, last_move) {
                return MatchStatus::Won(self.state.state.next_player());
            }
        }

        if self.legal_moves.is_empty() {
            return MatchStatus::Won(self.state.state.next_player())
        }

        MatchStatus::ToMove(self.state.state.to_move)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UIState {
    pub current_slots : Vec<Slot>, // this is our clicked slots
    pub tentative_slot : Option<Slot>, // mouse over slot
}

impl UIState {
    pub fn clear(&mut self) {
        self.current_slots.clear();
        self.tentative_slot = None;
    }

    pub fn empty() -> UIState {
        UIState {
            current_slots: Vec::new(),
            tentative_slot : None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionState {
    AnimatingMove { prior_state: State, mve:Move, player: Player, elapsed : Milliseconds, winner: Option<Player> }, // player_type is for who's move we're animating ...
    AwaitingInput { player: Player },
    WaitingVictory { player: Player, elapsed : Milliseconds },
}

impl InteractionState {
    pub fn awaiting_input(state:&State, players:&Players) -> InteractionState {
        let player_idx = state.to_move.0 as usize;
        let player_actual = players.players[player_idx].0.clone();

        InteractionState::AwaitingInput { player: player_actual }
    }
}

#[derive(Eq, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PlayerProfile {
    pub player: Human,
    pub progress: Progress
}

#[derive(Eq, Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Progress {
    pub level: usize,
    pub wins: usize,
}

impl Progress {
    pub fn players(&self, client_player: &Human) -> Players {
        let human_player = Player::Human(client_player.clone());

        let cpu_opponent = Player::AI;


        let ai = AIProfile {
            depth: self.level as Depth,
            heuristic: HeuristicName::AdjustedNeighbour,
            time_limit: Some(10_000),
        };

        let mut players = vec![ 
            (human_player, PlayerState::Connected(UIState::empty())),
            (cpu_opponent, PlayerState::Connected(UIState::empty()))
        ];

        if self.wins % 2 == 0 {
            players.reverse();
        } 

        Players {
            players,
            ai,
        }
    }

    pub fn win(&mut self) {
        self.wins += 1;
        if self.wins == wins_to_pass_for_level(self.level) {
            self.level += 1;
            self.wins = 0;
        }
    }

    pub fn loss(&mut self) {
        if self.wins == 0 { // can't go lower than depth 2
            if self.level > 2 {
                self.level -= 1;
                self.wins = wins_to_pass_for_level(self.level) - 1;
            } 
        } else {
            self.wins -= 1;
        }
    }
}


pub fn wins_to_pass_for_level(level: usize) -> usize {
    match level {
        0...3 => 1,
        4...6 => 2,
        _ => 4,
    }
}