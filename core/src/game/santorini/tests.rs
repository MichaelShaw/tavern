
use game::santorini::*;
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
        case("init_a", initial(board, Player(0)), vec![0,0,0,0]),
        case("init_b", initial(board, Player(1)), vec![0,0,0,0]),

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

pub fn focus_test_cases(board:&StandardBoard) -> Vec<TestCase> {
    vec![
        // case("a_in_1", a_in_1(board, Player(0)), vec![PLAYER_0_WIN, PLAYER_0_WIN] ),
        case("a_in_1", a_in_1(board, Player(1)), vec![2, PLAYER_0_WIN]),
        // case("a_in_2", a_in_2(board, Player(0)), vec![2,2,PLAYER_0_WIN, PLAYER_0_WIN] ),
        case("a_in_2", a_in_2(board, Player(1)), vec![1,2,1, PLAYER_0_WIN] ),
    ]
}

pub fn evaluate_state<E, H>(evaluator_state: &mut E::EvaluatorState, board:&StandardBoard, state:&State, max_depth: u8) -> (Vec<HeuristicValue>, EvaluatorInfo) where E: Evaluator, H:Heuristic {
    let mut info = EvaluatorInfo::new();
    let heuristic_values : Vec<_> = (1..(max_depth+1)).flat_map(|depth| {
        let (moves, new_info) = E::evaluate_moves::<H>(evaluator_state, board, state, depth);
        info += new_info;
        moves.iter().map(|&(_, sc)| sc).take(1).collect::<Vec<_>>()
    }).collect();
    (heuristic_values,info)
}
pub fn test_all_cases<E, H>() -> (u32, EvaluatorInfo) where E: Evaluator, H: Heuristic {
    println!("==== Testing {} all cases =====", E::name());
    let mut info = EvaluatorInfo::new();
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let mut error_cases = 0;
    let cases = test_cases(&board);

    let mut evaluator_state = E::new_state();
    
    for case in &cases {
        println!("Testing {} to move {:?}", case.name, case.state.to_move);

        let (scores, new_info) = evaluate_state::<E, H>(&mut evaluator_state, &board, &case.state, case.scores.len() as u8);
        info += new_info.clone();


        if scores != case.scores {
            // playout::<E, H>(&board, &case.state, case.scores.len() as u8);
            error_cases += 1;
            println!("{}", format!("test case expected {:?} but got {:?}", case.scores, scores).red());
        } else {
            println!("{}", format!("ok -> {:?}", new_info).green());
        }
    }

    if error_cases > 0 {
        println!("{}", format!("==== {:?} had {}/{} error cases", E::name(), error_cases, cases.len()));
    }

    (error_cases, info)
}

pub fn time_test_cases<E, H>() -> bool where E: Evaluator, H: Heuristic {

    let (v, info) = test_all_cases::<E, H>();



    if v > 0 {
        println!("{}", format!("testing {} info {:?}", E::name(), info).red());    
    } else {
        println!("{}", format!("testing {} info -> {:?}", E::name(), info).green());    
    }
    
    v == 0
}

pub fn time_exploration<E, H>(depth:u8) -> EvaluatorInfo where E: Evaluator, H: Heuristic  {
    let mut info = EvaluatorInfo::new();
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let cases = test_cases(&board);

    let mut evaluator_state = E::new_state();

    for case in &cases {
        let (_, new_info) = evaluate_state::<E, H>(&mut evaluator_state, &board, &case.state, depth);
        info += new_info;
    }

    println!("{}", format!("PERFORMANCE TIMING {} info -> {:?}", E::name(), info).green());    
    info
}


fn evaluate_cross_state(board: &StandardBoard, state:&State, depth: u8) {
    println!("test case state");
    println!("{}", board.print(&state));


    let (minimax_best_move, minimax_info) = MiniMax::evaluate_moves_impl::<SimpleHeightHeuristic>(&mut (), &board, &state, depth);

    println!("\n\n=== MINIMAX ===");
    println!("\nmoves -> {:?}",minimax_best_move);
    println!("\ninfo -> {:?}", minimax_info);

    // let (negamax_moves, negamax_info) = NegaMax::evaluate_moves_impl::<SimpleHeightHeuristic>(&board, &state, depth);
    // let negamax_winners = winners(&negamax_moves);
    // println!("\n\n=== NEGAMAX ===");
    // println!("\nmoves -> {:?}", negamax_moves);
    // println!("\ninfo -> {:?}", negamax_info);

    let (minimax_alphabeta_best_move, minimax_alphabeta_info) = MiniMaxAlphaBeta::evaluate_moves_impl::<SimpleHeightHeuristic>(&mut (), &board, &state, depth);

    println!("\n\n=== MINIMAX ALPHABETA ===");
    println!("\nmoves -> {:?}", minimax_alphabeta_best_move);
    println!("\ninfo -> {:?}", minimax_alphabeta_info);

    // let (negamax_alphabeta_moves, negamax_alphabeta_info) = NegaMaxAlphaBeta::evaluate_moves_impl::<SimpleHeightHeuristic>(&board, &state, depth);
    // let negamax_alphabeta_winners = winners(&negamax_alphabeta_moves);
    // println!("\n\n=== NEGAMAX ALPHABETA ===");
    // println!("\ninfo -> {:?}", negamax_alphabeta_info);
    
}




#[cfg(test)]
mod tests {
    // use game::santorini::*;
    use super::*;

    #[test]
    fn test_adverserial_playout() {
        let board = StandardBoard::new(ZobristHash::new_unseeded_secure());
        // let board = StandardBoard::new(ZobristHash::new_unseeded());
        let depth = 6;

        let mut move_number = 0;

        println!("starting negamax_ab_exp adversarial playout");

        let (winner, a_info, b_info) = iterative_adversarial_playout::<NegaMaxAlphaBetaExp, NeighbourHeuristic, _>(&board, depth, |state, mve, score| {
            move_number += 1;

            let h = NeighbourHeuristic::evaluate(&board, state);
            println!("======= MOVE {} =======", move_number);
            println!("{:?} makes {:?} with expected score {}", state.next_player(), mve, score);
            println!("{}", board.print(state));
            println!("heuristic current state -> {}", h);
            println!("");
        });

        println!("winner -> {:?}", winner);
        println!("a info -> {:?}", a_info);
        println!("b info -> {:?}", b_info);
    }

    // #[test]
    fn test_adverserial_playout_old() {
        let board = StandardBoard::new(ZobristHash::new_unseeded());
        let depth = 5;

        let mut move_number = 0;

        println!("===== OLD starting negamax_ab ORIG adversarial playout =====");

        let (winner, a_info, b_info) = iterative_adversarial_playout::<NegaMaxAlphaBeta, NeighbourHeuristic, _>(&board, depth, |state, mve, score| {
            move_number += 1;

            let h = NeighbourHeuristic::evaluate(&board, state);
            println!("======= OLD MOVE {} =======", move_number);
            println!("{:?} makes {:?} with expected score {}", state.next_player(), mve, score);
            println!("{}", board.print(state));
            println!("heuristic current state -> {}", h);
            println!("");
        });

        println!("==== OLD =====");
        println!("winner -> {:?}", winner);
        println!("a info -> {:?}", a_info);
        println!("b info -> {:?}", b_info);
    }

    // #[test]
    fn minimax_vs_negamax() {
        let board = StandardBoard::new(ZobristHash::new_unseeded());
        let depth = 4;
        evaluate_cross_state(&board, &a_blockable(&board, Player(1)), depth);
        // evaluate_cross_state(&board, &b_blockable(&board, Player(0)), depth);
        
    }

    // #[test]
    fn minimax_alphabeta() {
        assert!(time_test_cases::<MiniMaxAlphaBeta, SimpleHeightHeuristic>());
    }

    // #[test]
    fn negamax_alphabeta() {
        assert!(time_test_cases::<NegaMaxAlphaBeta, SimpleHeightHeuristic>());
    }   

    #[test]
    fn negamax_alphabeta_exp() {
        assert!(time_test_cases::<NegaMaxAlphaBetaExp, SimpleHeightHeuristic>());
    }

    // #[test]
    fn minimax() {
        assert!(time_test_cases::<MiniMax, SimpleHeightHeuristic>());

    }

    // #[test]
    fn negamax() {
        assert!(time_test_cases::<NegaMax, SimpleHeightHeuristic>());
    } 

    mod bench {
        use game::santorini::tests::*;
        // use game::santorini::*;

        // #[test]
        fn all() {
           println!("==== PERFORMANCE TESTING =======");
           time_exploration::<MiniMax, NeighbourHeuristic>(3);
           time_exploration::<NegaMax, NeighbourHeuristic>(4);
           time_exploration::<NegaMaxAlphaBeta, NeighbourHeuristic>(4);
           time_exploration::<NegaMaxAlphaBetaExp, NeighbourHeuristic>( 4);
        }
    }
}
   

