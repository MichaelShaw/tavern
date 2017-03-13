
use game::santorini::*;
use time;

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


pub fn evaluate<E, H>(board:&StandardBoard, state:&State, max_depth: u8) -> Vec<HeuristicValue> where E: Evaluation, H:Heuristic {
    (1..(max_depth+1)).flat_map(|depth| {
        E::evaluate::<H>(board, state, depth).iter().map(|&(_, sc)| sc).take(1).collect::<Vec<_>>()
    }).collect()
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

pub fn test_all_cases<E, H>(name:&str) -> bool where E: Evaluation, H: Heuristic {
    use colored::*;

    println!("==== Testing {} all cases =====", name);
    let board = StandardBoard::new();
    let mut error_cases = 0;
    let cases = test_cases(&board);
    for case in &cases {
        println!("Testing {} to move {:?}", case.name, case.state.to_move);
        let scores = evaluate::<E, H>(&board, &case.state, case.scores.len() as u8);

        if scores != case.scores {
            playout::<E, H>(&board, &case.state, case.scores.len() as u8);
            error_cases += 1;
            println!("{}", format!("test case expected {:?} but got {:?}", case.scores, scores).red());
        } else {
            println!("{}", "ok".green());
        }
    }

    if error_cases > 0 {
        println!("{}", format!("==== {:?} had {}/{} error cases", name, error_cases, cases.len()));
    }

    error_cases == 0
}

pub fn time_test_cases<E, H>(name: &str) -> bool where E: Evaluation, H: Heuristic {
    let start = time::precise_time_ns();
    let v = test_all_cases::<E, H>(name);
    let duration = (time::precise_time_ns() - start) as f64 / 1_000_000_000f64;

    println!("testing {} took {:.5} seconds", name, duration);
    v
}

mod minimax {
    use super::*;

    #[test]
    fn all() {
        assert!(test_all_cases::<MiniMax, SimpleHeightHeuristic>("MiniMax"));
    }

    #[test]
    fn bench() {
        let errors = time_test_cases::<MiniMax, SimpleHeightHeuristic>("MiniMax");
        println!("errors {:?}", errors);
    }
}


mod negamax {
    use super::*;

   #[test]
    fn all() {
        assert!(test_all_cases::<NegaMax, SimpleHeightHeuristic>("NegaMax"));
    }

    #[test]
    fn bench() {
        let errors = time_test_cases::<NegaMax, SimpleHeightHeuristic>("NegaMax");
        println!("errors {:?}", errors);
    }
}
    

