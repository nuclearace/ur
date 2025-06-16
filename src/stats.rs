use std::io::{self, Write};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    style::{Color, Print, ResetColor, SetForegroundColor},
    cursor::{MoveTo, Hide, Show},
};

use crate::optimized_game::{FastGameState, FastPlayer};
use crate::ai::HybridAI;
use crate::ai_helpers::{choose_random_move_fast, choose_smart_move_fast};

#[derive(Debug, Clone, Copy)]
pub enum StatsAIType {
    Random,
    Smart,
    MCTS,
}

#[derive(Debug)]
pub struct GameStatistics {
    player1_wins: usize,
    player2_wins: usize,
    total_games: usize,
    total_turns: usize,
    shortest_game: usize,
    longest_game: usize,
    total_captures_p1: usize,
    total_captures_p2: usize,
}

impl GameStatistics {
    pub fn new() -> Self {
        GameStatistics {
            player1_wins: 0,
            player2_wins: 0,
            total_games: 0,
            total_turns: 0,
            shortest_game: usize::MAX,
            longest_game: 0,
            total_captures_p1: 0,
            total_captures_p2: 0,
        }
    }

    pub fn add_game(&mut self, winner: FastPlayer, turns: usize, captures_p1: usize, captures_p2: usize) {
        match winner {
            FastPlayer::One => self.player1_wins += 1,
            FastPlayer::Two => self.player2_wins += 1,
        }
        self.total_games += 1;
        self.total_turns += turns;
        self.shortest_game = self.shortest_game.min(turns);
        self.longest_game = self.longest_game.max(turns);
        self.total_captures_p1 += captures_p1;
        self.total_captures_p2 += captures_p2;
    }

    pub fn display(&self, p1_desc: &str, p2_desc: &str) {
        println!("\n=== GAME STATISTICS ===");
        println!("Total games played: {}", self.total_games);
        println!();

        println!("WINS:");
        println!("  {} ({}): {} ({:.1}%)",
                 FastPlayer::One.name(), p1_desc, self.player1_wins,
                 (self.player1_wins as f64 / self.total_games as f64) * 100.0);
        println!("  {} ({}): {} ({:.1}%)",
                 FastPlayer::Two.name(), p2_desc, self.player2_wins,
                 (self.player2_wins as f64 / self.total_games as f64) * 100.0);
        println!();

        println!("GAME LENGTH:");
        println!("  Average turns per game: {:.1}", self.total_turns as f64 / self.total_games as f64);
        println!("  Shortest game: {} turns", self.shortest_game);
        println!("  Longest game: {} turns", self.longest_game);
        println!();

        println!("CAPTURES:");
        println!("  {} total captures: {} (avg: {:.1} per game)",
                 FastPlayer::One.name(), self.total_captures_p1,
                 self.total_captures_p1 as f64 / self.total_games as f64);
        println!("  {} total captures: {} (avg: {:.1} per game)",
                 FastPlayer::Two.name(), self.total_captures_p2,
                 self.total_captures_p2 as f64 / self.total_games as f64);
    }
}

pub fn display_running_stats(stats: &GameStatistics, current_game: usize, total_games: usize, p1_desc: &str, p2_desc: &str) {
    // Clear multiple lines to ensure we overwrite previous display
    for _ in 0..15 {
        print!("\r{}", " ".repeat(80));
        print!("\n");
    }

    // Move back to start
    let _ = execute!(io::stdout(), MoveTo(0, 0));

    let progress = (current_game as f64 / total_games as f64) * 100.0;
    let progress_bar_width = 40;
    let filled_width = ((progress / 100.0) * progress_bar_width as f64) as usize;

    // Header with progress
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           🎮 LIVE GAME STATISTICS 🎮                          ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");

    // Progress bar
    print!("║ Progress: [");
    for i in 0..progress_bar_width {
        if i < filled_width {
            let _ = execute!(io::stdout(), SetForegroundColor(Color::Green), Print("█"), ResetColor);
        } else {
            print!(" ");
        }
    }
    println!("] {:.1}% ({}/{}) ║", progress, current_game, total_games);
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");

    if stats.total_games > 0 {
        // Win statistics
        let p1_win_pct = (stats.player1_wins as f64 / stats.total_games as f64) * 100.0;
        let p2_win_pct = (stats.player2_wins as f64 / stats.total_games as f64) * 100.0;

        print!("║ ");
        let _ = execute!(io::stdout(), SetForegroundColor(Color::Blue), Print("🔵"), ResetColor);
        print!(" {} wins: {} ({:.1}%)", p1_desc, stats.player1_wins, p1_win_pct);

        // Pad to align
        let line_len = format!(" {} wins: {} ({:.1}%)", p1_desc, stats.player1_wins, p1_win_pct).len();
        let padding = 77 - line_len;
        for _ in 0..padding {
            print!(" ");
        }
        println!("║");

        print!("║ ");
        let _ = execute!(io::stdout(), SetForegroundColor(Color::Red), Print("🔴"), ResetColor);
        print!(" {} wins: {} ({:.1}%)", p2_desc, stats.player2_wins, p2_win_pct);

        // Pad to align
        let line_len = format!(" {} wins: {} ({:.1}%)", p2_desc, stats.player2_wins, p2_win_pct).len();
        let padding = 77 - line_len;
        for _ in 0..padding {
            print!(" ");
        }
        println!("║");

        println!("╠═══════════════════════════════════════════════════════════════════════════════╣");

        // Game length statistics
        let avg_turns = stats.total_turns as f64 / stats.total_games as f64;
        let avg_captures_p1 = stats.total_captures_p1 as f64 / stats.total_games as f64;
        let avg_captures_p2 = stats.total_captures_p2 as f64 / stats.total_games as f64;

        println!("║ 📊 Avg game length: {:.1} turns | Shortest: {} | Longest: {}{}║",
                avg_turns,
                if stats.shortest_game == usize::MAX { 0 } else { stats.shortest_game },
                stats.longest_game,
                " ".repeat(25));

        println!("║ ⚔️  Avg captures per game: {:.1} vs {:.1}{}║",
                avg_captures_p1, avg_captures_p2, " ".repeat(42));
    } else {
        println!("║ Waiting for first game to complete...{}║", " ".repeat(45));
        println!("║{}║", " ".repeat(79));
        println!("║{}║", " ".repeat(79));
        println!("║{}║", " ".repeat(79));
    }

    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");

    io::stdout().flush().unwrap();
}

pub fn run_statistics_menu() {
    println!("\n=== STATISTICS MENU ===");
    println!("Choose AI matchup:");
    println!("  1: Random AI vs Random AI");
    println!("  2: Random AI vs Smart AI");
    println!("  3: Random AI vs MCTS AI");
    println!("  4: Smart AI vs Random AI");
    println!("  5: Smart AI vs Smart AI");
    println!("  6: Smart AI vs MCTS AI");
    println!("  7: MCTS AI vs Random AI");
    println!("  8: MCTS AI vs Smart AI");
    println!("  9: MCTS AI vs MCTS AI");
    print!("Enter choice [1-9]: ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let matchup: usize = buf.trim().parse().unwrap_or(5);

    let (p1_type, p2_type, p1_desc, p2_desc) = match matchup {
        1 => (StatsAIType::Random, StatsAIType::Random, "Random AI", "Random AI"),
        2 => (StatsAIType::Random, StatsAIType::Smart, "Random AI", "Smart AI"),
        3 => (StatsAIType::Random, StatsAIType::MCTS, "Random AI", "MCTS AI"),
        4 => (StatsAIType::Smart, StatsAIType::Random, "Smart AI", "Random AI"),
        5 => (StatsAIType::Smart, StatsAIType::Smart, "Smart AI", "Smart AI"),
        6 => (StatsAIType::Smart, StatsAIType::MCTS, "Smart AI", "MCTS AI"),
        7 => (StatsAIType::MCTS, StatsAIType::Random, "MCTS AI", "Random AI"),
        8 => (StatsAIType::MCTS, StatsAIType::Smart, "MCTS AI", "Smart AI"),
        9 => (StatsAIType::MCTS, StatsAIType::MCTS, "MCTS AI", "MCTS AI"),
        _ => (StatsAIType::Smart, StatsAIType::Smart, "Smart AI", "Smart AI"),
    };

    println!();
    print!("Enter number of games to simulate [1-10000]: ");
    io::stdout().flush().unwrap();

    buf.clear();
    io::stdin().read_line(&mut buf).unwrap();
    let num_games: usize = buf.trim().parse().unwrap_or(100).min(10000).max(1);

    println!("\nRunning {} games: {} vs {}...", num_games, p1_desc, p2_desc);

    // Show MCTS configuration if using MCTS AI
    if matches!(p1_type, StatsAIType::MCTS) || matches!(p2_type, StatsAIType::MCTS) {
        let num_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        let mcts_info_ai = HybridAI::new_with_threads(num_cpus * 500, num_cpus); // Fewer sims for stats
        println!("MCTS Configuration: {}", mcts_info_ai.get_info());
    }

    println!();

    let mut stats = GameStatistics::new();

    // Hide cursor for cleaner display
    let _ = execute!(io::stdout(), Hide);

    // Clear screen and move to top for our display area
    let _ = execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0));
    let start_row = 0;

    for game_num in 1..=num_games {
        let (winner, turns, captures_p1, captures_p2) = run_silent_game(p1_type, p2_type);
        stats.add_game(winner, turns, captures_p1, captures_p2);

        // Update display every 10 games, or for the first few games, or at the end
        let should_update = game_num % 10 == 0 || game_num <= 5 || game_num == num_games;

        if should_update {
            // Clear the display area and show current stats
            let _ = execute!(io::stdout(), MoveTo(0, start_row));
            display_running_stats(&stats, game_num, num_games, p1_desc, p2_desc);
        }
    }

    // Show cursor again
    let _ = execute!(io::stdout(), Show);

    println!("\n✅ Simulation complete!");
    stats.display(p1_desc, p2_desc);
}

pub fn run_silent_game(p1_type: StatsAIType, p2_type: StatsAIType) -> (FastPlayer, usize, usize, usize) {
    let mut game = FastGameState::new();
    let mut turn_count = 0;
    let mut captures_p1 = 0;
    let mut captures_p2 = 0;

    // Create MCTS AI for stats (fewer simulations for speed)
    let num_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mcts_ai = HybridAI::new_with_threads(num_cpus * 400, num_cpus); // Fast MCTS for stats

    loop {
        turn_count += 1;

        // Count pieces before move for capture detection
        let p1_pieces_before = count_on_board_pieces(&game, FastPlayer::One);
        let p2_pieces_before = count_on_board_pieces(&game, FastPlayer::Two);

        let roll = FastGameState::roll_dice();

        if roll == 0 {
            // Switch turn manually since we don't have a move to make
            game.scores_and_turn ^= 1 << 6;
            continue;
        }

        let moves = game.generate_moves(roll);
        if moves.is_empty() {
            // Switch turn manually
            game.scores_and_turn ^= 1 << 6;
            continue;
        }

        let current_player = game.current_player();
        let current_ai_type = match current_player {
            FastPlayer::One => p1_type,
            FastPlayer::Two => p2_type,
        };

        let chosen_piece = match current_ai_type {
            StatsAIType::Random => choose_random_move_fast(&moves),
            StatsAIType::Smart => choose_smart_move_fast(&game, current_player, &moves, roll),
            StatsAIType::MCTS => {
                if let Some(piece_idx) = mcts_ai.choose_move(&game, current_player, roll) {
                    piece_idx
                } else {
                    choose_random_move_fast(&moves)
                }
            }
        };

        if let Some(_move_info) = game.make_move(chosen_piece, roll) {
            // Count pieces after move to detect captures
            let p1_pieces_after = count_on_board_pieces(&game, FastPlayer::One);
            let p2_pieces_after = count_on_board_pieces(&game, FastPlayer::Two);

            // If opponent lost pieces, current player made captures
            match current_player {
                FastPlayer::One => {
                    if p2_pieces_after < p2_pieces_before {
                        captures_p1 += p2_pieces_before - p2_pieces_after;
                    }
                }
                FastPlayer::Two => {
                    if p1_pieces_after < p1_pieces_before {
                        captures_p2 += p1_pieces_before - p1_pieces_after;
                    }
                }
            }

            if game.is_winner(current_player) {
                return (current_player, turn_count, captures_p1, captures_p2);
            }

            // Note: Turn switching is handled automatically by make_move() if no extra turn
        }

        // Safety valve to prevent infinite games
        if turn_count > 1000 {
            let winner = if game.get_score(FastPlayer::One) > game.get_score(FastPlayer::Two) {
                FastPlayer::One
            } else if game.get_score(FastPlayer::Two) > game.get_score(FastPlayer::One) {
                FastPlayer::Two
            } else {
                FastPlayer::One
            };
            return (winner, turn_count, captures_p1, captures_p2);
        }
    }
}

pub fn count_on_board_pieces(game: &FastGameState, player: FastPlayer) -> usize {
    let mut count = 0;
    for piece_idx in 0..7 {
        let pos = game.get_piece_pos(player, piece_idx);
        if pos >= 1 && pos <= 14 {
            count += 1;
        }
    }
    count
}