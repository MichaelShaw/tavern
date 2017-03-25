use time;
// use HashMap;
// use rand::{XorShiftRng, Rng};
use game::santorini::*;
// use std::collections::hash_map::Entry::*;

// fn run_playouts() {
//     // this playout stuff is worthless until we can find a way to reintroduce randomness?
//     // does it matter? root sort?

//     let mut descriptions : HashMap<String, u32> = HashMap::default();

//     let mut rng = XorShiftRng::new_unseeded();
    
//     let (a, b) = aggregate_playouts::<NegaMaxAlphaBetaExp, NegaMaxAlphaBetaExp, NeighbourHeuristic, NeighbourHeuristic, _>(5, 5, 1, &mut rng, &mut descriptions);    

//     println!("A info -> {:?}", a);
//     println!("B info -> {:?}", b);

//     println!("\n\n\n=== PLAYOUTS DONE ==== \n\n");
//     for (description, count) in descriptions.iter() {
//         println!("{}x   {}", count, description);
//     }
// }

pub fn iterative_adversarial_playout<E, H, F>(board: &StandardBoard, depth: u8, mut on_move: F) -> (Player, EvaluatorInfo, EvaluatorInfo)  where E : Evaluator, H : Heuristic, F: FnMut(&State, &Move, HeuristicValue) -> () {
    let mut state = State::initial();

    let mut winner : Option<Player> = None;

    let mut a_info = EvaluatorInfo::new();
    let mut b_info = EvaluatorInfo::new();

    let mut a_state = E::new_state();
    let mut b_state = E::new_state();

    let mut move_count = 0;
    
    while winner == None {
        let mut depth = if state.to_move == Player(0) { depth } else { depth }; 
        if move_count < 2 {
            depth = max(4, depth - 1);
        }

        let mut best_move : Option<(Move, HeuristicValue)> = None;

        // iterative deepening to allow warmup
        for d in 1..(depth+1) {
            let (i_best_move, info) = if state.to_move == Player(0) {
                E::evaluate_moves::<H>(&mut a_state, board, &state, d)
            } else {
                E::evaluate_moves::<H>(&mut b_state, board, &state, d)
            };
            best_move = i_best_move;

            if state.to_move == Player(0) {
                println!("A evaluated depth {} move -> {:?} info -> {:?}", d, i_best_move, info);
                a_info += info;
            } else {
                println!("B evaluated depth {} move -> {:?} info -> {:?}", d, i_best_move, info);
                b_info += info;
            }
        }

        winner = if let Some((mve, score)) = best_move {
            let is_winning_move = board.ascension_winning_move(&state, mve);
            if is_winning_move {
                let winner = state.to_move;
                state = board.apply(mve, &state);
                on_move(&state, &mve, score);
                Some(winner) // swap it back
            } else {
                state = board.apply(mve, &state);
                on_move(&state, &mve, score);
                None
            }
        } else {
            Some(state.next_player())
        };
        move_count += 1;
    }

    (winner.unwrap(), a_info, b_info)
}


// fn adversarial_playout<EA, EB, AH, BH, R, F>(board:&StandardBoard, a_depth: u8, b_depth: u8, r: &mut R, mut on_move: F) -> (Player, EvaluatorInfo, EvaluatorInfo) where EA: Evaluator, EB: Evaluator, AH: Heuristic, BH: Heuristic, R: Rng, F: FnMut(&State, &Move, HeuristicValue) -> () {
//     let mut state = State::initial();

//     let mut winner : Option<Player> = None;

//     let mut a_info = EvaluatorInfo::new();
//     let mut b_info = EvaluatorInfo::new();

//     let mut a_state = EA::new_state();
//     let mut b_state = EB::new_state();

//     let mut move_count = 0;
    
//     while winner == None {
//         let mut depth = if state.to_move == Player(0) { a_depth } else { b_depth }; 
//         if move_count < 2 {
//             depth = max(2, depth - 1);
//         }

//         let (best_move, info) = if state.to_move == Player(0) {
//             EA::evaluate_moves::<AH>(&mut a_state, board, &state, depth)
//         } else {
//             EB::evaluate_moves::<BH>(&mut b_state, board, &state, depth)
//         };

//         if state.to_move == Player(0) {
//             a_info += info;
//         } else {
//             b_info += info;
//         }

//         winner = if let Some((mve, score)) = best_move {
//             let is_winning_move = board.ascension_winning_move(&state, mve);
//             if is_winning_move {
//                 let winner = state.to_move;
//                 state = board.apply(mve, &state);
//                 on_move(&state, &mve, score);
//                 Some(winner) // swap it back
//             } else {
//                 state = board.apply(mve, &state);
//                 on_move(&state, &mve, score);
//                 None
//             }
//         } else {
//             Some(state.next_player())
//         };
//         move_count += 1;
//     }

//     (winner.unwrap(), a_info, b_info)
// }

// fn aggregate_playouts<EA, EB, HA, HB, R>(a_depth: u8, b_depth: u8, count: u32, r: &mut R, descriptions: &mut HashMap<String, u32>) -> (EvaluatorInfo, EvaluatorInfo) where EA : Evaluator, EB : Evaluator, HA: Heuristic, HB: Heuristic, R : Rng {
//     let mut a_info = EvaluatorInfo::new();
//     let mut b_info = EvaluatorInfo::new();
//     for i in 0..count {
//         let (winner_description, a, b) = sample_adversarial_playout::<EA, EB, HA, HB, _>(a_depth, b_depth, r);    
//         a_info += a;
//         b_info += b;
//         match descriptions.entry(winner_description) {
//             Occupied(mut oe) => {
//                 *oe.get_mut() += 1;
//             },
//             Vacant(ve) => { ve.insert(1); },
//         }
//         println!("Completed {} out of {}", i, count);

//     }
//     (a_info, b_info)
// }

// fn sample_adversarial_playout<EA, EB, HA, HB, R>(a_depth: u8, b_depth: u8, r: &mut R) -> (String, EvaluatorInfo, EvaluatorInfo) where EA : Evaluator, EB : Evaluator, HA: Heuristic, HB: Heuristic, R : Rng {
//     let board = StandardBoard::new(ZobristHash::new_unseeded());
//     let mut move_number = 0;

//     let (winner, a_info, b_info) = adversarial_playout::<EA, EB, HA, HB, R, _>(&board, a_depth, b_depth, r, |state, mve, score| { 
//         move_number += 1;
//         let h = NeighbourHeuristic::evaluate(&board, state);
//         println!("======= MOVE {} =======", move_number);
//         println!("{:?} makes {:?} with expected score {}", state.next_player(), mve, score);
//         println!("{}", board.print(state));
//         println!("heuristic current state -> {}", h);
//         println!("");
//     });

//     let a_desription = format!("{} ({} depth {})", EA::name(), HA::name(), a_depth);
//     let b_desription = format!("{} ({} depth {})", EB::name(), HB::name(), b_depth);

//     let winner_message : String = if winner == Player(0) {
//         format!("Player(0) as {} beat {}", a_desription, b_desription)
//     } else {
//         format!("Player(1) {} beat {}", b_desription, a_desription)
//     };

//     println!("winner was -> {:?}", winner);
//     println!(" === winner {} ===", winner_message);
//     println!("a info -> {:?}", a_info);
//     println!("b info -> {:?}", b_info);

//     (winner_message, a_info, b_info)
// }

fn sample_principal_variant(depth:u8) {
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let init = State::initial();
    let new_state = board.apply(Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, &init);
    let mut new_state_b = board.apply(Move::PlaceBuilders { a: Slot(23), b: Slot(24) }, &new_state);
    new_state_b.buildings = new_state_b.buildings.set(Slot(5), 1);

    principal_variant::<MiniMax, SimpleHeightHeuristic>(&mut (), &board, &new_state_b, depth);
}

fn count_moves() {
    let board = StandardBoard::new(ZobristHash::new_unseeded());

    let init = State::initial();
    let new_state = board.apply(Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, &init);
    let new_state_b = board.apply(Move::PlaceBuilders { a: Slot(3), b: Slot(4) }, &new_state);

    println!("start {}", board.print(&new_state_b));

    for depth in 0..20 {
        let start = time::precise_time_ns();
        let moves = board.perft(&new_state_b, depth);
        let duration = time::precise_time_ns() - start;
        let as_seconds = (duration as f64) / 1_000_000_000f64;

        let branch_factor = (moves as f64).powf(1.0 / (depth as f64));

        let million_moves_per_second = (moves as f64) / as_seconds / 1_000_000f64;

        println!("depth {:?} moves -> {:?} in {:.3}s branch {:.1} ({:.2} million moves/second)", depth, moves, as_seconds, branch_factor, million_moves_per_second);
    }
}