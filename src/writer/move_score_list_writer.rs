use crate::arithmetic::used_bits_safe;
use crate::binpack_error::Result;
use crate::chess::attacks;
use crate::chess::bitboard::Bitboard;
use crate::chess::castling_rights::{CastleType, CastlingRights, CastlingTraits};
use crate::chess::color::Color;
use crate::chess::coords::Rank;
use crate::chess::coords::{FlatSquareOffset, Square};
use crate::chess::piecetype::PieceType;
use crate::chess::position::Position;
use crate::chess::r#move::{Move, MoveType};
use crate::writer::bitwriter::BitWriter;

pub struct PackedMoveScoreList {
    pub num_plies: u16,
    writer: BitWriter,
    last_score: i16,
}

impl Default for PackedMoveScoreList {
    fn default() -> Self {
        Self::new()
    }
}

impl PackedMoveScoreList {
    pub fn new() -> Self {
        Self {
            num_plies: 0,
            writer: BitWriter::new(),
            last_score: 0,
        }
    }

    pub fn clear(&mut self, initial_score: i16) {
        self.num_plies = 0;
        self.writer.movetext.clear();
        self.last_score = -initial_score;
    }

    pub fn add_move_score(&mut self, pos: &Position, move_: Move, score: i16) -> Result<()> {
        const SCORE_VLE_BLOCK_SIZE: usize = 4;

        let side_to_move = pos.side_to_move();
        let our_pieces = pos.pieces_bb(side_to_move);
        let their_pieces = pos.pieces_bb(!side_to_move);
        let occupied = our_pieces | their_pieces;

        // Calculate piece_id (number of pieces before the moving piece)
        let piece_id = (pos.pieces_bb(side_to_move) & before(move_.from())).count();
        let num_pieces = our_pieces.count();

        // Get legal moves and calculate move_id based on piece type
        let (move_id, num_moves) = self.calculate_move_encoding(pos, move_, occupied)?;

        // Encode the move
        self.writer
            .add_bits_le8(piece_id as u8, used_bits_safe(num_pieces as u64));
        self.writer
            .add_bits_le8(move_id as u8, used_bits_safe(num_moves as u64));

        // Encode the score
        let score_delta = signed_to_unsigned(score - self.last_score);
        self.writer
            .add_bits_vle16(score_delta, SCORE_VLE_BLOCK_SIZE);
        self.last_score = -score;

        self.num_plies += 1;
        Ok(())
    }

    fn calculate_move_encoding(
        &self,
        pos: &Position,
        move_: Move,
        occupied: Bitboard,
    ) -> Result<(u32, u32)> {
        let side_to_move = pos.side_to_move();
        let our_pieces = pos.pieces_bb(side_to_move);
        let their_pieces = pos.pieces_bb(!side_to_move);

        let piece_type = pos.piece_at(move_.from()).piece_type();

        match piece_type {
            PieceType::Pawn => {
                let second_to_last_rank = if side_to_move == Color::White {
                    Rank::SEVENTH
                } else {
                    Rank::SECOND
                };
                let start_rank = if side_to_move == Color::White {
                    Rank::SECOND
                } else {
                    Rank::SEVENTH
                };
                let forward = if side_to_move == Color::White {
                    FlatSquareOffset::new(0, 1)
                } else {
                    FlatSquareOffset::new(0, -1)
                };

                let ep_square = pos.ep_square();

                // Calculate attack targets (including en passant square)
                let mut attack_targets = their_pieces;
                if ep_square != Square::NONE {
                    attack_targets |= Bitboard::from_square(ep_square);
                }

                // Calculate possible destinations
                let mut destinations = attacks::pawn(side_to_move, move_.from()) & attack_targets;

                // Add forward moves
                let sq_forward = move_.from() + forward;
                if !occupied.sq_set(sq_forward) {
                    destinations |= Bitboard::from_square(sq_forward);

                    if move_.from().rank() == start_rank {
                        let sq_forward2 = sq_forward + forward;
                        if !occupied.sq_set(sq_forward2) {
                            destinations |= Bitboard::from_square(sq_forward2);
                        }
                    }
                }

                let mut move_id = (destinations & before(move_.to())).count();
                let mut num_moves = destinations.count();

                if move_.from().rank() == second_to_last_rank {
                    let promotion_index =
                        move_.promoted_piece().piece_type().ordinal() - PieceType::Knight.ordinal();
                    move_id = move_id * 4 + promotion_index as u32;
                    num_moves *= 4;
                }

                Ok((move_id, num_moves))
            }

            PieceType::King => {
                let our_castling_rights_mask = if side_to_move == Color::White {
                    CastlingRights::WHITE
                } else {
                    CastlingRights::BLACK
                };

                let castling_rights = pos.castling_rights();

                let attacks = attacks::king(move_.from()) & !our_pieces;
                let attacks_size = attacks.count();
                let num_castling_rights = (castling_rights & our_castling_rights_mask).count_ones();

                let num_moves = attacks_size + num_castling_rights;
                let mut move_id;

                if move_.mtype() == MoveType::Castle {
                    let long_castling_rights =
                        CastlingTraits::castling_rights(side_to_move, CastleType::Long);

                    move_id = attacks_size - 1;

                    if castling_rights.contains(long_castling_rights) {
                        move_id += 1;
                    }

                    if move_.castle_type() == CastleType::Short {
                        move_id += 1;
                    }
                } else {
                    move_id = (attacks & before(move_.to())).count();
                }

                Ok((move_id, num_moves))
            }

            _ => {
                let attacks =
                    attacks::piece_attacks(piece_type, move_.from(), occupied) & !our_pieces;
                let move_id = (attacks & before(move_.to())).count();
                let num_moves = attacks.count();

                Ok((move_id, num_moves))
            }
        }
    }
}

// Helper functions

fn signed_to_unsigned(a: i16) -> u16 {
    let mut r = a as u16;
    if r & 0x8000 != 0 {
        r ^= 0x7FFF;
    }
    r.rotate_left(1)
}

fn before(_sq: Square) -> Bitboard {
    todo!()
}
