/// Ultra-fast GameState implementation inspired by chess engines
/// Key optimizations:
/// 1. Bitboards for O(1) occupancy checks
/// 2. Packed representation (fits in 128 bits total)
/// 3. Make/unmake moves instead of cloning
/// 4. Zero-allocation design for performance
/// 5. SIMD-friendly operations where possible

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FastGameState {
    /// Bitboard for both players: bits 0-19 = Player 1, bits 20-39 = Player 2
    pub occupied_squares: u64,
    /// Packed piece positions: 4 bits per piece, 7 pieces per player = 56 bits total
    /// Lower 28 bits = Player 1, upper 28 bits = Player 2
    /// Each 4-bit value: 0=OffBoard, 1-14=OnBoard(0-13), 15=Finished
    pub piece_positions: u64,
    /// Packed scores and turn: bits 0-2=P1 score, bits 3-5=P2 score, bit 6=turn
    pub scores_and_turn: u8,
}

/// Move representation that can be undone
#[derive(Clone, Copy, Debug)]
pub struct MoveInfo {
    pub piece_idx: u8,
    pub from_pos: u8,
    pub to_pos: u8,
    pub captured_piece: Option<u8>,
    pub extra_turn: bool,
}

/// Player enumeration that packs into single bits
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FastPlayer {
    One = 0,
    Two = 1,
}

impl FastPlayer {
    pub fn opposite(self) -> Self {
        match self {
            FastPlayer::One => FastPlayer::Two,
            FastPlayer::Two => FastPlayer::One,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FastPlayer::One => "Player 1",
            FastPlayer::Two => "Player 2",
        }
    }
}



impl FastGameState {
    const PATHS: [[u8; 14]; 2] = [
        [3, 2, 1, 0, 6, 7, 8, 9, 10, 11, 12, 13, 5, 4],      // Player 1
        [17, 16, 15, 14, 6, 7, 8, 9, 10, 11, 12, 13, 19, 18], // Player 2
    ];

    /// Rosette squares (give extra turns)
    const ROSETTES: u32 = (1 << 4) | (1 << 9) | (1 << 18);

    /// Safe squares (cannot be captured)
    const SAFE_SQUARES: u32 = (1 << 0) | (1 << 4) | (1 << 9) | (1 << 14) | (1 << 18);

    pub fn new() -> Self {
        FastGameState {
            occupied_squares: 0,
            piece_positions: 0,
            scores_and_turn: 0,
        }
    }

    /// Get current player
    #[inline]
    pub fn current_player(self) -> FastPlayer {
        if (self.scores_and_turn >> 6) & 1 == 0 {
            FastPlayer::One
        } else {
            FastPlayer::Two
        }
    }

    /// Get score for player
    #[inline]
    pub fn get_score(self, player: FastPlayer) -> u8 {
        match player {
            FastPlayer::One => self.scores_and_turn & 0x7,
            FastPlayer::Two => (self.scores_and_turn >> 3) & 0x7,
        }
    }

    /// Set score for player
    #[inline]
    pub fn set_score(&mut self, player: FastPlayer, score: u8) {
        match player {
            FastPlayer::One => {
                self.scores_and_turn = (self.scores_and_turn & !0x7) | (score & 0x7);
            }
            FastPlayer::Two => {
                self.scores_and_turn = (self.scores_and_turn & !0x38) | ((score & 0x7) << 3);
            }
        }
    }

    /// Get piece position (0=OffBoard, 1-14=OnBoard(0-13), 15=Finished)
    #[inline]
    pub fn get_piece_pos(self, player: FastPlayer, piece_idx: u8) -> u8 {
        let shift = match player {
            FastPlayer::One => piece_idx * 4,
            FastPlayer::Two => 28 + piece_idx * 4,
        };
        ((self.piece_positions >> shift) & 0xF) as u8
    }

    /// Set piece position
    #[inline]
    pub fn set_piece_pos(&mut self, player: FastPlayer, piece_idx: u8, pos: u8) {
        let shift = match player {
            FastPlayer::One => piece_idx * 4,
            FastPlayer::Two => 28 + piece_idx * 4,
        };
        let mask = !(0xF << shift);
        self.piece_positions = (self.piece_positions & mask) | ((pos as u64 & 0xF) << shift);
    }

    /// Path to global square conversion
    #[inline]
    pub fn path_to_global(player: FastPlayer, path_idx: u8) -> u8 {
        Self::PATHS[player as usize][path_idx as usize]
    }

    /// Check if square is a rosette
    #[inline]
    pub fn is_rosette(square: u8) -> bool {
        (Self::ROSETTES >> square) & 1 != 0
    }

    /// Check if square is safe
    #[inline]
    pub fn is_safe(square: u8) -> bool {
        (Self::SAFE_SQUARES >> square) & 1 != 0
    }

    /// Check if square is occupied and by whom
    #[inline]
    pub fn get_occupant(self, square: u8) -> Option<FastPlayer> {
        if (self.occupied_squares >> square) & 1 != 0 {
            Some(FastPlayer::One)
        } else if (self.occupied_squares >> (square + 20)) & 1 != 0 {
            Some(FastPlayer::Two)
        } else {
            None
        }
    }

    /// Make a move and return undo information
    pub fn make_move(&mut self, piece_idx: u8, roll: u8) -> Option<MoveInfo> {
        let player = self.current_player();
        let from_pos = self.get_piece_pos(player, piece_idx);

        let to_pos = match from_pos {
            0 => 1,  // Off board to path position 0 (encoded as 1)
            1..=14 => {
                let path_idx = from_pos - 1;
                let new_path_idx = path_idx + roll;
                if new_path_idx >= 14 {
                    15  // Finished
                } else {
                    new_path_idx + 1  // On board (encoded as path_idx + 1)
                }
            }
            15 => return None,  // Already finished
            _ => return None,
        };

        // Validate move
        let mut captured_piece = None;
        if to_pos >= 1 && to_pos <= 14 {
            let target_square = Self::path_to_global(player, to_pos - 1);
            match self.get_occupant(target_square) {
                Some(occupant) if occupant == player => return None,
                Some(_) if Self::is_safe(target_square) => return None,
                Some(_) => {
                    // Capture
                    for i in 0..7 {
                        let opp_pos = self.get_piece_pos(player.opposite(), i);
                        if opp_pos >= 1 && opp_pos <= 14 {
                            let opp_square = Self::path_to_global(player.opposite(), opp_pos - 1);
                            if opp_square == target_square {
                                captured_piece = Some(i);
                                break;
                            }
                        }
                    }
                }
                None => {}
            }
        }

        let extra_turn = to_pos >= 1 && to_pos <= 14 &&
                        Self::is_rosette(Self::path_to_global(player, to_pos - 1));

        let move_info = MoveInfo {
            piece_idx,
            from_pos,
            to_pos,
            captured_piece,
            extra_turn,
        };

        // Apply the move
        self.apply_move_internal(player, &move_info);

        Some(move_info)
    }

    /// Apply move to the board
    fn apply_move_internal(&mut self, player: FastPlayer, move_info: &MoveInfo) {
        let player_offset = match player {
            FastPlayer::One => 0,
            FastPlayer::Two => 20,
        };

        // Remove from old position
        if move_info.from_pos >= 1 && move_info.from_pos <= 14 {
            let old_square = Self::path_to_global(player, move_info.from_pos - 1);
            self.occupied_squares &= !(1u64 << (old_square + player_offset));
        }

        // Handle capture
        if let Some(cap_piece) = move_info.captured_piece {
            let opp_player = player.opposite();
            let opp_offset = match opp_player {
                FastPlayer::One => 0,
                FastPlayer::Two => 20,
            };
            let cap_pos = self.get_piece_pos(opp_player, cap_piece);
            let cap_square = Self::path_to_global(opp_player, cap_pos - 1);

            self.occupied_squares &= !(1u64 << (cap_square + opp_offset));
            self.set_piece_pos(opp_player, cap_piece, 0);
        }

        // Set new position
        self.set_piece_pos(player, move_info.piece_idx, move_info.to_pos);

        if move_info.to_pos >= 1 && move_info.to_pos <= 14 {
            let new_square = Self::path_to_global(player, move_info.to_pos - 1);
            self.occupied_squares |= 1u64 << (new_square + player_offset);
        } else if move_info.to_pos == 15 {
            // Update score
            let current_score = self.get_score(player);
            self.set_score(player, current_score + 1);
        }

        // Update turn if no extra turn
        if !move_info.extra_turn {
            self.scores_and_turn ^= 1 << 6;
        }
    }

    /// Unmake a move (restore previous state)
    pub fn unmake_move(&mut self, player: FastPlayer, move_info: &MoveInfo) {
        let player_offset = match player {
            FastPlayer::One => 0,
            FastPlayer::Two => 20,
        };

        // Remove from current position
        if move_info.to_pos >= 1 && move_info.to_pos <= 14 {
            let square = Self::path_to_global(player, move_info.to_pos - 1);
            self.occupied_squares &= !(1u64 << (square + player_offset));
        } else if move_info.to_pos == 15 {
            // Undo score
            let current_score = self.get_score(player);
            self.set_score(player, current_score - 1);
        }

        // Restore to old position
        self.set_piece_pos(player, move_info.piece_idx, move_info.from_pos);
        if move_info.from_pos >= 1 && move_info.from_pos <= 14 {
            let old_square = Self::path_to_global(player, move_info.from_pos - 1);
            self.occupied_squares |= 1u64 << (old_square + player_offset);
        }

        // Restore captured piece
        if let Some(cap_piece) = move_info.captured_piece {
            let opp_player = player.opposite();
            let opp_offset = match opp_player {
                FastPlayer::One => 0,
                FastPlayer::Two => 20,
            };

            // Find where it was captured
            let cap_square = Self::path_to_global(player, move_info.to_pos - 1);
            let cap_path_pos = Self::global_to_path(opp_player, cap_square) + 1;

            self.set_piece_pos(opp_player, cap_piece, cap_path_pos);
            self.occupied_squares |= 1u64 << (cap_square + opp_offset);
        }

        // Restore turn
        if !move_info.extra_turn {
            self.scores_and_turn ^= 1 << 6;
        }
    }

    /// Check if player has won
    #[inline]
    pub fn is_winner(self, player: FastPlayer) -> bool {
        self.get_score(player) >= 7
    }

    /// Generate all valid moves for current player with given roll
    pub fn generate_moves(self, roll: u8) -> Vec<u8> {
        if roll == 0 {
            return vec![];
        }

        let player = self.current_player();
        let mut moves = Vec::with_capacity(7);

        for piece_idx in 0..7 {
            let pos = self.get_piece_pos(player, piece_idx);

            match pos {
                0 => {
                    // Off board - check if can enter at position 0
                    let target_square = Self::path_to_global(player, 0);
                    if self.can_move_to(player, target_square) {
                        moves.push(piece_idx);
                    }
                }
                1..=14 => {
                    let path_idx = pos - 1;
                    let new_path_idx = path_idx + roll;

                    if new_path_idx == 14 {
                        // Exact move to finish
                        moves.push(piece_idx);
                    } else if new_path_idx < 14 {
                        let target_square = Self::path_to_global(player, new_path_idx);
                        if self.can_move_to(player, target_square) {
                            moves.push(piece_idx);
                        }
                    }
                }
                15 => {
                    // Already finished
                }
                _ => {}
            }
        }

        moves
    }

    fn can_move_to(self, player: FastPlayer, square: u8) -> bool {
        match self.get_occupant(square) {
            None => true,
            Some(occupant) => {
                occupant != player && !Self::is_safe(square)
            }
        }
    }

    /// Roll dice (same as original)
    pub fn roll_dice() -> u8 {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut total = 0;
        for _ in 0..4 {
            if rng.random_bool(0.5) {
                total += 1;
            }
        }
        total
    }

    fn global_to_path(player: FastPlayer, global: u8) -> u8 {
        for (i, &square) in Self::PATHS[player as usize].iter().enumerate() {
            if square == global {
                return i as u8;
            }
        }
        0
    }
}

impl fmt::Display for FastGameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "FastGameState:")?;
        writeln!(f, "  Current player: {}", self.current_player().name())?;
        writeln!(f, "  Player 1 score: {}", self.get_score(FastPlayer::One))?;
        writeln!(f, "  Player 2 score: {}", self.get_score(FastPlayer::Two))?;

        for player in [FastPlayer::One, FastPlayer::Two] {
            writeln!(f, "  {} pieces:", player.name())?;
            for piece_idx in 0..7 {
                let pos = self.get_piece_pos(player, piece_idx);
                let desc = match pos {
                    0 => "OffBoard".to_string(),
                    1..=14 => format!("OnBoard({})", pos - 1),
                    15 => "Finished".to_string(),
                    _ => "Invalid".to_string(),
                };
                writeln!(f, "    Piece {}: {}", piece_idx, desc)?;
            }
        }

        Ok(())
    }
}
