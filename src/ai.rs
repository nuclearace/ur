use std::collections::HashMap;
use std::f64::consts::SQRT_2;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::optimized_game::{FastGameState, FastPlayer};

/// Optimized MCTS implementation using FastGameState with make/unmake moves
pub struct MCTSAI {
    /// Number of simulations to run
    pub simulations: usize,
    /// Exploration constant for UCB1
    pub exploration_constant: f64,
    /// Maximum depth for simulations
    pub max_simulation_depth: usize,
    /// Number of threads to use for parallel simulation
    pub num_threads: usize,
}

#[derive(Debug, Clone)]
struct MoveStats {
    visits: usize,
    wins: f64,
}

impl MoveStats {
    fn new() -> Self {
        MoveStats { visits: 0, wins: 0.0 }
    }

    fn add(&mut self, other: &MoveStats) {
        self.visits += other.visits;
        self.wins += other.wins;
    }
}

impl MCTSAI {
    pub fn new_with_threads(simulations: usize, exploration_constant: f64, num_threads: usize) -> Self {
        MCTSAI {
            simulations,
            exploration_constant,
            max_simulation_depth: 200,
            num_threads: num_threads.max(1),
        }
    }

    /// Choose the best move using optimized MCTS with make/unmake moves
    pub fn choose_move(
        &self,
        game_state: &FastGameState,
        player: FastPlayer,
        roll: u8,
    ) -> Option<u8> {
        let moves = game_state.generate_moves(roll);
        if moves.is_empty() {
            return None;
        }

        // For single move, just return it
        if moves.len() == 1 {
            return Some(moves[0]);
        }

        // Use multithreading for complex decisions
        let best_piece_idx = if self.num_threads > 1 && self.simulations >= self.num_threads * 10 {
            self.choose_move_parallel(game_state, player, roll, &moves)
        } else {
            self.choose_move_sequential(game_state, player, roll, &moves)
        };

        Some(best_piece_idx)
    }

    fn choose_move_parallel(
        &self,
        game_state: &FastGameState,
        player: FastPlayer,
        roll: u8,
        moves: &[u8],
    ) -> u8 {
        let simulations_per_thread = self.simulations / self.num_threads;
        let extra_simulations = self.simulations % self.num_threads;

        // Shared results that threads will write to
        let combined_stats = Arc::new(Mutex::new(HashMap::<u8, MoveStats>::new()));

        // Initialize combined stats
        {
            let mut stats = combined_stats.lock().unwrap();
            for &piece_idx in moves {
                stats.insert(piece_idx, MoveStats::new());
            }
        }

        let fast_state = Arc::new(*game_state);
        let moves = Arc::new(moves.to_vec());

        // Spawn worker threads
        let mut handles = vec![];

        for thread_id in 0..self.num_threads {
            let fast_state = Arc::clone(&fast_state);
            let moves = Arc::clone(&moves);
            let combined_stats = Arc::clone(&combined_stats);

            // Give some threads one extra simulation to handle remainder
            let thread_simulations = if thread_id < extra_simulations {
                simulations_per_thread + 1
            } else {
                simulations_per_thread
            };

            let exploration_constant = self.exploration_constant;
            let max_depth = self.max_simulation_depth;

            let handle = thread::spawn(move || {
                // Run MCTS simulations for this thread
                let mut local_stats = HashMap::<u8, MoveStats>::new();
                for &piece_idx in moves.iter() {
                    local_stats.insert(piece_idx, MoveStats::new());
                }

                for _ in 0..thread_simulations {
                    // Select move using UCB1
                    let selected_piece = Self::select_move_ucb1_static(&moves, &local_stats, exploration_constant);

                    // Simulate game from this move using make/unmake
                    let win_value = Self::simulate_move_fast(*fast_state, player, selected_piece, roll, max_depth);

                    // Update local statistics
                    let stats = local_stats.get_mut(&selected_piece).unwrap();
                    stats.visits += 1;
                    stats.wins += win_value;
                }

                // Merge local results into combined results
                let mut combined = combined_stats.lock().unwrap();
                for (piece_idx, local_stat) in local_stats {
                    combined.get_mut(&piece_idx).unwrap().add(&local_stat);
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Select best move from combined results
        let stats = combined_stats.lock().unwrap();
        *moves.iter()
            .max_by(|&&a, &&b| {
                let stats_a = &stats[&a];
                let stats_b = &stats[&b];
                let win_rate_a = if stats_a.visits > 0 { stats_a.wins / stats_a.visits as f64 } else { 0.0 };
                let win_rate_b = if stats_b.visits > 0 { stats_b.wins / stats_b.visits as f64 } else { 0.0 };
                win_rate_a.partial_cmp(&win_rate_b).unwrap()
            })
            .unwrap()
    }

    fn choose_move_sequential(
        &self,
        game_state: &FastGameState,
        player: FastPlayer,
        roll: u8,
        moves: &[u8],
    ) -> u8 {
        // Initialize move statistics
        let mut move_stats: HashMap<u8, MoveStats> = HashMap::new();
        for &piece_idx in moves {
            move_stats.insert(piece_idx, MoveStats::new());
        }

        // Run simulations
        for _ in 0..self.simulations {
            // Select move using UCB1
            let selected_piece = self.select_move_ucb1(moves, &move_stats);

            // Simulate game from this move using make/unmake
            let win_value = Self::simulate_move_fast(*game_state, player, selected_piece, roll, self.max_simulation_depth);

            // Update statistics
            let stats = move_stats.get_mut(&selected_piece).unwrap();
            stats.visits += 1;
            stats.wins += win_value;
        }

        // Select move with highest win rate
        *moves.iter()
            .max_by(|&&a, &&b| {
                let stats_a = &move_stats[&a];
                let stats_b = &move_stats[&b];
                let win_rate_a = if stats_a.visits > 0 { stats_a.wins / stats_a.visits as f64 } else { 0.0 };
                let win_rate_b = if stats_b.visits > 0 { stats_b.wins / stats_b.visits as f64 } else { 0.0 };
                win_rate_a.partial_cmp(&win_rate_b).unwrap()
            })
            .unwrap()
    }

    fn select_move_ucb1(
        &self,
        moves: &[u8],
        move_stats: &HashMap<u8, MoveStats>,
    ) -> u8 {
        Self::select_move_ucb1_static(moves, move_stats, self.exploration_constant)
    }

    fn select_move_ucb1_static(
        moves: &[u8],
        move_stats: &HashMap<u8, MoveStats>,
        exploration_constant: f64,
    ) -> u8 {
        let total_visits: usize = move_stats.values().map(|s| s.visits).sum();

        *moves.iter()
            .max_by(|&&a, &&b| {
                let stats_a = &move_stats[&a];
                let stats_b = &move_stats[&b];

                let ucb1_a = Self::calculate_ucb1_static(stats_a, total_visits, exploration_constant);
                let ucb1_b = Self::calculate_ucb1_static(stats_b, total_visits, exploration_constant);

                ucb1_a.partial_cmp(&ucb1_b).unwrap()
            })
            .unwrap()
    }

    fn calculate_ucb1_static(stats: &MoveStats, total_visits: usize, exploration_constant: f64) -> f64 {
        if stats.visits == 0 {
            return f64::INFINITY;
        }

        let exploitation = stats.wins / stats.visits as f64;
        let exploration = exploration_constant *
            ((total_visits as f64).ln() / stats.visits as f64).sqrt();

        exploitation + exploration
    }

    /// Ultra-fast simulation using make/unmake moves - NO ALLOCATIONS!
    fn simulate_move_fast(
        initial_state: FastGameState,
        initial_player: FastPlayer,
        piece_idx: u8,
        roll: u8,
        max_depth: usize,
    ) -> f64 {
        let mut game_state = initial_state;

        // Make the initial move
        if let Some(_move_info) = game_state.make_move(piece_idx, roll) {
            // Check for immediate win
            if game_state.is_winner(initial_player) {
                return 1.0;
            }

            // Simulate rest of game
            let result = Self::simulate_game_fast(game_state, initial_player, max_depth);

            // No need to unmake the initial move since we're working with a copy
            result
        } else {
            0.0 // Invalid move
        }
    }

    fn simulate_game_fast(
        mut game_state: FastGameState,
        initial_player: FastPlayer,
        max_depth: usize,
    ) -> f64 {
        let mut moves_stack = Vec::with_capacity(max_depth);

        for _ in 0..max_depth {
            let current_player = game_state.current_player();

            // Check for terminal state
            if game_state.is_winner(FastPlayer::One) {
                // Unmake all moves in reverse order
                for (player, move_info) in moves_stack.into_iter().rev() {
                    game_state.unmake_move(player, &move_info);
                }
                return if initial_player == FastPlayer::One { 1.0 } else { 0.0 };
            }
            if game_state.is_winner(FastPlayer::Two) {
                // Unmake all moves in reverse order
                for (player, move_info) in moves_stack.into_iter().rev() {
                    game_state.unmake_move(player, &move_info);
                }
                return if initial_player == FastPlayer::Two { 1.0 } else { 0.0 };
            }

            let sim_roll = FastGameState::roll_dice();
            if sim_roll == 0 {
                continue; // Game handles turn switching internally
            }

            let sim_moves = game_state.generate_moves(sim_roll);
            if sim_moves.is_empty() {
                continue; // Game handles turn switching internally
            }

            // Choose move (70% smart-ish, 30% random for variety)
             let chosen_piece = if rand::random::<f64>() < 0.7 {
                 // Simple heuristic: prefer moves that advance pieces furthest or finish pieces
                 Self::choose_smart_piece(&game_state, current_player, &sim_moves, sim_roll)
             } else {
                 // Random move
                 use rand::Rng;
                 let mut rng = rand::rng();
                 sim_moves[rng.random_range(0..sim_moves.len())]
             };

            // Make move
            if let Some(move_info) = game_state.make_move(chosen_piece, sim_roll) {
                moves_stack.push((current_player, move_info));

                // Check for win after move
                if game_state.is_winner(current_player) {
                    // Unmake all moves in reverse order
                    for (player, move_info) in moves_stack.into_iter().rev() {
                        game_state.unmake_move(player, &move_info);
                    }
                    return if initial_player == current_player { 1.0 } else { 0.0 };
                }
            } else {
                break; // Invalid move, end simulation
            }
        }

        // Unmake all moves in reverse order
        for (player, move_info) in moves_stack.into_iter().rev() {
            game_state.unmake_move(player, &move_info);
        }

        // Evaluate final position based on progress
        let our_score = game_state.get_score(initial_player) as f64;
        let opp_score = game_state.get_score(initial_player.opposite()) as f64;

        ((our_score + (7.0 - opp_score)) / 14.0).clamp(0.0, 1.0)
    }

    /// Simple heuristic for choosing good moves during simulation
    pub fn choose_smart_piece(game_state: &FastGameState, player: FastPlayer, moves: &[u8], roll: u8) -> u8 {
        let mut best_piece = moves[0];
        let mut best_score = f64::NEG_INFINITY;

        for &piece_idx in moves {
            let pos = game_state.get_piece_pos(player, piece_idx);
            let mut score = 0.0;

            match pos {
                0 => score = 10.0, // Entering is good
                1..=14 => {
                    let path_idx = pos - 1;
                    let new_path_idx = path_idx + roll;

                    if new_path_idx >= 14 {
                        score = 50.0; // Finishing is excellent
                    } else {
                        score = new_path_idx as f64; // Advancing is good

                        // Check if we land on a rosette
                        let target_square = FastGameState::path_to_global(player, new_path_idx);
                        if FastGameState::is_rosette(target_square) {
                            score += 5.0; // Rosettes are good
                        }

                        // Check for captures
                        if let Some(occupant) = game_state.get_occupant(target_square) {
                            if occupant != player && !FastGameState::is_safe(target_square) {
                                score += 8.0; // Captures are very good
                            }
                        }
                    }
                }
                _ => {}
            }

            if score > best_score {
                best_score = score;
                best_piece = piece_idx;
            }
        }

        best_piece
    }

    /// Get information about the threading configuration
    pub fn get_thread_info(&self) -> String {
        format!("FastMCTS: {} threads, {} simulations ({} per thread)",
                self.num_threads,
                self.simulations,
                self.simulations / self.num_threads)
    }
}

/// Enhanced AI that combines MCTS with the existing evaluation function
pub struct HybridAI {
    pub mcts: MCTSAI,
    pub use_mcts_threshold: usize, // Use MCTS only if there are this many or more moves
}

impl HybridAI {
    pub fn new_with_threads(mcts_simulations: usize, num_threads: usize) -> Self {
        HybridAI {
            mcts: MCTSAI::new_with_threads(mcts_simulations, SQRT_2, num_threads),
            use_mcts_threshold: 2,
        }
    }

    /// Choose the best move using hybrid approach
    pub fn choose_move(
        &self,
        game_state: &FastGameState,
        player: FastPlayer,
        roll: u8,
    ) -> Option<u8> {
        let moves = game_state.generate_moves(roll);
        if moves.is_empty() {
            return None;
        }

        if moves.len() == 1 {
            return Some(moves[0]);
        }

        if moves.len() >= self.use_mcts_threshold {
            // Use optimized MCTS for complex decisions
            self.mcts.choose_move(game_state, player, roll)
        } else {
            // Use simple depth-1 evaluation for simple decisions
            Some(MCTSAI::choose_smart_piece(game_state, player, &moves, roll))
        }
    }

    /// Get information about the MCTS configuration
    pub fn get_info(&self) -> String {
        format!("HybridAI: {}, MCTS threshold: {} moves",
                self.mcts.get_thread_info(),
                self.use_mcts_threshold)
    }
}

