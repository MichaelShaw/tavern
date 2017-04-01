
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
        let state = EvState {
            transposition : TranspositionTable::new(24),
            // pv_nodes : Vec::new(),
        };
        println!("constructed state with size -> {} ({} bytes)", state.transposition.entries.len(), state.transposition.approx_size_bytes());
        state
    }
     

    #[allow(unused_variables)]
    fn evaluate_moves_impl<H>(evaluator_state: &mut EvState, board: &StandardBoard, state: &State, depth: u8) -> (Option<(Move, HeuristicValue)>, EvaluatorInfo) where H: Heuristic {
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

        // println!("== starting depth {:?}", depth);

        // TT TABLE READ
        let mut tt_best_move : Option<Move> = None;
        {
            if let Some(entry) = evaluator_state.transposition.get(hash)  {
                // let ok = &entry.state == state;
                // if !ok {
                //     println!("FUCK WE GOT A STATE MISMATCH!!!!");


                //     println!("CURRENT {:?} state -> {}", hash, board.print(state));
                //     println!("cur hash -> {:?}", board.hash(&state));
                //     println!("ENTRY {:?} state -> {}", entry.hash, board.print(&entry.state));
                //     println!("entry hash -> {:?}", board.hash(&entry.state));


                //     panic!("FUCK THIS SHIT IM OUTTY");
                // }
                // println!("WE ROOT DID WE GET HIT -> {:?} desired depth {:?}", entry, depth);
                if entry.depth >= depth {
                    info.tt_valid += 1;
                    match entry.entry_type {
                        EntryType::Exact => {
                            // println!("valid exact!");
                            if let Some(mv) = entry.best_move {
                                // println!("it has a best move ... returning");
                                return (Some((mv, entry.value)), info)  // unsure about this negation   
                            } 
                            // return (Some((entry.best_move.unwrap(), entry.value)), info)
                            // return (entry.value, 0)
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
        }

        if let Some(mve) = tt_best_move {
            // println!("we have a best move {:?} stack begin {} end {}", mve, stack_begin, stack_end);
            for idx in stack_begin..stack_end {
                if move_stack.moves[idx] == mve {
                    // println!("PREEEEE");
                    // println!("ok we're at {} move is {:?}", idx, move_stack.moves[idx]);
                    // println!("start is {:?}", move_stack.moves[stack_begin]);

                    move_stack.moves.swap(stack_begin, idx);
                    // move_stack.moves.swap(stack_begin, idx);

                    // println!("POST");
                    // println!("ok we're at {} move is {:?}", idx, move_stack.moves[idx]);
                    // println!("start is {:?}", move_stack.moves[stack_begin]);

                    break;
                }
            }    
        }

        let mut best_move : Option<Move> = None;
        let mut best_observed = WORST;

        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];
            // get the fuckin move
            let (v, count) = if board.ascension_winning_move(state, mve) {
                let av = BEST * color;
                if av > alpha {
                    alpha = av;
                    best_move = Some(mve);
                    best_observed = av;
                    info.pv_count += 1;
                }
                // info.move_count += 1;
                (av, 1)
            } else {
                let new_state = board.apply(mve, state);
                let delta_hash = board.delta_hash(state, mve);
                let (v, move_count) = Self::eval::<H>(board, &new_state, hash ^ delta_hash, depth - 1, -beta, -alpha, -color, &mut move_stack, &mut info, evaluator_state); // 

                // println!("move has value {:?}", v);
                let av = v * -color;
                if -v > alpha {

                    alpha = -v;
                    // println!("its better! alpha is now {:?}", alpha);
                    best_move = Some(mve);
                    best_observed = av; // FUCK, WHAT DO WE DO HERE
                    info.pv_count += 1;
                }
                // if av > alpha {
                //     alpha = av;
                //     best_move = Some(mve);
                //     best_observed = av; // FUCK, WHAT DO WE DO HERE
                //     info.pv_count += 1;
                // }
                // alpha = max(alpha, -v);
                (av, move_count)
            };
            info.move_count += count;
            unsorted_moves.push((mve, v));
        }

        info.pv_count += 1;

        let entry = TranspositionEntry {
            hash: hash,
            value: best_observed,
            entry_type: EntryType::Exact,
            depth: depth,
            best_move: best_move,
        };
        evaluator_state.transposition.put(entry);
  
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);

        info.branch_factors.push(branch_factor(info.move_count, depth));
        
        (unsorted_moves.first().cloned(), info)
    }
}

impl NegaMaxAlphaBetaExp {
    pub fn eval<H>(board: &StandardBoard, state: &State, hash: StateHash, depth: u8, alpha:HeuristicValue, beta:HeuristicValue, color: HeuristicValue, move_stack: &mut MoveStack, info: &mut EvaluatorInfo, ev_state : &mut EvState) -> (HeuristicValue, MoveCount) where H: Heuristic {
        let mut new_alpha = alpha;
        let mut new_beta = beta;


        let mut tt_best_move : Option<Move> = None;

        {
            if let Some(entry) = ev_state.transposition.get(hash)  {
                // let ok = &entry.state == state;
                // if !ok {
                //     println!("FUCK WE GOT A STATE MISMATCH!!!!");


                //     println!("CURRENT {:?} state -> {}", hash, board.print(state));
                //     println!("cur hash -> {:?}", board.hash(&state));
                //     println!("ENTRY {:?} state -> {}", entry.hash, board.print(&entry.state));
                //     println!("entry hash -> {:?}", board.hash(&entry.state));


                //     panic!("FUCK THIS SHIT IM OUTTY");
                // }


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
        }

        

        if depth == 0 {
            // let v = if stack_begin == stack_end {
            //    WORST
            // } else {
            //    H::evaluate(board, state) * color
            // };
            let v = H::evaluate(board, state) * color;
            // move_stack.next = stack_begin;
            return (v, 1);
        }

        let stack_begin = move_stack.next;
        board.next_moves(state, move_stack);
        let stack_end = move_stack.next;
       
        let mut total_moves = 0;
        let mut best_observed = WORST;
        let mut best_move : Option<Move> = None;


        // WE NEED TO ORDER THE MOVES

        // find the best move and swap it to first?!
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

            // best_observed = max(score, best_observed);
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
            // println!("PV NODE mve {:?} depth {:?}", best_move,  depth);
            EntryType::Exact
        };

        let entry = TranspositionEntry {
            // state: state.clone(),
            hash: hash,
            value: best_observed,
            entry_type: score_type,
            depth: depth,
            best_move: best_move,
        };
        // if score_type == EntryType::Exact {
        //     ev_state.pv_nodes.push(entry.clone());
        // }
        ev_state.transposition.put(entry);

        move_stack.next = stack_begin;
        (best_observed, total_moves)
    }
}
