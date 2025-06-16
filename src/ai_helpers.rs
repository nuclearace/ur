use crate::optimized_game::{FastGameState, FastPlayer};

/// Fast AI functions that work directly with FastGameState
pub fn choose_random_move_fast(moves: &[u8]) -> u8 {
    use rand::Rng;
    let mut rng = rand::rng();
    moves[rng.random_range(0..moves.len())]
}

pub fn choose_smart_move_fast(game: &FastGameState, player: FastPlayer, moves: &[u8], roll: u8) -> u8 {
    let mut best_move = moves[0];
    let mut best_score = f64::NEG_INFINITY;

    for &piece_idx in moves {
        let score = evaluate_move_fast(game, player, piece_idx, roll);
        if score > best_score {
            best_score = score;
            best_move = piece_idx;
        }
    }

    best_move
}

pub fn evaluate_move_fast(game: &FastGameState, player: FastPlayer, piece_idx: u8, roll: u8) -> f64 {
    let pos = game.get_piece_pos(player, piece_idx);
    let mut score = 0.0;

    match pos {
        0 => {
            // Entering the board
            score += 50.0;
            // Check if we land on a rosette
            let target_square = FastGameState::path_to_global(player, 0);
            if FastGameState::is_rosette(target_square) {
                score += 200.0; // Extra turn bonus
            }
        }
        1..=14 => {
            let path_idx = pos - 1;
            let new_path_idx = path_idx + roll;

            if new_path_idx >= 14 {
                // Finishing a piece
                score += 1000.0;
                // Bonus if this wins the game
                if game.get_score(player) == 6 {
                    score += 10000.0;
                }
            } else {
                // Moving on board
                score += new_path_idx as f64 * 10.0; // Advancement bonus

                let target_square = FastGameState::path_to_global(player, new_path_idx);

                // Rosette bonus
                if FastGameState::is_rosette(target_square) {
                    score += 200.0;
                }

                // Capture bonus
                if let Some(occupant) = game.get_occupant(target_square) {
                    if occupant != player && !FastGameState::is_safe(target_square) {
                        // Find the piece being captured to get its advancement bonus
                        for i in 0..7 {
                            let opp_pos = game.get_piece_pos(occupant, i);
                            if opp_pos >= 1 && opp_pos <= 14 {
                                let opp_square = FastGameState::path_to_global(occupant, opp_pos - 1);
                                if opp_square == target_square {
                                    score += 150.0 + ((opp_pos - 1) as f64 * 5.0);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    score
}