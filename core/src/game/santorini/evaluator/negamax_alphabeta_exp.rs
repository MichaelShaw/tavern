
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

impl Evaluator for NegaMaxAlphaBetaExp {
    type EvaluatorState = ();

    fn name() -> String {
        "NegaMaxAlphaBetaExp".into()
    }

    fn new_state() -> () {
        ()
    }
     

    #[allow(unused_variables)]
    fn evaluate_moves_impl<H>(evaluator_state: &mut (), board: &StandardBoard, state: &State, depth: u8) -> (Option<(Move, HeuristicValue)>, EvaluatorInfo) where H: Heuristic {
        let color = color(state.to_move);
        let mut moves = Vec::with_capacity(200);

        board.next_moves(state, &mut moves);

        let mut unsorted_moves : Vec<(Move, HeuristicValue)> = Vec::with_capacity(200);

        let mut move_stack = MoveStack::new();

        let mut alpha = WORST;

        let mut info = EvaluatorInfo::new();

        for &mve in &moves {
            let (v, count) = if board.ascension_winning_move(state, mve) {
                let av = BEST * color;
                if av > alpha {
                    alpha = av;
                    info.pv_count += 1;
                }
                // info.move_count += 1;
                (av, 1)
            } else {
                let new_state = board.apply(mve, state);
                let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, WORST, -alpha, -color, &mut move_stack, &mut info); // 
                let av = v * -color;
                if -v > alpha {
                    alpha = -v;
                    info.pv_count += 1;
                }
                // alpha = max(alpha, -v);
                (av, move_count)
            };
            info.move_count += count;
            unsorted_moves.push((mve, v));
        }

        
  
        unsorted_moves.sort_by_key(|&(_, hv)| hv * -color);

        info.branch_factors.push(branch_factor(info.move_count, depth));
        
        (unsorted_moves.first().cloned(), info)
    }
}

impl NegaMaxAlphaBetaExp {
    pub fn eval<H>(board: &StandardBoard, state: &State, depth: u8, alpha:HeuristicValue, beta:HeuristicValue, color: HeuristicValue, move_stack: &mut MoveStack, info: &mut EvaluatorInfo) -> (HeuristicValue, MoveCount) where H: Heuristic {
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
        let mut best_move : Option<Move> = None;
        
        for idx in stack_begin..stack_end {
            let mve = move_stack.moves[idx];
            let (score, count) = if board.ascension_winning_move(state, mve) {
                // adding depth prioritizes close victories (forces ai to play smart, drag it out)
                // I see this as more a teaching point rather than being rude
                (BEST, 1) // VICTORY
            } else {
                let new_state = board.apply(mve, state);
                let (v, move_count) = Self::eval::<H>(board, &new_state, depth - 1, -beta, -new_alpha, -color, move_stack, info);
                (-v, move_count)
            };

            if score > best_observed {
                best_move = Some(mve);
                best_observed = score;
            }

            // best_observed = max(score, best_observed);
            new_alpha = max(new_alpha, score);
            total_moves += count;
            if beta <= new_alpha {
                break;
            }
        }

        let score_type = if best_observed <= alpha {
            EntryType::Upper
        } else if best_observed >= beta {
            EntryType::Lower
        } else {
            info.pv_count += 1;
            // println!("PV NODE mve {:?} depth {:?}", best_move,  depth);
            EntryType::Exact
        };


        move_stack.next = stack_begin;
        (best_observed, total_moves)
    }
}
