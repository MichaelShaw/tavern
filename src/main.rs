#![allow(dead_code)]

extern crate jam;
extern crate rand;
extern crate tavern_core;
extern crate time;
extern crate tavern;

use rand::{Rng, XorShiftRng};

use std::mem;

use tavern_core::game::santorini::*;
use tavern_core::game::util::*; // , Packed, Packed1, Packed2, Slot};

use tavern_core::HashMap;
use std::collections::hash_map::Entry::*;

fn main() {
    tavern::app::run_app();

    // SimpleHeightHeuristic
    // NeighbourHeuristic

    // NegaMaxAlphaBetaExp
    
    // run_playouts();    
}

fn run_playouts() {
    let mut descriptions : HashMap<String, u32> = HashMap::default();

    let mut rng = XorShiftRng::new_unseeded();
    
    let (a, b) = aggregate_playouts::<NegaMaxAlphaBetaExp, NegaMaxAlphaBetaExp, NeighbourHeuristic, NeighbourHeuristic, _>(4, 4, 1, &mut rng, &mut descriptions);    

    println!("A info -> {:?}", a);
    println!("B info -> {:?}", b);

    println!("\n\n\n=== PLAYOUTS DONE ==== \n\n");
    for (description, count) in descriptions.iter() {
        println!("{}x   {}", count, description);
    }
}

fn aggregate_playouts<EA, EB, HA, HB, R>(a_depth: u8, b_depth: u8, count: u32, r: &mut R, descriptions: &mut HashMap<String, u32>) -> (EvaluatorInfo, EvaluatorInfo) where EA : Evaluator, EB : Evaluator, HA: Heuristic, HB: Heuristic, R : Rng {
    let mut a_info = EvaluatorInfo::new();
    let mut b_info = EvaluatorInfo::new();
    for i in 0..count {
        let (winner_description, a, b) = sample_adversarial_playout::<EA, EB, HA, HB, _>(a_depth, b_depth, r);    
        a_info += a;
        b_info += b;
        match descriptions.entry(winner_description) {
            Occupied(mut oe) => {
                *oe.get_mut() += 1;
            },
            Vacant(ve) => { ve.insert(1); },
        }
        println!("Completed {} out of {}", i, count);

    }
    (a_info, b_info)
}

fn sample_adversarial_playout<EA, EB, HA, HB, R>(a_depth: u8, b_depth: u8, r: &mut R) -> (String, EvaluatorInfo, EvaluatorInfo) where EA : Evaluator, EB : Evaluator, HA: Heuristic, HB: Heuristic, R : Rng {
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let mut move_number = 0;

    let (winner, a_info, b_info) = adversarial_playout::<EA, EB, HA, HB, R, _>(&board, a_depth, b_depth, r, |state, mve, score| { 
        move_number += 1;
        let h = NeighbourHeuristic::evaluate(&board, state);
        println!("======= MOVE {} =======", move_number);
        println!("{:?} makes {:?} with expected score {}", state.next_player(), mve, score);
        println!("{}", board.print(state));
        println!("heuristic current state -> {}", h);
        println!("");
    });

    let a_desription = format!("{} ({} depth {})", EA::name(), HA::name(), a_depth);
    let b_desription = format!("{} ({} depth {})", EB::name(), HB::name(), b_depth);

    let winner_message : String = if winner == Player(0) {
        format!("Player(0) as {} beat {}", a_desription, b_desription)
    } else {
        format!("Player(1) {} beat {}", b_desription, a_desription)
    };

    println!("winner was -> {:?}", winner);
    println!(" === winner {} ===", winner_message);
    println!("a info -> {:?}", a_info);
    println!("b info -> {:?}", b_info);

    (winner_message, a_info, b_info)
}

fn sample_principal_variant(depth:u8) {
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let init = State::initial();
    let new_state = board.apply(Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, &init);
    let mut new_state_b = board.apply(Move::PlaceBuilders { a: Slot(23), b: Slot(24) }, &new_state);
    new_state_b.buildings = new_state_b.buildings.set(Slot(5), 1);

    principal_variant::<MiniMax, SimpleHeightHeuristic>(&mut MiniMax::new_state(&board), &board, &new_state_b, depth);
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

fn do_stuff() {
    let board = StandardBoard::new(ZobristHash::new_unseeded());

	print!("{}[2J", 27 as char);
    // let mut threaded_rng = rand::thread_rng();
    // let random_seed = [threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32()];
    // let mut rng = rand::XorShiftRng::from_seed(random_seed);

    println!("Santorini!");
    print_sizes();


    for trans in &board.transforms {
        let ok = trans.check();

        println!("is transform {:?} ok -> {:?}", trans, ok);
    }

    let state = State::initial();
    let new_state = board.apply(Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, &state);

    println!("{}", board.print(&new_state));

    let mut out = Vec::new();

    board.permute(&new_state, &mut out);

    for p_state in &out {
        println!("== permuted ==");
        println!("{}", board.print(&p_state));
    }


    return;
    
 
    
}

fn print_sizes() {
    let position_size = mem::size_of::<Position>();
    println!("position size -> {}", position_size);
    let state_size = mem::size_of::<State>();
    println!("state size -> {}", state_size);
    let move_size = mem::size_of::<Move>();
    println!("move size -> {}", move_size);
    let v_size = mem::size_of::<Vec<u32>>();
    println!("v size -> {:?}", v_size);
}
