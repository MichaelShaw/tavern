
use game::santorini::*;

pub fn adversarial_playout<F>(board: &StandardBoard, ai_profiles: [AIProfile; 2], mut on_move: F) -> (Player, [EvaluatorInfo; 2]) where F : FnMut(&State, &Move, HeuristicValue) {
    let mut state = INITIAL_STATE;

    let mut winner : Option<Player> = None;

    let mut infos = [EvaluatorInfo::new(), EvaluatorInfo::new()];

    let mut evaluator_states : [EvState; 2] = [NegaMaxAlphaBetaExp::new_state(), NegaMaxAlphaBetaExp::new_state()];

    let mut move_count = 0;

    while winner == None {
        let player_idx = state.to_move.0 as usize;
        let ai_profile = ai_profiles[player_idx];

        let depth : Depth = if move_count < 2 {
            max(2, ai_profile.depth - 1)
        }  else {
            ai_profile.depth
        };

        let mut best_move : Option<(Move, HeuristicValue)> = None;
        for d in 1 ..(depth+1) {
            let (best_move_for_depth, info) = match ai_profile.heuristic {
                HeuristicName::Simple => NegaMaxAlphaBetaExp::evaluate_moves::<SimpleHeightHeuristic>(&mut evaluator_states[player_idx], &board, &state, d),
                HeuristicName::Neighbour => NegaMaxAlphaBetaExp::evaluate_moves::<NeighbourHeuristic>(&mut evaluator_states[player_idx], &board, &state, d),
                HeuristicName::AdjustedNeighbour => NegaMaxAlphaBetaExp::evaluate_moves::<AdjustedNeighbourHeuristic>(&mut evaluator_states[player_idx], &board, &state, d),
            };
            infos[player_idx] += info;
            best_move = best_move_for_depth;
        }
        
        winner = if let Some((mve, score)) = best_move {
            let is_winning_move = board.ascension_winning_move(&state, mve);
            if is_winning_move {
                let winner = state.to_move;
                state = board.apply(mve, &state);
                on_move(&state, &mve, score);
                Some(winner)
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

    (winner.unwrap(), infos)
}

fn sample_principal_variant(depth:Depth) {
    let board = StandardBoard::new(ZobristHash::new_unseeded());
    let init = INITIAL_STATE;
    let new_state = board.apply(Move::PlaceBuilders { a: Slot(0), b: Slot(1) }, &init);
    let mut new_state_b = board.apply(Move::PlaceBuilders { a: Slot(23), b: Slot(24) }, &new_state);
    new_state_b.set_building_height(Slot(5), 1);

    principal_variant::<MiniMax, SimpleHeightHeuristic>(&mut (), &board, &new_state_b, depth);
}


#[cfg(test)]
mod tests {
    use game::santorini::*;

    use super::*;

    #[test]
    fn depth_check() {
        use super::HeuristicName::*;

        let depths : Vec<Depth> = (2..7).collect();
        let heuristics : Vec<HeuristicName> = vec![Simple, Neighbour, AdjustedNeighbour];
        for d in depths {


        }


    }
}
