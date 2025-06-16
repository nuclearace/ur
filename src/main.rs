// General Rules of the Royal Game of Ur:
//
// The Game of Ur is a race game and is probably an ancestor of the tables family of
// games (including backgammon). The Game of Ur is played using two sets of seven game
// pieces, similar to those used in draughts or checkers. One set of pieces is white
// with five black dots and the other set is black with five white dots.
//
// The gameboard is composed of two rectangular sets of boxes, one containing three rows
// of four boxes each and the other containing three rows of two boxes each, joined by
// a "narrow bridge" of two boxes.
//
// The gameplay involves elements of both luck and strategy. Movements are determined by
// rolling a set of four-sided, tetrahedron-shaped dice. Two of the four corners of each
// die are marked and the other two are not, giving each die an equal chance of landing
// with a marked or unmarked corner facing up. The number of marked ends facing upwards
// after a roll of the dice indicates how many spaces a player may move during that turn.
// A single game can last up to half an hour.
//
// The objective of the game is for a player to move all seven of their pieces along
// the course and off the board before their opponent. On all surviving gameboards,
// the two sides of the board are always identical with each other, suggesting that
// one side of the board belongs to one player and the opposite side to the other player.
// When a piece is on one of the player's own squares, it is safe from capture.
//
// When it is on one of the eight squares in the middle of the board, the opponent's
// pieces may capture it by landing on the same space, sending the piece back off the
// board so that it must restart the course from the beginning. This means there are
// six "safe" squares and eight "combat" squares. There can never be more than one
// piece on a single square at any given time, so having too many pieces on the board
// at once can impede a player's mobility.
//
// When a player rolls a number using the dice, they may choose to move any of their
// pieces on the board or add a new piece to the board if they still have pieces that
// have not entered the game. A player is not required to capture a piece every time
// they have the opportunity. Nonetheless, players are required to move a piece whenever
// possible, even if it results in an unfavorable outcome.
//
// All surviving gameboards have a colored rosette in the middle of the center row.
// According to Finkel's reconstruction, if a piece is located on the space with the
// rosette, it is safe from capture. Finkel also states that when a piece lands on
// any of the three rosettes, the player gets an extra roll.
//
// In order to remove a piece from the board, a player must roll exactly the number
// of spaces remaining until the end of the course plus one. If the player rolls a
// number any higher or lower than this number, they may not remove the piece from
// the board. Once a player removes all their pieces off the board in this manner,
// that player wins the game.

use std::io::{self, Write};
use std::{thread, time::Duration};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};

mod ai;
mod optimized_game;
mod ai_helpers;
mod display;
mod stats;

use optimized_game::{FastGameState, FastPlayer};
use ai::HybridAI;
use ai_helpers::{choose_random_move_fast, choose_smart_move_fast};
use display::{clear_screen, display_board, print_piece_positions, print_score, global_to_coord, show_winner};
use stats::run_statistics_menu;

#[derive(Debug, Clone, Copy)]
enum AIType {
    Human,
    Random,
    Smart,
    MCTS,
}


fn main() {
    println!("=== Royal Game of Ur (Optimized Edition) ===\n");
    println!("Rules Summary:");
    println!("- Two players (Player 1 = top row, Player 2 = bottom row).");
    println!("- Each has 7 pieces off‐board initially.");
    println!("- Roll 4 binary dice => move 0..4 steps; '0' = pass turn.");
    println!("- Each piece travels a 14‐square path; exact roll to exit.");
    println!("- Capture by landing on opponent on a non‐rosette shared square.");
    println!("- Safe squares (5 total) protect from capture; rosettes (3 of them) give extra rolls.");
    println!();

    println!("Choose game mode:");
    println!("  0: Watch two smart AI bots play against each other");
    println!("  1: Play against smart AI (you are Player 1)");
    println!("  2: Two human players");
    println!("  3: Watch random AI vs smart AI");
    println!("  4: Statistics - Run multiple games and show results");
    println!("  5: Play against MCTS AI (you are Player 1)");
    println!("  6: Watch MCTS AI vs Smart AI");
    println!("  7: Watch two MCTS AI bots play against each other");
    print!("Enter choice [0-7]: ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let choice: usize = buf.trim().parse().unwrap_or(1);

    println!();

    // Handle statistics mode separately
    if choice == 4 {
        run_statistics_menu();
        return;
    }

    // Configure threading for MCTS
    let num_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    println!("System has {} logical cores available", num_cpus);

    let use_threads = if choice == 0 || choice == 5 || choice == 6 || choice == 7 {
        // For AI vs AI or human vs MCTS, ask about threading
        print!("Use multithreaded MCTS? [Y/n]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        !input.trim().to_lowercase().starts_with('n')
    } else {
        true // Default to using threads
    };

    let num_threads = if use_threads {
        print!("Number of threads to use [1-{}] (default {}): ", num_cpus * 2, num_cpus);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().parse().unwrap_or(num_cpus).min(num_cpus * 2).max(1)
    } else {
        1
    };

    let (player1_type, player2_type) = match choice {
        0 => (AIType::Smart, AIType::Smart),      // Two smart AIs
        1 => (AIType::Human, AIType::Smart),      // Human vs Smart AI
        2 => (AIType::Human, AIType::Human),      // Two humans
        3 => (AIType::Random, AIType::Smart),     // Random AI vs Smart AI
        5 => (AIType::Human, AIType::MCTS),       // Human vs MCTS AI
        6 => (AIType::MCTS, AIType::Smart),       // MCTS AI vs Smart AI
        7 => (AIType::MCTS, AIType::MCTS),        // Two MCTS AIs
        _ => (AIType::Human, AIType::Smart),      // Default: Human vs Smart AI
    };

    // Create MCTS AI instances with explicit threading configuration
    let mcts_simulations = if use_threads {
        // More simulations when using multiple threads
        num_threads * 1000
    } else {
        // Fewer simulations for single-threaded
        2000
    };

    let mcts_ai = HybridAI::new_with_threads(mcts_simulations, num_threads);

    // Show AI configuration for MCTS players
    if matches!(player1_type, AIType::MCTS) || matches!(player2_type, AIType::MCTS) {
        println!("MCTS AI Configuration: {}", mcts_ai.get_info());
        println!();
    }

    let mut game = FastGameState::new();

    loop {
        // Check for a winner at the start of the turn
        let winner = if game.is_winner(FastPlayer::One) {
            Some(FastPlayer::One)
        } else if game.is_winner(FastPlayer::Two) {
            Some(FastPlayer::Two)
        } else {
            None
        };

        if let Some(winner_player) = winner {
            show_winner(winner_player, &game);
            break;
        }

        clear_screen();
        display_board(&game);
        print_piece_positions(&game, game.current_player());
        print_score(&game);

        // Show whose turn it is with emphasis
        let current_player = game.current_player();
        let (player_color, player_symbol) = match current_player {
            FastPlayer::One => (Color::Blue, "🔵"),
            FastPlayer::Two => (Color::Red, "🔴"),
        };

        println!("┌─────────────────────────────────────┐");
        print!("│ ");
        let _ = execute!(
            io::stdout(),
            SetForegroundColor(player_color),
            Print(format!("⭐ {}'s Turn {} ⭐", current_player.name(), player_symbol)),
            ResetColor
        );
        println!("                │");
        println!("└─────────────────────────────────────┘");
        println!();

        // Check if current player is human or bot
        let current_player_type = match game.current_player() {
            FastPlayer::One => player1_type,
            FastPlayer::Two => player2_type,
        };
        let current_player_is_human = matches!(current_player_type, AIType::Human);

        // Roll dice
        if current_player_is_human {
            print!("⚡ Press ENTER to roll dice... ");
            io::stdout().flush().unwrap();
            let _ = io::stdin().read_line(&mut String::new());
        } else {
            // Bot turn - pause to show thinking
            let ai_type_name = match current_player_type {
                AIType::Random => "🎲 Random AI",
                AIType::Smart => "🧠 Smart AI",
                AIType::MCTS => "🤖 MCTS AI",
                AIType::Human => unreachable!(),
            };
            print!("🤔 {} is thinking", ai_type_name);
            for _ in 0..3 {
                thread::sleep(Duration::from_millis(300));
                print!(".");
                io::stdout().flush().unwrap();
            }
            println!();
        }

        let roll = FastGameState::roll_dice();
        print!("🎲 Rolled: ");
        let dice_color = match roll {
            0 => Color::DarkGrey,
            1 => Color::White,
            2 => Color::Yellow,
            3 => Color::Cyan,
            4 => Color::Green,
            _ => Color::White,
        };
        let _ = execute!(
            io::stdout(),
            SetForegroundColor(dice_color),
            Print(format!("{}", roll)),
            ResetColor
        );

        let dice_visual = match roll {
            0 => " (no moves)",
            1 => " 🎯",
            2 => " 🎯🎯",
            3 => " 🎯🎯🎯",
            4 => " 🎯🎯🎯🎯",
            _ => "",
        };
        println!("{}", dice_visual);

        if roll == 0 {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print("❌ No moves available. Turn passes."),
                ResetColor
            );
            println!("\n");
            thread::sleep(Duration::from_millis(1500));
            game.scores_and_turn ^= 1 << 6; // Switch turn manually
            continue;
        }

        // Compute valid moves
        let moves = game.generate_moves(roll);
        if moves.is_empty() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("❌ No legal moves with roll = {}. Turn passes.", roll)),
                ResetColor
            );
            println!("\n");
            thread::sleep(Duration::from_millis(1500));
            game.scores_and_turn ^= 1 << 6; // Switch turn manually
            continue;
        }

        let chosen_piece = if current_player_is_human {
            // Human player chooses
            println!("Legal moves:");
            for (idx, &piece_idx) in moves.iter().enumerate() {
                let pos = game.get_piece_pos(game.current_player(), piece_idx);
                match pos {
                    0 => {
                        let target_square = FastGameState::path_to_global(game.current_player(), 0);
                        let (r, c) = global_to_coord(target_square);
                        let extra_info = if FastGameState::is_rosette(target_square) {
                            ", lands on rosette (extra turn)"
                        } else if FastGameState::is_safe(target_square) {
                            ", lands on safe square"
                        } else {
                            ""
                        };
                        println!("  [{}] Enter piece {} → path 0 (grid ({}, {})){}",
                                idx, piece_idx, r, c, extra_info);
                    }
                    1..=14 => {
                        let path_idx = pos - 1;
                        let new_path_idx = path_idx + roll;
                        if new_path_idx >= 14 {
                            println!("  [{}] Move piece {} → EXIT", idx, piece_idx);
                        } else {
                            let target_square = FastGameState::path_to_global(game.current_player(), new_path_idx);
                            let (r, c) = global_to_coord(target_square);
                            let extra_info = if FastGameState::is_rosette(target_square) {
                                ", lands on rosette (extra turn)"
                            } else if FastGameState::is_safe(target_square) {
                                ", lands on safe square"
                            } else {
                                ""
                            };
                            println!("  [{}] Move piece {} → path {} (grid ({}, {})){}",
                                    idx, piece_idx, new_path_idx, r, c, extra_info);
                        }
                    }
                    _ => {}
                }
            }
            print!("Choose move index [0..{}]: ", moves.len() - 1);
            io::stdout().flush().unwrap();
            let mut inp = String::new();
            io::stdin().read_line(&mut inp).unwrap();
            let choice: usize = inp.trim().parse().unwrap_or(0).min(moves.len() - 1);
            moves[choice]
        } else {
            // Bot player chooses
            let mv = match current_player_type {
                AIType::Random => choose_random_move_fast(&moves),
                AIType::Smart => choose_smart_move_fast(&game, game.current_player(), &moves, roll),
                AIType::MCTS => {
                    if let Some(piece_idx) = mcts_ai.choose_move(&game, game.current_player(), roll) {
                        piece_idx
                    } else {
                        choose_random_move_fast(&moves)
                    }
                },
                AIType::Human => unreachable!(),
            };

            // Print which piece it moved and to where
            let ai_type = match current_player_type {
                AIType::Random => "random AI",
                AIType::Smart => "smart AI",
                AIType::MCTS => "MCTS AI",
                AIType::Human => unreachable!(),
            };

            let pos = game.get_piece_pos(game.current_player(), mv);
            match pos {
                0 => {
                    let target_square = FastGameState::path_to_global(game.current_player(), 0);
                    let (r, c) = global_to_coord(target_square);
                    let extra_info = if FastGameState::is_rosette(target_square) {
                        " (rosette - extra turn!)"
                    } else if FastGameState::is_safe(target_square) {
                        " (safe square)"
                    } else {
                        ""
                    };
                    println!("{} ({}) enters piece {} → path 0, grid ({}, {}){}",
                            game.current_player().name(), ai_type, mv, r, c, extra_info);
                }
                1..=14 => {
                    let path_idx = pos - 1;
                    let new_path_idx = path_idx + roll;
                    if new_path_idx >= 14 {
                        println!("{} ({}) moves piece {} → EXIT",
                                game.current_player().name(), ai_type, mv);
                    } else {
                        let target_square = FastGameState::path_to_global(game.current_player(), new_path_idx);
                        let (r, c) = global_to_coord(target_square);
                        let extra_info = if FastGameState::is_rosette(target_square) {
                            " (rosette - extra turn!)"
                        } else if FastGameState::is_safe(target_square) {
                            " (safe square)"
                        } else {
                            ""
                        };
                        println!("{} ({}) moves piece {} → path {}, grid ({}, {}){}",
                                game.current_player().name(), ai_type, mv, new_path_idx, r, c, extra_info);
                    }
                }
                _ => {}
            }

            // Pause so we can observe
            thread::sleep(Duration::from_millis(1000));
            mv
        };

        // Apply the chosen move
        if let Some(move_info) = game.make_move(chosen_piece, roll) {
            // Check for extra turn
            if move_info.extra_turn {
                println!("{} gets an extra roll (landed on rosette).", game.current_player().name());
                println!();
                continue;
            }

            // Turn switching is handled automatically by make_move()
        } else {
            println!("Invalid move attempt!");
            continue;
        }

        println!("Turn passes.\n");
    }
}