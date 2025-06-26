use std::io;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    style::{Color, ResetColor, SetForegroundColor, SetBackgroundColor, Print},
    cursor::MoveTo,
};

use crate::optimized_game::{FastGameState, FastPlayer};

pub fn clear_screen() {
    let _ = execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0));
}

pub fn display_board(game: &FastGameState) {
    // Build a 3×8 grid representation with colors
    let mut grid: [[char; 8]; 3] = [[' '; 8]; 3];
    let mut grid_colors: [[Color; 8]; 3] = [[Color::Reset; 8]; 3];
    let mut grid_bg_colors: [[Color; 8]; 3] = [[Color::Reset; 8]; 3];

    // Initialize empty squares
    let valid_squares = [
        (0, 0), (0, 1), (0, 2), (0, 3), (0, 6), (0, 7),  // Top row
        (1, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7),  // Middle row
        (2, 0), (2, 1), (2, 2), (2, 3), (2, 6), (2, 7),  // Bottom row
    ];

    // Mark safe squares and rosettes with colors
    for &(row, col) in &valid_squares {
        let global = coord_to_global(row, col);
        if let Some(g) = global {
            if FastGameState::is_rosette(g) {
                grid[row][col] = '★';
                grid_colors[row][col] = Color::Yellow;
                grid_bg_colors[row][col] = Color::DarkMagenta;
            } else if FastGameState::is_safe(g) {
                grid[row][col] = '▣';
                grid_colors[row][col] = Color::Green;
                grid_bg_colors[row][col] = Color::DarkGreen;
            } else {
                grid[row][col] = '·';
                grid_colors[row][col] = Color::DarkGrey;
            }
        }
    }

    // Place pieces with distinct colors
    for player in [FastPlayer::One, FastPlayer::Two] {
        let (symbol, color) = match player {
            FastPlayer::One => ('●', Color::Blue),
            FastPlayer::Two => ('●', Color::Red),
        };

        for piece_idx in 0..7 {
            let pos = game.get_piece_pos(player, piece_idx);
            if pos >= 1 && pos <= 14 {
                let global_square = FastGameState::path_to_global(player, pos - 1);
                let (row, col) = global_to_coord(global_square);
                grid[row][col] = symbol;
                grid_colors[row][col] = color;
            }
        }
    }

    // Display the enhanced board
    println!("\n╔═══════════════════════════════════════╗");
    println!("║        🏛️  Royal Game of Ur  🏛️         ║");
    println!("╠═══════════════════════════════════════╣");
    print!("║     ");
    for col in 0..8 {
        print!("{} ", col);
    }
    println!("     ║");
    println!("╠═══════════════════════════════════════╣");

    for (row, line) in grid.iter().enumerate() {
        print!("║  {} │ ", row);
        for (col, &cell) in line.iter().enumerate() {
            if valid_squares.contains(&(row, col)) {
                let _ = execute!(
                    io::stdout(),
                    SetForegroundColor(grid_colors[row][col]),
                    SetBackgroundColor(grid_bg_colors[row][col]),
                    Print(cell),
                    ResetColor,
                    Print(" ")
                );
            } else {
                print!("  ");
            }
        }
        println!("│  ║");
    }
    println!("╚═══════════════════════════════════════╝");
    println!();
}

pub fn coord_to_global(row: usize, col: usize) -> Option<u8> {
    match (row, col) {
        (0, 0) => Some(0),   (0, 1) => Some(1),   (0, 2) => Some(2),   (0, 3) => Some(3),
        (0, 6) => Some(4),   (0, 7) => Some(5),
        (1, 0) => Some(6),   (1, 1) => Some(7),   (1, 2) => Some(8),   (1, 3) => Some(9),
        (1, 4) => Some(10),  (1, 5) => Some(11),  (1, 6) => Some(12),  (1, 7) => Some(13),
        (2, 0) => Some(14),  (2, 1) => Some(15),  (2, 2) => Some(16),  (2, 3) => Some(17),
        (2, 6) => Some(18),  (2, 7) => Some(19),
        _ => None,
    }
}

pub fn global_to_coord(global: u8) -> (usize, usize) {
    match global {
        0 => (0, 0),   1 => (0, 1),   2 => (0, 2),   3 => (0, 3),
        4 => (0, 6),   5 => (0, 7),
        6 => (1, 0),   7 => (1, 1),   8 => (1, 2),   9 => (1, 3),
        10 => (1, 4),  11 => (1, 5),  12 => (1, 6),  13 => (1, 7),
        14 => (2, 0),  15 => (2, 1),  16 => (2, 2),  17 => (2, 3),
        18 => (2, 6),  19 => (2, 7),
        _ => (0, 0), // Default fallback
    }
}

pub fn print_piece_positions(game: &FastGameState, player: FastPlayer) {
    let (player_color, player_symbol) = match player {
        FastPlayer::One => (Color::Blue, "🔵"),
        FastPlayer::Two => (Color::Red, "🔴"),
    };

    let _ = execute!(
        io::stdout(),
        SetForegroundColor(player_color),
        Print(format!("{} {}'s pieces:", player_symbol, player.name())),
        ResetColor
    );
    println!();

    let mut off_board = 0;
    let mut on_board = Vec::new();
    let mut finished = 0;

    for piece_idx in 0..7 {
        let pos = game.get_piece_pos(player, piece_idx);
        match pos {
            0 => off_board += 1,
            15 => finished += 1,
            1..=14 => {
                let path_idx = pos - 1;
                on_board.push((piece_idx, path_idx));
            }
            _ => {}
        }
    }

    // Summary line
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::DarkGrey),
        Print(format!("  📊 Off board: {} | On board: {} | Finished: {}",
               off_board, on_board.len(), finished)),
        ResetColor
    );
    println!();

    // Details for pieces on board
    if !on_board.is_empty() {
        on_board.sort_by_key(|(_, path_idx)| *path_idx);
        print!("  🎯 Active pieces: ");
        for (i, (piece_idx, path_idx)) in on_board.iter().enumerate() {
            if i > 0 { print!(" | "); }
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(player_color),
                Print(format!("#{} at path {}", piece_idx, path_idx)),
                ResetColor
            );
        }
        println!();
    }
    println!();
}

pub fn print_score(game: &FastGameState) {
    let p1_score = game.get_score(FastPlayer::One);
    let p2_score = game.get_score(FastPlayer::Two);

    println!("╔═══════════════════════════════════════╗");
    print!("║ 🏆 SCORE: ");

    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Blue),
        Print("🔵"),
        ResetColor,
        Print(format!(" {} = ", FastPlayer::One.name())),
        SetForegroundColor(if p1_score > p2_score { Color::Green } else { Color::White }),
        Print(format!("{}", p1_score)),
        ResetColor,
        Print(" | "),
        SetForegroundColor(Color::Red),
        Print("🔴"),
        ResetColor,
        Print(format!(" {} = ", FastPlayer::Two.name())),
        SetForegroundColor(if p2_score > p1_score { Color::Green } else { Color::White }),
        Print(format!("{}", p2_score)),
        ResetColor
    );

    // Pad to align with box
    let padding = 39 - 11 - FastPlayer::One.name().len() - FastPlayer::Two.name().len() - 8;
    for _ in 0..padding {
        print!(" ");
    }
    println!("║");
    println!("╚═══════════════════════════════════════╝");
    println!();
}


pub fn show_winner(winner: FastPlayer, game: &FastGameState) {
    clear_screen();
    display_board(game);

    let (winner_color, winner_symbol) = match winner {
        FastPlayer::One => (Color::Blue, "🔵"),
        FastPlayer::Two => (Color::Red, "🔴"),
    };

    println!("\n╔═══════════════════════════════════════╗");
    println!("║                                       ║");
    print!("║          🎉 VICTORY! 🎉             ║\n");
    print!("║                                       ║\n");
    print!("║   ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(winner_color),
        Print(format!("{} {} WINS!", winner_symbol, winner.name())),
        ResetColor
    );
    println!("                ║");
    println!("║                                       ║");
    println!("║     All 7 pieces successfully        ║");
    println!("║     completed the journey! 🏁        ║");
    println!("║                                       ║");
    println!("╚═══════════════════════════════════════╝");
}

/// Display AI search progress in a formatted way
pub fn display_search_progress(ai_name: &str, nodes: u64, time_ms: u64, nps: u64, depth: u32) {
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("🔍 "),
        SetForegroundColor(Color::White),
        Print(format!("{} ", ai_name)),
        SetForegroundColor(Color::DarkGrey),
        Print("│ "),
        SetForegroundColor(Color::Yellow),
        Print(format!("depth {}", depth)),
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        SetForegroundColor(Color::Green),
        Print(format!("{} nodes", format_number(nodes))),
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        SetForegroundColor(Color::Magenta),
        Print(format!("{}ms", time_ms)),
        SetForegroundColor(Color::DarkGrey),
        Print(" │ "),
        SetForegroundColor(Color::Cyan),
        Print(format!("{} nps", format_number(nps))),
        ResetColor
    );
    println!();
}

/// Display AI search completion summary
pub fn display_search_complete(ai_name: &str, nodes: u64, time_ms: u64, nps: u64, depth: u32, best_move: Option<u8>, best_score: Option<f64>) {
    println!("┌─────────────────────────────────────┐");
    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print("✅ "),
        SetForegroundColor(Color::White),
        Print(format!("{} Search Complete", ai_name)),
        ResetColor
    );
    println!("       │");
    println!("├─────────────────────────────────────┤");

    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Yellow),
        Print("📊 Depth: "),
        SetForegroundColor(Color::White),
        Print(format!("{}", depth)),
        ResetColor
    );
    println!("                        │");

    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Green),
        Print("🔢 Nodes: "),
        SetForegroundColor(Color::White),
        Print(format!("{}", format_number(nodes))),
        ResetColor
    );
    let padding = 24 - format_number(nodes).len();
    for _ in 0..padding { print!(" "); }
    println!("│");

    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Magenta),
        Print("⏱️  Time: "),
        SetForegroundColor(Color::White),
        Print(format!("{}ms", time_ms)),
        ResetColor
    );
    let time_str = format!("{}ms", time_ms);
    let padding = 25 - time_str.len();
    for _ in 0..padding { print!(" "); }
    println!("│");

    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::Cyan),
        Print("⚡ Speed: "),
        SetForegroundColor(Color::White),
        Print(format!("{} nps", format_number(nps))),
        ResetColor
    );
    let nps_str = format!("{} nps", format_number(nps));
    let padding = 24 - nps_str.len();
    for _ in 0..padding { print!(" "); }
    println!("│");

    if let (Some(mv), Some(score)) = (best_move, best_score) {
        print!("│ ");
        let _ = execute!(
            io::stdout(),
            SetForegroundColor(Color::Blue),
            Print("🎯 Best: "),
            SetForegroundColor(Color::White),
            Print(format!("piece {} (score: {:.2})", mv, score)),
            ResetColor
        );
        let best_str = format!("piece {} (score: {:.2})", mv, score);
        let padding = 25 - best_str.len();
        for _ in 0..padding { print!(" "); }
        println!("│");
    }

    println!("└─────────────────────────────────────┘");
}

/// Display AI search header when search starts
pub fn display_search_header(ai_name: &str, player: FastPlayer, roll: u8, moves: &[u8]) {
    let (player_color, player_symbol) = match player {
        FastPlayer::One => (Color::Blue, "🔵"),
        FastPlayer::Two => (Color::Red, "🔴"),
    };

    println!("┌─────────────────────────────────────┐");
    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(Color::White),
        Print(format!("🤖 {} Analyzing", ai_name)),
        ResetColor
    );
    let padding = 21 - ai_name.len();
    for _ in 0..padding { print!(" "); }
    println!("│");

    print!("│ ");
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(player_color),
        Print(format!("{} {} ", player_symbol, player.name())),
        SetForegroundColor(Color::DarkGrey),
        Print("│ "),
        SetForegroundColor(Color::Yellow),
        Print(format!("roll {} ", roll)),
        SetForegroundColor(Color::DarkGrey),
        Print("│ "),
        SetForegroundColor(Color::Green),
        Print(format!("{} moves", moves.len())),
        ResetColor
    );
    let content_len = player.name().len() + 15; // Approximate content length
    let padding = 35 - content_len;
    for _ in 0..padding { print!(" "); }
    println!("│");
    println!("└─────────────────────────────────────┘");
}

/// Format large numbers with K/M suffixes for readability
fn format_number(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}
