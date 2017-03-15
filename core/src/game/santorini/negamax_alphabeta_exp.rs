
use game::santorini::*;
use std::cmp::{max, min};

fn color(player:Player) -> HeuristicValue {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

pub const MAX_DEPTH: usize = 15;
pub const MAX_MOVES: usize = 256;
pub const MAX_MOVE_STACK : usize = MAX_DEPTH * MAX_MOVES;

pub struct MoveStack {
    pub moves: [Move; MAX_MOVE_STACK],
    pub next: usize,
}

impl MoveStack {
    pub fn new() -> MoveStack {
        MoveStack {
            moves: [Move::PlaceBuilders { a: Slot(0), b: Slot(0) } ; MAX_MOVE_STACK],
            next: 0,
        }
    }

    pub fn push(&mut self, mve:Move) {
       self.moves[self.next] = mve;
       self.next += 1;
    }
}

impl MoveSink for MoveStack {
    fn sink(&mut self, mve:Move) {
        self.push(mve);
    }
}

pub struct NegaMaxAlphaBetaExp {}

impl Evaluation for NegaMaxAlphaBetaExp {
    // THIS IS 100% FUCKED
    fn evaluate<H>(board: &StandardBoard, state: &State, depth: u8) -> (Vec<(Move, HeuristicValue)>, MoveCount) where H: Heuristic {
    	let color = color(state.to_move);
        let mut moves = Vec::with_capacity(200);
        board.next_moves(state, &mut moves);

        let mut total_moves = 0;
        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);


        let mut move_stack = MoveStack::new();

    	// let mut alpha = WORST;
        if state.to_move == Player(0) {
            let mut alpha = WORST;
            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    let av = BEST * color;
                    unsorted_moves.push((mve, av));

                    alpha = max(alpha, av);

                    total_moves += 1;
                } else {
                    let new_state = board.apply(mve, state);

                    let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, WORST, -alpha, -color, &mut move_stack);
                    
                    let av = v * -color;

                    alpha = max(alpha, av);

                    unsorted_moves.push((mve, av));
                    total_moves += move_count;
                }
            }
        } else {
            let mut beta = BEST;
            for &mve in &moves {
                if board.ascension_winning_move(state, mve) {
                    let av = BEST * color;
                    unsorted_moves.push((mve, av));

                    // alpha = max(alpha, av);
                    beta = min(beta, av);

                    total_moves += 1;
                } else {
                    let new_state = board.apply(mve, state);

                    let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, WORST, beta, -color, &mut move_stack);
                    
                    let av = v * -color;

                    beta = min(beta, av);

                    // alpha = max(alpha, av);
                    unsorted_moves.push((mve, av));
                    total_moves += move_count;
                }
            }
        }
  
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);
        (unsorted_moves, total_moves)
    }
}

impl NegaMaxAlphaBetaExp {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, alpha:HeuristicValue, beta:HeuristicValue, color: HeuristicValue, move_stack: &mut MoveStack) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut new_alpha = alpha;
        let mut new_beta = beta;

        let stack_begin = move_stack.next;
        board.next_moves(state, move_stack);
        let stack_end = move_stack.next;

        if depth == 0 {
            let v = if stack_begin == stack_end {
                WORST
            } else {
               H::evaluate(board, state) * color
            };
            // let v = H::evaluate(board, state) * color;
            move_stack.next = stack_begin;
            return (v, 1);
        }
       
        let mut total_moves = 0;
        let mut best_observed = WORST;
        
        
        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];
            if board.ascension_winning_move(state, mve) {
                move_stack.next = stack_begin;
                return (BEST, total_moves + 1);
            } else {
                let new_state = board.apply(mve, state);
                let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, -beta, -new_alpha, -color, move_stack);
                total_moves += move_count;
                best_observed = max(-v, best_observed);
                new_alpha = max(new_alpha, -v);
                if beta <= new_alpha {
                	break;
                }
            }
        }

        move_stack.next = stack_begin;
        (best_observed, total_moves)
    }
}
