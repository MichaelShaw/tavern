
use game::santorini::*;
use time;
use colored::*;

pub fn mild_a_advantage(board:&StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    state.buildings = state.buildings.set(Slot(5), 1);
    state.to_move = to_move;
    state
}

pub fn distant_state(board:&StandardBoard) -> State {
    let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(23), b: Slot(24) }] {
        state = board.apply(mve, &state);
    }
    state
}

pub fn initial(board: &StandardBoard, to_move:Player) -> State {
    let mut state = State::initial();
    state.to_move = to_move;
    state
}

pub fn a_in_1(board:&StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 0
    state.buildings = state.buildings.set(Slot(1), 2);
    state.buildings = state.buildings.set(Slot(2), 3);
    state.to_move = to_move;
    state
}

pub fn a_in_2(board:&StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 0
    state.buildings = state.buildings.set(Slot(1), 1);
    state.buildings = state.buildings.set(Slot(2), 2);
    state.buildings = state.buildings.set(Slot(3), 3);
    state.to_move = to_move;
    state
}

pub fn b_in_1(board: &StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 1
    state.buildings = state.buildings.set(Slot(23), 2);
    state.buildings = state.buildings.set(Slot(22), 3);

    state.to_move = to_move;
    state
}

pub fn b_in_2(board: &StandardBoard, to_move: Player) -> State {
    let mut state = distant_state(board);
    // staircase for player 1
    state.buildings = state.buildings.set(Slot(23), 1);
    state.buildings = state.buildings.set(Slot(22), 2);
    state.buildings = state.buildings.set(Slot(21), 3);
    state.to_move = to_move;
    state
}

pub fn a_blockable(board: &StandardBoard, to_move: Player) -> State {
    let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(3), b: Slot(4) }] {
        state = board.apply(mve, &state);
    }
    state.buildings = state.buildings.set(Slot(1), 2);
    state.buildings = state.buildings.set(Slot(2), 3);
    state.buildings = state.buildings.set(Slot(3), 1); // we put the B player up a bit so it must move down to sacrifice
    state.to_move = to_move;
    state
}

pub fn b_blockable(board: &StandardBoard, to_move: Player) -> State {
    let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(3), b: Slot(4) }] {
        state = board.apply(mve, &state);
    }
    state.buildings = state.buildings.set(Slot(1), 1);
    state.buildings = state.buildings.set(Slot(2), 3);
    state.buildings = state.buildings.set(Slot(3), 2); // we put the B player up a bit so it must move down to sacrifice
    state.to_move = to_move;
    state
}

pub fn any_in_1(board:&StandardBoard, to_move: Player) -> State {
    let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(3), b: Slot(4) }] {
        state = board.apply(mve, &state);
    }
    state.buildings = state.buildings.set(Slot(1), 2);
    state.buildings = state.buildings.set(Slot(2), 3);
    state.buildings = state.buildings.set(Slot(3), 2); // we put the B player up a bit so it must move down to sacrifice
    state.to_move = to_move;
    state
}

pub fn any_trap_in_1(board:&StandardBoard, to_move: Player) -> State {
 let mut state = State::initial();
    for &mve in &vec![Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, 
                     Move::PlaceBuilders { a: Slot(3), b: Slot(4) }] {
        state = board.apply(mve, &state);
    }
    state.buildings = state.buildings.set(Slot(2), 1); // in between, so can build on top of this to block in 1
    state.buildings = state.buildings.set(Slot(5), 2); 
    state.buildings = state.buildings.set(Slot(6), 2); 
    state.buildings = state.buildings.set(Slot(7), 1); // escape route
    state.buildings = state.buildings.set(Slot(8), 2); 
    state.buildings = state.buildings.set(Slot(9), 2); 
    state.to_move = to_move;
    state
}


pub fn evaluate<E, H>(board:&StandardBoard, state:&State, max_depth: u8) -> (Vec<HeuristicValue>, MoveCount, BranchFactor) where E: Evaluation, H:Heuristic {
    let mut branch_factors = Vec::new();
    let mut total_moves = 0;
    let heuristic_values : Vec<_> = (1..(max_depth+1)).flat_map(|depth| {
        let (moves, move_count) = E::evaluate::<H>(board, state, depth);
        branch_factors.push(branch_factor(move_count, depth));
        total_moves += move_count;
        moves.iter().map(|&(_, sc)| sc).take(1).collect::<Vec<_>>()
    }).collect();
    (heuristic_values, total_moves, average(&branch_factors))
}

fn average(arr: &[f64]) -> f64 {
    let n = arr.len() as f64;
    arr.iter().fold(0.0, |acc, val| acc + val) / n
}

pub struct TestCase {
    pub name: String,
    pub state: State,
    pub scores: Vec<HeuristicValue>,
}

pub fn case(name:&str, state:State, scores:Vec<HeuristicValue>) -> TestCase {
    TestCase {
        name: name.into(),
        state: state,
        scores: scores
    }
}

pub fn test_cases(board:&StandardBoard) -> Vec<TestCase> {
    vec![
        case("init_a", initial(board, Player(0)), vec![0,0,0,0,0]),
        case("init_b", initial(board, Player(1)), vec![0,0,0,0,0]),

        case("mild_a", mild_a_advantage(board, Player(0)), vec![1,1,2,1]),
        case("mild_a", mild_a_advantage(board, Player(1)), vec![0,1,0,1]),

        case("a_in_1", a_in_1(board, Player(0)), vec![PLAYER_0_WIN, PLAYER_0_WIN] ),
        case("a_in_1", a_in_1(board, Player(1)), vec![2, PLAYER_0_WIN]),

        case("a_in_2", a_in_2(board, Player(0)), vec![2,2,PLAYER_0_WIN, PLAYER_0_WIN] ),
        case("a_in_2", a_in_2(board, Player(1)), vec![1,2,1, PLAYER_0_WIN] ),

        case("b_in_1", b_in_1(board, Player(0)), vec![-2, PLAYER_1_WIN] ),
        case("b_in_1", b_in_1(board, Player(1)), vec![PLAYER_1_WIN, PLAYER_1_WIN] ),

        case("b_in_2", b_in_2(board, Player(0)), vec![-1,-2,-1, PLAYER_1_WIN] ),
        case("b_in_2", b_in_2(board, Player(1)), vec![-2,-2,PLAYER_1_WIN, PLAYER_1_WIN] ),

        case("a_blockable", a_blockable(board, Player(0)), vec![PLAYER_0_WIN] ),
        case("a_blockable", a_blockable(board, Player(1)), vec![1, 1] ),

        case("b_blockable", b_blockable(board, Player(0)), vec![-1, -1] ),
        case("b_blockable", b_blockable(board, Player(1)), vec![PLAYER_1_WIN] ),

        case("any_in_1", any_in_1(board, Player(0)), vec![PLAYER_0_WIN] ),
        case("any_in_1", any_in_1(board, Player(1)), vec![PLAYER_1_WIN] ),

        case("any_trap_in_1", any_trap_in_1(board, Player(0)), vec![PLAYER_0_WIN] ),
        case("any_trap_in_1", any_trap_in_1(board, Player(1)), vec![PLAYER_1_WIN] ),
    ]
}

pub fn test_all_cases<E, H>(name:&str) -> (u32, MoveCount, BranchFactor) where E: Evaluation, H: Heuristic {
    

    let mut total_moves = 0;
    let mut branch_factors = Vec::new();

    println!("==== Testing {} all cases =====", name);
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let mut error_cases = 0;
    let cases = test_cases(&board);
    for case in &cases {
        println!("Testing {} to move {:?}", case.name, case.state.to_move);
        let (scores, move_count, average_branch_factor) = evaluate::<E, H>(&board, &case.state, case.scores.len() as u8);
        total_moves += move_count;
        branch_factors.push(average_branch_factor);
        if scores != case.scores {
            // playout::<E, H>(&board, &case.state, case.scores.len() as u8);
            error_cases += 1;
            println!("{}", format!("test case expected {:?} but got {:?}", case.scores, scores).red());
        } else {
            println!("{}", format!("ok {} moves {:.2} average branch factor", move_count, average_branch_factor).green());
        }
    }

    if error_cases > 0 {
        println!("{}", format!("==== {:?} had {}/{} error cases", name, error_cases, cases.len()));
    }

    (error_cases, total_moves, average(&branch_factors))
}

pub fn time_test_cases<E, H>(name: &str) -> bool where E: Evaluation, H: Heuristic {
    let start = time::precise_time_ns();
    let (v, move_count, average_branch_factor) = test_all_cases::<E, H>(name);
    let duration = (time::precise_time_ns() - start) as f64 / 1_000_000_000f64;

    if v > 0 {
        println!("{}", format!("testing {} took {:.5} seconds", name, duration).red());    
    } else {
        let moves_per_second = move_count as f64 / duration;
        println!("{}", format!("testing {} took {:.5} seconds {} moves ({:.0}/second) {:.3} average branch factor", name, duration, move_count,  moves_per_second, average_branch_factor).green());    
    }
    
    v == 0
}

pub fn time_exploration<E, H>(name:&str, depth:u8) -> MoveCount where E: Evaluation, H: Heuristic  {
    let mut total_moves = 0;
    let mut branch_factors = Vec::new();
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let cases = test_cases(&board);

    let start = time::precise_time_ns();
    for case in &cases {
        let (_, move_count, average_branch_factor) = evaluate::<E, H>(&board, &case.state, depth);
        total_moves += move_count;
        branch_factors.push(average_branch_factor);
    }

    let duration = (time::precise_time_ns() - start) as f64 / 1_000_000_000f64;
    let moves_per_second = total_moves as f64 / duration;
    println!("{}", format!("PERFORMANCE TIMING {} took {:.5} seconds {} moves ({:.1}M/second) {:.3} average branch factor", name, duration, total_moves,  moves_per_second / 1_000_000f64, average(&branch_factors)).green());    
    total_moves
}

#[cfg(test)]
mod tests {
    use game::santorini::*;
    use super::*;

    #[test]
    fn minimax_alphabeta() {
        // assert!(time_test_cases::<MiniMaxAlphaBeta, SimpleHeightHeuristic>("MiniMax_AlphaBeta"));
    }

    // #[test]
    fn negamax_alphabeta() {
        assert!(time_test_cases::<NegaMaxAlphaBeta, SimpleHeightHeuristic>("NegaMax_AlphaBeta"));
    }   

    // #[test]
    fn negamax_alphabeta_exp() {
        assert!(time_test_cases::<NegaMaxAlphaBetaExp, SimpleHeightHeuristic>("NegaMax_AlphaBeta_Exp"));
    }

    #[test]
    fn minimax() {
        // assert!(time_test_cases::<MiniMax, SimpleHeightHeuristic>("MiniMax"));
    }

    #[test]
    fn negamax() {
        // assert!(time_test_cases::<NegaMax, SimpleHeightHeuristic>("NegaMax"));
    } 

    mod bench {
        use game::santorini::tests::*;
        use game::santorini::*;

        #[test]
        fn all() {
            println!("==== PERFORMANCE TESTING =======");
            time_exploration::<MiniMax, NeighbourHeuristic>("MiniMax", 4);
            // time_exploration::<NegaMax, NeighbourHeuristic>("NegaMax", 4);
            time_exploration::<NegaMaxAlphaBeta, NeighbourHeuristic>("NegaMax_AlphaBeta", 4);
            time_exploration::<NegaMaxAlphaBetaExp, NeighbourHeuristic>("NegaMax_AlphaBeta_Exp", 4);
        }
    }
}
   

