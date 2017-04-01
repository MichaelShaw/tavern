
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

pub fn pairs<T>(items: &Vec<T>) -> Vec<(T, T)> where T : Copy {
    let mut pairs = Vec::new();
    for a in 0..items.len() {
        for b in (a+1)..items.len() {
            pairs.push((items[a], items[b]));
        }
    }
    pairs
}


#[cfg(test)]
mod tests {
    use game::santorini::*;

    use super::*;

    use HashMap;

    #[test]
    fn heuristic_check() {
        use super::HeuristicName::*;

        let board = StandardBoard::new(ZobristHash::new_unseeded_secure());

        let depths : Vec<Depth> = (2..8).collect();
        let heuristics : Vec<HeuristicName> = vec![Simple, Neighbour, AdjustedNeighbour];

        let heuristic_pairs = pairs(&heuristics);

        for d in depths {
            println!("\n\n==== DEPTH {} ====", d);

            let mut won_games : HashMap<HeuristicName, u32> = HashMap::default();

            for &(a_heuristic, b_heuristic) in &heuristic_pairs {
                let a_profile = AIProfile { depth: d, heuristic: a_heuristic };
                let b_profile = AIProfile { depth: d, heuristic: b_heuristic };

                let ai_profiles = [a_profile, b_profile];
                let (a_first_winner, _)= adversarial_playout(&board, ai_profiles, |_, _, _| { });
                *won_games.entry(ai_profiles[a_first_winner.0 as usize].heuristic).or_insert(0) += 1;
                println!(".");

                let rev_ai_profiles = [b_profile, a_profile];
                let (b_first_winner, _)= adversarial_playout(&board, rev_ai_profiles, |_, _, _| { });
                *won_games.entry(rev_ai_profiles[b_first_winner.0 as usize].heuristic).or_insert(0) += 1;
                println!(".");
            }
            println!("\n");

            for (heuristic, count) in won_games {
                println!("{:?} won {} games", heuristic, count);
            }
        }
    }

    #[test]
    fn depth_check() {
        let board = StandardBoard::new(ZobristHash::new_unseeded_secure());

        let depths : Vec<Depth> = (2..8).collect();

        let depth_pairs = pairs(&depths);

        println!("how many depth pairs -> {}", depth_pairs.len());

        let mut won_games : HashMap<Depth, u32> = HashMap::default();

        for &(a_depth, b_depth) in &depth_pairs {
            let a_profile = AIProfile { depth: a_depth, heuristic: HeuristicName::AdjustedNeighbour };
            let b_profile = AIProfile { depth: b_depth, heuristic: HeuristicName::AdjustedNeighbour };

            let ai_profiles = [a_profile, b_profile];
            let (a_first_winner, _)= adversarial_playout(&board, ai_profiles, |_, _, _| { });
            *won_games.entry(ai_profiles[a_first_winner.0 as usize].depth).or_insert(0) += 1;
            
            let w = ai_profiles[a_first_winner.0 as usize].depth;
            let l = ai_profiles[((a_first_winner.0 + 1) % 2) as usize].depth;

            println!("when {} started {} beat {}", a_depth, w, l);

            let rev_ai_profiles = [b_profile, a_profile];
            let (b_first_winner, _)= adversarial_playout(&board, rev_ai_profiles, |_, _, _| { });
            *won_games.entry(rev_ai_profiles[b_first_winner.0 as usize].depth).or_insert(0) += 1;
            
            let w = rev_ai_profiles[b_first_winner.0 as usize].depth;
            let l = rev_ai_profiles[((b_first_winner.0 + 1) % 2) as usize].depth;

            println!("when {} started {} beat {}", b_depth, w, l);
        }
        println!("\n\n=== Totals ===");

        for (depth, count) in won_games {
            println!("{:?} won {} games", depth , count);
        }
    }
}
