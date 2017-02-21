#![allow(dead_code)]

extern crate jam;
extern crate rand;
extern crate tavern_core;
extern crate time;
extern crate tavern;



use std::mem;
use rand::Rng;
use rand::SeedableRng;

use tavern_core::game::santorini::{State, Move, StandardBoard};
use tavern_core::game::util::{Position}; // , Packed, Packed1, Packed2, Slot};



fn main() {
	print!("{}[2J", 27 as char);
    let mut threaded_rng = rand::thread_rng();
    let random_seed = [threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32(), threaded_rng.next_u32()];
    let mut rng = rand::XorShiftRng::from_seed(random_seed);

    println!("Santorini!");
    print_sizes();
    
    let mut info = GameInfo::empty();
    let start = time::precise_time_ns();

    let board_count = 1;

    for _ in 0..board_count {
        let mut mvs : Vec<Move> = Vec::new();
        let game_info = play_board(&mut rng, &mut mvs);
        info = combine(game_info, info);
    }

    let elapsed = time::precise_time_ns() - start;
    let seconds_elapsed = (elapsed as f64) / (1_000_000_000 as f64);
    let moves_per_second = (info.moves as f64) / seconds_elapsed;

    let branching_factor = info.moves / info.turns;

    println!("out of {} games, branching factor is {}", board_count, branching_factor);
    println!("moves observed {} in {} seconds ({} moves/second)", info.moves, seconds_elapsed, moves_per_second);


    tavern::app::run_app();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct GameInfo {
    pub moves: u64,
    pub turns: u64,
}

impl GameInfo {
    pub fn empty() -> GameInfo {
        GameInfo {
            moves: 0,
            turns: 0,
        }
    }
}

fn combine(l:GameInfo, r:GameInfo) -> GameInfo {
    GameInfo {
        moves: l.moves + r.moves,
        turns: l.turns + r.turns,
    }
}

fn play_board<R : Rng>(rng: &mut R, moves: &mut Vec<Move>) -> GameInfo {
    let board = StandardBoard::new();
    let mut state = State::initial();
    
    let mut turn_count = 0;
    let mut move_count = 0;

    for move_idx in 0..1000 {
        moves.clear();
        // println!(" :: board at move {} ", move_idx);
        // println!("{}", board.print(&state));
        board.next_moves(&state, moves);
        if moves.is_empty() {
            // println!(" :: we have a winner, no more legal moves {:?}", state.next_player());
            break;
        } else {
            // choose a moves
            let mve = moves[rng.gen_range(0, moves.len())];

            if board.ascension_winning_move(&state, mve) {
                // println!(" :: we have an ascension winner {:?} on move {:?}", state.to_move, move_idx);
                // println!("{}", board.print(&new_state));
                break;
            }

            // println!(" :: {:?} moves applying {:?}", moves.len(), mve);
            let new_state = board.apply(mve, &state);


            move_count += moves.len() as u64;
            turn_count += 1;

            
            state = new_state
        }
    }

    GameInfo {
        moves: move_count,
        turns: turn_count,
    }
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
