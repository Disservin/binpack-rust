use crate::{
    chess::{
        attacks,
        bitboard::Bitboard,
        castling_rights::{CastleType, CastlingRights, CastlingTraits},
        coords::{FlatSquareOffset, Rank, Square},
        piece::Piece,
        piecetype::PieceType,
        r#move::Move,
    },
    common::{
        arithmetic::{nth_set_bit_index, unsigned_to_signed, used_bits_safe},
        entry::TrainingDataEntry,
    },
};

use super::bitreader::BitReader;

#[derive(Debug)]
pub struct PackedMoveScoreListReader {
    reader: BitReader,
    last_score: i16,
    num_plies: u16,
    num_read_plies: u16,
    entry: TrainingDataEntry,
}

impl PackedMoveScoreListReader {
    pub fn new(entry: TrainingDataEntry, movetext: *const u8, num_plies: u16) -> Self {
        Self {
            reader: BitReader::new(movetext),
            num_plies,
            entry,
            num_read_plies: 0,
            last_score: -entry.score,
        }
    }

    pub fn has_next(&self) -> bool {
        self.num_read_plies < self.num_plies
    }

    // Get the next TrainingDataEntry from the movetext
    pub fn next_entry(&mut self) -> TrainingDataEntry {
        self.entry.pos.do_move(self.entry.mv);
        let (mv, score) = self.next_move_score();
        self.entry.mv = mv;
        self.entry.score = score;
        self.entry.ply += 1;
        self.entry.result = -self.entry.result;
        self.entry
    }

    // Read a move and score from the movetext
    pub fn next_move_score(&mut self) -> (Move, i16) {
        // if !self.has_next() {
        //     return Ok(None);
        // }

        let pos = &self.entry.pos;

        let side_to_move = pos.side_to_move();
        let our_pieces = pos.pieces_bb(side_to_move);
        let their_pieces = pos.pieces_bb(!side_to_move);
        let occupied = our_pieces | their_pieces;

        let piece_id = self
            .reader
            .extract_bits_le8(used_bits_safe(our_pieces.count() as u64));

        // Extract the move
        let move_ = self.decode_move(piece_id, occupied);

        // Extract the score
        let score = self.decode_score();

        self.last_score = -score;

        self.num_read_plies += 1;

        (move_, score)
    }

    // EBNF: EncodedMove
    fn decode_score(&mut self) -> i16 {
        const SCORE_VLE_BLOCK_SIZE: usize = 4;
        let delta = unsigned_to_signed(self.reader.extract_vle16(SCORE_VLE_BLOCK_SIZE));

        self.last_score.wrapping_add(delta)
    }

    // EBNF: EncodedScore
    fn decode_move(&mut self, piece_id: u8, occupied: Bitboard) -> Move {
        let pos = &self.entry.pos;

        let side_to_move = pos.side_to_move();
        let our_pieces = pos.pieces_bb(side_to_move);
        let idx = nth_set_bit_index(our_pieces.bits(), piece_id as u64);

        let from = Square::new(idx);

        let piece_type = pos.piece_at(from).piece_type();

        match piece_type {
            PieceType::Pawn => {
                let promotion_rank = Rank::last_pawn_rank(side_to_move);
                let start_rank = Rank::last_pawn_rank(!side_to_move);
                let forward = FlatSquareOffset::forward(side_to_move);

                let ep_square = pos.ep_square();
                let their_pieces = pos.pieces_bb(!side_to_move);

                let mut attack_targets = their_pieces;

                if ep_square != Square::NONE {
                    attack_targets |= Bitboard::from_square(ep_square);
                }

                let mut destinations = attacks::pawn(side_to_move, from) & attack_targets;

                let sq_forward = from + forward;
                if !occupied.sq_set(sq_forward) {
                    destinations |= Bitboard::from_square(sq_forward);

                    // Add double push if on starting rank
                    if from.rank() == start_rank {
                        let sq_forward2 = sq_forward + forward;
                        if !occupied.sq_set(sq_forward2) {
                            destinations |= Bitboard::from_square(sq_forward2);
                        }
                    }
                }

                let destinations_count = destinations.count();

                if from.rank() == promotion_rank {
                    let move_id = self
                        .reader
                        .extract_bits_le8(used_bits_safe((destinations_count * 4) as u64));
                    let pt =
                        PieceType::from_ordinal(PieceType::Knight.ordinal() + (move_id % 4) as u8);
                    let promoted_piece = Piece::new(pt, side_to_move);
                    let to =
                        Square::new(nth_set_bit_index(destinations.bits(), move_id as u64 / 4));

                    Move::promotion(from, to, promoted_piece)
                } else {
                    let move_id = self
                        .reader
                        .extract_bits_le8(used_bits_safe(destinations_count as u64));

                    let idx = nth_set_bit_index(destinations.bits(), move_id as u64);

                    let to = Square::new(idx);

                    if to == ep_square {
                        Move::en_passant(from, to)
                    } else {
                        Move::normal(from, to)
                    }
                }
            }

            PieceType::King => {
                let our_castling_rights_mask = CastlingRights::castling_rights(side_to_move);

                let castling_rights = pos.castling_rights();

                let attacks = attacks::king(from) & !our_pieces;
                let attacks_size = attacks.count();

                let num_castlings =
                    (castling_rights & our_castling_rights_mask).count_ones() as usize;

                let offset = attacks_size as usize + num_castlings;
                let move_id = self.reader.extract_bits_le8(used_bits_safe(offset as u64)) as u32;

                if move_id >= attacks_size {
                    let idx = move_id - attacks_size;

                    let castle_type = if idx == 0
                        && castling_rights.contains(CastlingTraits::castling_rights(
                            side_to_move,
                            CastleType::Long,
                        )) {
                        CastleType::Long
                    } else {
                        CastleType::Short
                    };

                    Move::from_castle(castle_type, side_to_move)
                } else {
                    let to = Square::new(nth_set_bit_index(attacks.bits(), move_id as u64));
                    Move::normal(from, to)
                }
            }

            // All other pieces (Queen, Rook, Bishop, Knight)
            _ => {
                let attacks = attacks::piece_attacks(piece_type, from, occupied) & !our_pieces;
                let move_id = self
                    .reader
                    .extract_bits_le8(used_bits_safe(attacks.count() as u64));
                let idx = nth_set_bit_index(attacks.bits(), move_id as u64);
                let to = Square::new(idx);
                Move::normal(from, to)
            }
        }
    }

    pub fn num_read_bytes(&self) -> usize {
        self.reader.num_read_bytes()
    }
}
