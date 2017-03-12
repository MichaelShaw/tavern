#![allow(dead_code)]

extern crate jam;
extern crate rand;
extern crate tavern_core;
extern crate time;
extern crate tavern;



use std::mem;

use tavern_core::game::santorini::*;
use tavern_core::game::util::*; // , Packed, Packed1, Packed2, Slot};

fn main() {
    // count_moves();
    // tavern::app::run_app();
    run_test();
}

fn run_test() {
    let board = StandardBoard::new();

    let init = State::initial();
    let new_state = board.apply(Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, &init);
    let mut new_state_b = board.apply(Move::PlaceBuilders { a: Slot(23), b: Slot(24) }, &new_state);
    new_state_b.buildings = new_state_b.buildings.set(Slot(5), 1);
    println!("to move -> {:?}", new_state_b.to_move);
    new_state_b.to_move = Player(0);

    // println!("init {}", board.print(&init));
    // println!("a {}", board.print(&new_state));
    println!("start {}", board.print(&new_state_b));

    println!("close move first, expect -> 1, 1, 2, 1");

    evaluate(&board, &new_state_b);

    println!("people far away move first, expect -> 0, 1, 0, 1"); // basically Player A gets a head start
    // B NOWHERE, A UP, B UP

    new_state_b.to_move = Player(1);
    evaluate(&board, &new_state_b);
}

fn evaluate(board:&StandardBoard, state:&State) {
    for depth in 1..5 {
        let mut negamax_moves = NegaMax::evaluate::<SimpleHeightHeuristic>(board, state, depth);
        let mut minimax_moves = MiniMax::evaluate::<SimpleHeightHeuristic>(board, state, depth);
        
        // if let Some(&(mve, score)) = negamax_moves.first() {
        //     println!("==== NEGAMAX depth {:?} winning move {:?} score -> {:?} ====", depth, mve, score);
        // }
        if let Some(&(mve, score)) = minimax_moves.first() {
            println!("==== MINIMAX depth {:?} winning move {:?} score -> {:?} ====", depth, mve, score);
        }
    }
}

fn count_moves() {
    let board = StandardBoard::new();

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
    let board = StandardBoard::new();

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
