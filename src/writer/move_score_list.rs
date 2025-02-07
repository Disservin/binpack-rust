use crate::{
    arithmetic::{signed_to_unsigned, used_bits_safe},
    chess::{
        attacks,
        bitboard::Bitboard,
        castling_rights::{CastleType, CastlingRights, CastlingTraits},
        color::Color,
        coords::{FlatSquareOffset, Rank, Square},
        piecetype::PieceType,
        position::Position,
        r#move::{Move, MoveType},
    },
    TrainingDataEntry,
};

const SCORE_VLE_BLOCK_SIZE: usize = 4;

#[derive(Debug)]
pub struct PackedMoveScoreList {
    pub num_plies: u16,
    pub movetext: Vec<u8>,
    bits_left: usize,
    last_score: i16,
}

impl PackedMoveScoreList {
    pub fn new() -> Self {
        Self {
            num_plies: 0,
            movetext: Vec::new(),
            bits_left: 0,
            last_score: 0,
        }
    }

    pub fn clear(&mut self, e: &TrainingDataEntry) {
        self.num_plies = 0;
        self.movetext.clear();
        self.bits_left = 0;
        self.last_score = -e.score;
    }

    fn add_bits_le8(&mut self, bits: u8, count: usize) {
        if count == 0 {
            return;
        }

        if self.bits_left == 0 {
            self.movetext.push(bits << (8 - count));
            self.bits_left = 8;
        } else if count <= self.bits_left {
            let last_idx = self.movetext.len() - 1;
            self.movetext[last_idx] |= bits << (self.bits_left - count);
        } else {
            let spill_count = count - self.bits_left;
            let last_idx = self.movetext.len() - 1;
            self.movetext[last_idx] |= bits >> spill_count;
            self.movetext.push(bits << (8 - spill_count));
            self.bits_left += 8;
        }

        self.bits_left -= count;
    }

    fn add_bits_vle16(&mut self, mut v: u16, block_size: usize) {
        let mask = (1 << block_size) - 1;
        loop {
            let block = ((v & mask) | (u16::from(v > mask) << block_size)) as u8;
            self.add_bits_le8(block, block_size + 1);
            v >>= block_size;
            if v == 0 {
                break;
            }
        }
    }

    pub fn add_move_score(&mut self, pos: &Position, mv: Move, score: i16) {
        let side_to_move = pos.side_to_move();
        let our_pieces = pos.pieces_bb(side_to_move);
        let their_pieces = pos.pieces_bb(!side_to_move);
        let occupied = our_pieces | their_pieces;

        let piece_id =
            (pos.pieces_bb(side_to_move) & Bitboard::from_before(mv.from().index())).count() as u8;
        let mut num_moves = 0u64;
        let mut move_id;
        let pt = pos.piece_at(mv.from()).piece_type();

        match pt {
            PieceType::Pawn => {
                let second_to_last_rank = Rank::last_pawn_rank(side_to_move);
                let start_rank = Rank::last_pawn_rank(!side_to_move);

                let forward = if side_to_move == Color::White {
                    FlatSquareOffset::new(0, 1)
                } else {
                    FlatSquareOffset::new(0, -1)
                };

                let ep_square = pos.ep_square();
                let mut attack_targets = their_pieces;
                if ep_square != Square::NONE {
                    attack_targets |= Bitboard::from_square(ep_square);
                }

                let mut destinations = attacks::pawn(side_to_move, mv.from()) & attack_targets;

                let sq_forward = mv.from() + forward;
                if !occupied.is_set(sq_forward.index()) {
                    destinations |= Bitboard::from_square(sq_forward);

                    if mv.from().rank() == start_rank && !occupied.sq_set(sq_forward + forward) {
                        destinations |= Bitboard::from_square(sq_forward + forward);
                    }
                }

                move_id = (destinations & Bitboard::from_before(mv.to().index())).count();
                num_moves = destinations.count() as u64;
                if mv.from().rank() == second_to_last_rank {
                    let promotion_index =
                        (mv.promoted_piece().piece_type() as usize) - (PieceType::Knight as usize);
                    move_id = move_id * 4 + promotion_index as u32;
                    num_moves *= 4;
                }
            }
            PieceType::King => {
                let our_castling_rights_mask = if side_to_move == Color::White {
                    CastlingRights::WHITE
                } else {
                    CastlingRights::BLACK
                };

                let castling_rights = pos.castling_rights();
                let attacks = attacks::king(mv.from()) & !our_pieces;
                let attacks_size = attacks.count();
                let num_castling_rights = (castling_rights & our_castling_rights_mask).count_ones();

                num_moves += attacks_size as u64;
                num_moves += num_castling_rights as u64;

                if mv.mtype() == MoveType::Castle {
                    let long_castling_rights =
                        CastlingTraits::castling_rights(side_to_move, CastleType::Long);

                    move_id = attacks_size - 1;

                    if castling_rights.contains(long_castling_rights) {
                        move_id += 1;
                    }

                    if mv.castle_type() == CastleType::Short {
                        move_id += 1;
                    }
                } else {
                    move_id = (attacks & Bitboard::from_before(mv.to().index())).count();
                }
            }
            _ => {
                let attacks = attacks::piece_attacks(pt, mv.from(), occupied) & !our_pieces;
                move_id = (attacks & Bitboard::from_before(mv.to().index())).count();
                num_moves = attacks.count() as u64;
            }
        }

        let num_pieces = our_pieces.count();
        self.add_bits_le8(piece_id, used_bits_safe(num_pieces as u64));
        self.add_bits_le8(move_id as u8, used_bits_safe(num_moves));

        let score_delta = signed_to_unsigned(score - self.last_score);
        self.add_bits_vle16(score_delta, SCORE_VLE_BLOCK_SIZE);
        self.last_score = -score;

        self.num_plies += 1;
    }
}
