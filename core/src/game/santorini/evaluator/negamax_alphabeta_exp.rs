
// use HashMap;
use game::santorini::*;
use std::cmp::{max, min};

fn color(player:Player) -> HeuristicValue {
    match player {
        Player(0) => 1,
        Player(1) => -1,
        _ => panic!("fn color was given player -> {:?} (only supports 0, 1)", player),
    }
}

pub struct NegaMaxAlphaBetaExp { }

// use rand::Rng;
// use rand::{XorShiftRng, ChaChaRng};

pub struct EvState {
    transposition: TranspositionTable,
    // pv_nodes : Vec<TranspositionEntry>,
}

impl Evaluator for NegaMaxAlphaBetaExp {
    type EvaluatorState = EvState;

    fn name() -> String {
        "NegaMaxAlphaBetaExp".into()
    }

    fn new_state() -> EvState {
        EvState {
            transposition : TranspositionTable::new(22),
        }
    }

    fn reset(evaluator_state: &mut EvState) {
        evaluator_state.transposition.reset();
    }

    fn new_search(evaluator_state: &mut EvState) {
        evaluator_state.transposition.increment_generation();
    }
     
    #[allow(unused_variables)]
    fn evaluate_moves_impl<H>(evaluator_state: &mut EvState, board: &StandardBoard, state: &State, depth: Depth) -> (Option<(Move, HeuristicValue)>, EvaluatorInfo) where H: Heuristic {
        let color = color(state.to_move);

        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);

        let mut move_stack = MoveStack::new();
        let stack_begin = 0;
        board.next_moves(state, &mut move_stack);
        let stack_end = move_stack.next;

        let mut alpha = WORST;
        let mut beta = BEST;

        let mut info = EvaluatorInfo::new();

        let hash = board.hash(state);

        let mut tt_best_move : Option<Move> = None;

        let (tt_idx, found) = evaluator_state.transposition.probe(hash);

        if found {
            let entry = &evaluator_state.transposition.entries[tt_idx];
            if entry.depth >= depth {
                info.tt_valid += 1;
                match entry.entry_type {
                    EntryType::Exact => {
                        if let Some(mv) = entry.best_move {
                            return (Some((mv, entry.value)), info)  // unsure about this negation
                        } 
                    },
                    EntryType::Lower => {
                        alpha = max(alpha, entry.value);
                    },
                    EntryType::Upper => {
                        beta = min(beta, entry.value)
                    },
                }
                if alpha >= beta {
                    if let Some(mv) = entry.best_move {
                        return (Some((mv, entry.value)), info)  // unsure about this negation
                    } 
                }
            } else {
                info.tt_suggest += 1;
            }
            tt_best_move = entry.best_move;
        } else {
            info.tt_miss += 1;
        }

        // if we have a best move, order it first
        if let Some(mve) = tt_best_move {
            for idx in stack_begin..stack_end {
                if move_stack.moves[idx] == mve {
                    move_stack.moves.swap(stack_begin, idx);
                    break;
                }
            }    
        }

        let mut best_move : Option<Move> = None;
        let mut best_observed = WORST;

        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];
            let (v, count) = if board.ascension_winning_move(state, mve) {
                let av = BEST * color;
                if av > alpha {
                    alpha = av;
                    best_move = Some(mve);
                    best_observed = av;
                    info.pv_count += 1;
                }
                (av, 1)
            } else {
                let new_state = board.apply(mve, state);
                let delta_hash = board.delta_hash(state, mve);
                let (v, move_count) = Self::eval::<H>(board, &new_state, hash ^ delta_hash, depth - 1, -beta, -alpha, -color, &mut move_stack, &mut info, evaluator_state); // 

                let av = v * -color;
                if -v > alpha {
                    alpha = -v;
                    best_move = Some(mve);
                    best_observed = av;
                    info.pv_count += 1;
                }

                (av, move_count)
            };
            info.move_count += count;
            unsorted_moves.push((mve, v));
        }

        info.pv_count += 1;

        evaluator_state.transposition.store(tt_idx, hash, best_observed, depth, EntryType::Exact, best_move);
  
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);

        info.branch_factors.push(branch_factor(info.move_count, depth));
        
        (unsorted_moves.first().cloned(), info)
    }
}

impl NegaMaxAlphaBetaExp {
    pub fn eval<H>(board: &StandardBoard, state: &State, hash: StateHash, depth: Depth, alpha:HeuristicValue, beta:HeuristicValue, color: HeuristicValue, move_stack: &mut MoveStack, info: &mut EvaluatorInfo, ev_state : &mut EvState) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut new_alpha = alpha;
        let mut new_beta = beta;

        // lookup transposition table
        let mut tt_best_move : Option<Move> = None;
        let (tt_idx, found) = ev_state.transposition.probe(hash);
        if found {
            let entry = &ev_state.transposition.entries[tt_idx];
            if entry.depth >= depth {
                info.tt_valid += 1;
                match entry.entry_type {
                    EntryType::Exact => {
                        return (entry.value, 0);
                    },
                    EntryType::Lower => {
                        new_alpha = max(new_alpha, entry.value);
                    },
                    EntryType::Upper => {
                        new_beta = min(new_beta, entry.value);
                    },
                }
                if new_alpha >= new_beta {
                    return (entry.value, 0)
                }
            } else {
                info.tt_suggest += 1;
            }
            tt_best_move = entry.best_move;
        } else {
            info.tt_miss += 1;
        }

        if depth == 0 {
            let v = H::evaluate(board, state) * color;
            return (v, 1);
        }

        let stack_begin = move_stack.next;
        board.next_moves(state, move_stack);
        let stack_end = move_stack.next;
       
        let mut total_moves = 0;
        let mut best_observed = WORST;
        let mut best_move : Option<Move> = None;

        // WE NEED TO ORDER THE MOVES

        // find the best move and swap it to first
        if let Some(mve) = tt_best_move {
            for idx in stack_begin..stack_end {
                if move_stack.moves[idx] == mve {
                    move_stack.moves.swap(stack_begin, idx);
                    break;
                }
            }    
        }
        
        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];
            let (score, count) = if board.ascension_winning_move(state, mve) {
                // adding depth prioritizes close victories (forces ai to play smart, drag it out)
                // I see this as more a teaching point rather than being rude
                (BEST, 1) // VICTORY
            } else {
                let new_state = board.apply(mve, state);
                let delta_hash = board.delta_hash(state, mve);
                let (v, move_count) = Self::eval::<H>(board, &new_state, hash ^ delta_hash, depth - 1, -new_beta, -new_alpha, -color, move_stack, info, ev_state);
                (-v, move_count)
            };

            if score > best_observed {
                best_move = Some(mve);
                best_observed = score;
            }

            new_alpha = max(new_alpha, score);
            total_moves += count;
            if new_beta <= new_alpha {
                break;
            }
        }

        let score_type = if best_observed <= alpha {
            EntryType::Upper
        } else if best_observed >= new_beta { // unsure if this should be beta
            EntryType::Lower
        } else {
            info.pv_count += 1;
            EntryType::Exact
        };

        ev_state.transposition.store(tt_idx, hash, best_observed, depth, score_type, best_move);

        move_stack.next = stack_begin;
        (best_observed, total_moves)
    }
}
