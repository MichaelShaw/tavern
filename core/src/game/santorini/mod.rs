// use std::fmt;

// extern crate pad;
pub mod move_builder;
pub mod perft;
pub mod minimax;
pub mod minimax_alphabeta;
pub mod negamax;
pub mod negamax_alphabeta;
pub mod negamax_alphabeta_exp;

pub mod heuristic;
pub mod board;
pub mod state;
pub mod transposition;


pub mod tests;

pub use self::move_builder::*;
pub use self::heuristic::*;
pub use self::negamax::*;
pub use self::negamax_alphabeta::*;
pub use self::negamax_alphabeta_exp::*;
pub use self::minimax::*;
pub use self::board::*;
pub use self::state::*;
pub use self::minimax_alphabeta::*;
pub use self::transposition::*;


use HashSet;
use super::util::*;
use pad::Alignment;

pub type MoveCount = u32;

pub type HeuristicValue = i16;

pub type BranchFactor = f64;

pub trait Heuristic {
    fn evaluate(board: &StandardBoard, state: &State) -> HeuristicValue;
}

pub trait Evaluation {
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic;
}

pub fn branch_factor(move_count: MoveCount, depth: u8) -> f64 {
    (move_count as f64).powf(1.0 / (depth as f64))
}

pub fn playout<E, H>(board:&StandardBoard, state:&State, depth:u8) where E: Evaluation, H: Heuristic {
    println!("about to playout {}", board.print(state));
    let mut current_state = state.clone();
    for d in (1..(depth+1)).rev() {
        let (moves, _) = E::evaluate::<H>(board, &current_state, d);
        println!("legal moves -> {:?}", moves);
        if let Some(&(mve, _)) = moves.first() {
            current_state = board.apply(mve, &current_state);
            let h = H::evaluate(board, &current_state);
            println!("depth {:?} playing move {:?} score {:?}", d, mve, h);
            println!("{}", board.print(&current_state));
        } else {
            println!("no move at depth {:?}", d);
        }
    }
    // println!("what is shit -> {:?}", shit);
}


// magics
pub const UNPLACED_BUILDER : Slot = Slot(-100);
pub const DEAD_BUILDER : Slot = Slot(-101);
pub const NONE : Slot = Slot(-102);

pub type BuilderLocations = [[Slot; 2]; 2];


const PLAYERS : usize = 2;
const BUILDERS : usize = 2;
const BOARD_SIZE : usize = 5;

const PRE_ROTATE : Position = Position { x: -2, y: -2 };
const POST_ROTATE : Position = Position { x: 2, y: 2 };

pub fn rotate_90(pos: Position) -> Position {
    (ROTATE_90 * (pos + PRE_ROTATE)) + POST_ROTATE
}

pub fn rotate_180(pos: Position) -> Position {
    (ROTATE_180 * (pos + PRE_ROTATE)) + POST_ROTATE
}

pub fn rotate_270(pos: Position) -> Position {
    (ROTATE_270 * (pos + PRE_ROTATE)) + POST_ROTATE
}

pub fn reflect_x(pos: Position) -> Position {
    let mut p = pos;
    p.x = (p.x - 2) * -1 + 2;
    p
}

pub fn reflect_y(pos: Position) -> Position {
    let mut p = pos;
    p.y = (p.y - 2) * -1 + 2;
    p
}

pub fn reflect_diag_a(pos: Position) -> Position {
    Position { x: pos.y, y: pos.x }
}

pub fn reflect_diag_b(pos: Position) -> Position {
    reflect_diag_a(rotate_180(pos))
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Move {
    PlaceBuilders { a: Slot, b: Slot },
    Move { from: Slot, to:Slot, build: Slot },
}

impl Move {
    pub fn to_slots(&self) -> Vec<Slot> {
        match self {
            &Move::PlaceBuilders { a, b } => vec![a, b],
            &Move::Move { from, to, build } => vec![from, to, build],
        }
    }
}





