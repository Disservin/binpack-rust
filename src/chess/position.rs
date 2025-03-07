use crate::chess::{
    attacks,
    bitboard::Bitboard,
    castling_rights::{CastleType, CastlingRights},
    color::Color,
    coords::Square,
    piece::Piece,
    piecetype::PieceType,
    r#move::{Move, MoveType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Bitboards for each piece type (PNBRQK)
    bb: [u64; 6],
    /// Bitboards for each color (White, Black)
    bb_color: [u64; 2],
    /// Piece list
    pieces: [Piece; 64],
    /// Side to move
    stm: Color,
    /// Castling rights
    castling_rights: CastlingRights,
    /// Halfmove clock for 50-move rule
    halfm: u8,
    /// Fullmove number
    fullm: u16,
    /// En passant target square
    enpassant: Square,
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl Position {
    pub fn new() -> Self {
        Self {
            bb: [0; 6],
            bb_color: [0; 2],
            pieces: [Piece::none(); 64],
            stm: Color::White,
            castling_rights: CastlingRights::NONE,
            halfm: 0,
            fullm: 1,
            enpassant: Square::NONE,
        }
    }

    /// Returns the current side to move's color
    pub fn side_to_move(&self) -> Color {
        self.stm
    }

    /// Returns a bitboard with all occupied squares
    pub fn occupied(&self) -> Bitboard {
        Bitboard::new(self.bb_color[0] | self.bb_color[1])
    }

    /// Returns the bitboard of all pieces of a given color
    pub fn pieces_bb(&self, color: Color) -> Bitboard {
        let bb = Bitboard::new(self.bb_color[color as usize]);

        debug_assert!(bb.count() > 0);

        bb
    }

    /// Returns the bitboard of all pieces of a given piece type
    pub fn pieces_bb_type(&self, pt: PieceType) -> Bitboard {
        debug_assert!(pt != PieceType::None);
        Bitboard::new(self.bb[pt.ordinal() as usize])
    }

    /// Returns the bitboard of all pieces of a given color and piece type
    pub fn pieces_bb_color(&self, color: Color, pt: PieceType) -> Bitboard {
        Bitboard::new(self.bb_color[color as usize] & self.bb[pt.ordinal() as usize])
    }

    /// Returns the piece at a given square, Piece::NONE if the square is empty
    pub fn piece_at(&self, square: Square) -> Piece {
        debug_assert!(square != Square::NONE);

        self.pieces[square.index() as usize]
    }

    /// Returns the castling rights
    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Returns the en passant square, or Square::NONE if there is none
    pub fn ep_square(&self) -> Square {
        self.enpassant
    }

    /// Make a legal move on the board
    pub fn do_move(&mut self, mv: Move) {
        debug_assert!(self.bb[PieceType::King.ordinal() as usize].count_ones() == 2);

        let from = mv.from();
        let to = mv.to();
        let piece = self.piece_at(from);
        let captured = self.piece_at(to);
        let genuine_capture = captured != Piece::none() && mv.mtype() != MoveType::Castle;

        debug_assert!(from != Square::NONE);
        debug_assert!(to != Square::NONE);
        debug_assert!(piece != Piece::none());

        // clear piece from start
        self.remove_piece(self.stm, piece, from);

        // capture piece
        if genuine_capture {
            self.remove_piece(!self.stm, captured, to);
        }

        if mv.mtype() == MoveType::Promotion {
            let promotion = mv.promoted_piece();
            self.place_piece(self.stm, promotion, to);
        } else if mv.mtype() == MoveType::EnPassant {
            debug_assert!(piece.piece_type() == PieceType::Pawn,);

            let captured_sq = Square::new(to.index() ^ 8);
            self.remove_piece(!self.stm, self.piece_at(captured_sq), captured_sq);
            self.place_piece(self.stm, piece, to);
        } else if mv.mtype() == MoveType::Normal {
            self.place_piece(self.stm, piece, to);
        } else if mv.mtype() == MoveType::Castle {
            if mv.castle_type() == CastleType::Short {
                let rook_to = if self.stm == Color::White {
                    Square::F1
                } else {
                    Square::F8
                };

                let king_to = if self.stm == Color::White {
                    Square::G1
                } else {
                    Square::G8
                };

                let rook = self.piece_at(to);

                self.remove_piece(self.stm, rook, to);
                self.place_piece(self.stm, rook, rook_to);
                self.place_piece(self.stm, piece, king_to);
            } else {
                let rook_to = if self.stm == Color::White {
                    Square::D1
                } else {
                    Square::D8
                };

                let king_to = if self.stm == Color::White {
                    Square::C1
                } else {
                    Square::C8
                };

                let rook = self.piece_at(to);

                self.remove_piece(self.stm, rook, to);
                self.place_piece(self.stm, rook, rook_to);
                self.place_piece(self.stm, piece, king_to);
            }
        }

        // update state

        // Update halfmove clock
        if genuine_capture || piece.piece_type() == PieceType::Pawn {
            self.halfm = 0;
        } else {
            self.halfm += 1;
        }

        // Update fullmove number
        if self.stm == Color::Black {
            self.fullm += 1;
        }

        self.update_castling_rights(from, to);

        self.enpassant = Square::NONE;

        // Update en passant square
        if piece.piece_type() == PieceType::Pawn
            && (to.index() as i32 - from.index() as i32).abs() == 16
        {
            let ep = Square::new(to.index() ^ 8);

            // check if enemy pawn can legally capture the pawn
            // if so set the ep square

            let ep_mask = attacks::pawn(self.stm, ep);
            let enemy_mask = self.pieces_bb_color(!self.stm, PieceType::Pawn);

            // enemy pawn can pseudo capture the pawn
            if (ep_mask & enemy_mask).bits() > 0 {
                // check if enemy pawn can legally capture the pawn
                // play the move

                // loop over enemy mask
                let mut enemy_mask = ep_mask & enemy_mask;

                while enemy_mask != Bitboard::new(0) {
                    let enemy_sq = Square::new(enemy_mask.bits().trailing_zeros());
                    enemy_mask = enemy_mask & Bitboard::new(enemy_mask.bits() - 1);

                    // move the enemy pawn
                    let enemy_pawn = self.piece_at(enemy_sq);
                    self.remove_piece(!self.stm, enemy_pawn, enemy_sq);
                    self.place_piece(!self.stm, enemy_pawn, ep);

                    // remove our pawn
                    self.remove_piece(self.stm, piece, to);

                    // check if the side which made the move is in check
                    let is_checked = self.is_checked(!self.stm);

                    // undo the move

                    // move the enemy pawn
                    self.place_piece(!self.stm, enemy_pawn, enemy_sq);
                    self.remove_piece(!self.stm, enemy_pawn, ep);

                    // place our pawn
                    self.place_piece(self.stm, piece, to);

                    if !is_checked {
                        self.enpassant = ep;
                        break;
                    }
                }
            }
        }

        // Switch side to move
        self.stm = !self.stm;

        debug_assert!(self.bb[PieceType::King.ordinal() as usize].count_ones() == 2);
    }

    pub fn set_castling_rights(&mut self, rights: CastlingRights) {
        self.castling_rights = rights;
    }

    /// No validation is done, use with caution
    pub fn set_ep_square_unchecked(&mut self, sq: Square) {
        self.enpassant = sq;
    }

    pub fn add_castling_rights(&mut self, rights: CastlingRights) {
        self.castling_rights |= rights;
    }

    pub fn set_side_to_move(&mut self, side: Color) {
        self.stm = side;
    }

    pub fn set_ply(&mut self, ply: u16) {
        self.fullm = (ply / 2) + 1;
    }

    pub fn ply(&self) -> u16 {
        ((self.fullm - 1) * 2) + (self.stm as u16)
    }

    pub fn set_rule50_counter(&mut self, counter: u16) {
        self.halfm = counter as u8;
    }

    pub fn rule50_counter(&self) -> u16 {
        self.halfm as u16
    }

    /// Places a piece on the board
    #[inline(always)]
    pub fn place(&mut self, pc: Piece, sq: Square) {
        debug_assert!(pc != Piece::none());
        debug_assert!(sq != Square::NONE);

        self.place_piece(pc.color(), pc, sq);
    }

    /// Places a piece on the board
    #[inline(always)]
    fn place_piece(&mut self, side: Color, pc: Piece, sq: Square) {
        debug_assert!(pc != Piece::none());
        debug_assert!(sq != Square::NONE);
        debug_assert!(side == pc.color());

        let mask = 1u64 << (sq.index());
        self.bb_color[side as usize] |= mask;
        self.bb[pc.piece_type().ordinal() as usize] |= mask;
        self.pieces[sq.index() as usize] = pc;
    }

    /// Removes a piece from the board
    #[inline(always)]
    fn remove_piece(&mut self, side: Color, pc: Piece, sq: Square) {
        debug_assert!(pc != Piece::none());
        debug_assert!(sq != Square::NONE);

        let mask = 1u64 << (sq.index());
        self.bb_color[side as usize] ^= mask;
        self.bb[pc.piece_type().ordinal() as usize] ^= mask;
        self.pieces[sq.index() as usize] = Piece::none();
    }

    /// Returns the FEN representation of the position
    pub fn fen(&self) -> String {
        let mut fen = String::new();

        // pieces
        for rank in (0..8).rev() {
            let mut empty_squares = 0;

            for file in 0..8 {
                let square = Square::new((rank * 8 + file) as u32);
                let piece = self.piece_at(square);

                if piece == Piece::none() {
                    empty_squares += 1;
                } else {
                    if empty_squares > 0 {
                        fen.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }

                    let mut c = match piece.piece_type() {
                        PieceType::Pawn => 'p',
                        PieceType::Knight => 'n',
                        PieceType::Bishop => 'b',
                        PieceType::Rook => 'r',
                        PieceType::Queen => 'q',
                        PieceType::King => 'k',
                        _ => panic!("Invalid piece type"),
                    };

                    if piece.color() == Color::White {
                        c = c.to_ascii_uppercase();
                    }
                    fen.push(c);
                }
            }
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // color
        fen.push(' ');
        fen.push(if self.stm == Color::White { 'w' } else { 'b' });

        // castling
        fen.push(' ');
        let castling = self.castling_rights();
        if castling == CastlingRights::NONE {
            fen.push('-');
        } else {
            if castling.contains(CastlingRights::WHITE_KING_SIDE) {
                fen.push('K');
            }
            if castling.contains(CastlingRights::WHITE_QUEEN_SIDE) {
                fen.push('Q');
            }
            if castling.contains(CastlingRights::BLACK_KING_SIDE) {
                fen.push('k');
            }
            if castling.contains(CastlingRights::BLACK_QUEEN_SIDE) {
                fen.push('q');
            }
        }

        // ep square
        fen.push(' ');
        if self.enpassant == Square::NONE {
            fen.push('-');
        } else {
            // let file = (self.enpassant.to_u32() % 8) as u8;
            // let rank = (self.enpassant.to_u32() / 8) as u8;
            // fen.push((b'a' + file) as char);
            // fen.push((b'1' + rank) as char);
            fen.push_str(&self.enpassant.to_string());
        }

        // halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfm.to_string());

        // fullmove number
        fen.push(' ');
        fen.push_str(&self.fullm.to_string());

        fen
    }

    /// Create a position from a FEN string
    pub fn from_fen(fen: &str) -> Self {
        let mut pos = Self::new();
        pos.parse_fen(fen);
        pos
    }

    /// Parse a FEN string and set the position
    fn parse_fen(&mut self, fen: &str) {
        let mut parts = fen.split_whitespace();

        let mut rank = 7;
        let mut file = 0;

        for c in parts.next().unwrap().chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
            } else if c.is_ascii_digit() {
                file += c.to_digit(10).unwrap() as usize;
            } else {
                let color = if c.is_uppercase() {
                    Color::White
                } else {
                    Color::Black
                };

                let piece = match c.to_ascii_lowercase() {
                    'p' => Piece::new(PieceType::Pawn, color),
                    'n' => Piece::new(PieceType::Knight, color),
                    'b' => Piece::new(PieceType::Bishop, color),
                    'r' => Piece::new(PieceType::Rook, color),
                    'q' => Piece::new(PieceType::Queen, color),
                    'k' => Piece::new(PieceType::King, color),
                    _ => panic!("Invalid piece type"),
                };

                self.place(piece, Square::new(rank * 8 + file as u32));
                file += 1;
            }
        }

        self.stm = if parts.next().unwrap() == "w" {
            Color::White
        } else {
            Color::Black
        };

        self.castling_rights = CastlingRights::NONE;
        for c in parts.next().unwrap().chars() {
            match c {
                'K' => self.castling_rights |= CastlingRights::WHITE_KING_SIDE,
                'Q' => self.castling_rights |= CastlingRights::WHITE_QUEEN_SIDE,
                'k' => self.castling_rights |= CastlingRights::BLACK_KING_SIDE,
                'q' => self.castling_rights |= CastlingRights::BLACK_QUEEN_SIDE,
                _ => {}
            }
        }

        let ep = parts.next().unwrap();
        if ep != "-" {
            self.enpassant = Square::from_string(ep).unwrap();
        }

        self.halfm = parts.next().unwrap().parse().unwrap();
        self.fullm = parts.next().unwrap().parse().unwrap();
    }

    /// Check if a square is attacked by the given color
    pub fn is_attacked(&self, sq: Square, c: Color) -> bool {
        let pieces = |piece_type| self.pieces_bb_color(c, piece_type);
        let occupied = self.occupied();

        // fast stuff first

        (attacks::pawn(!c, sq) & pieces(PieceType::Pawn)
            | attacks::knight(sq) & pieces(PieceType::Knight)
            | attacks::king(sq) & pieces(PieceType::King)
            | attacks::bishop(sq, occupied)
                & (pieces(PieceType::Bishop) | pieces(PieceType::Queen))
            | attacks::rook(sq, occupied) & (pieces(PieceType::Rook) | pieces(PieceType::Queen)))
        .bits()
            > 0
    }

    /// Returns the square of the king of the given color
    pub fn king_sq(&self, c: Color) -> Square {
        self.pieces_bb_color(c, PieceType::King).lsb()
    }

    /// Returns true if the given color is in check
    pub fn is_checked(&self, c: Color) -> bool {
        self.is_attacked(self.king_sq(c), !c)
    }

    fn update_castling_rights(&mut self, from: Square, to: Square) {
        // Remove castling rights if king or rook moves
        if from == Square::E1 || to == Square::E1 {
            self.castling_rights &= !CastlingRights::WHITE;
        }
        if from == Square::E8 || to == Square::E8 {
            self.castling_rights &= !CastlingRights::BLACK;
        }
        if from == Square::A1 || to == Square::A1 {
            self.castling_rights &= !CastlingRights::WHITE_QUEEN_SIDE;
        }
        if from == Square::H1 || to == Square::H1 {
            self.castling_rights &= !CastlingRights::WHITE_KING_SIDE;
        }
        if from == Square::A8 || to == Square::A8 {
            self.castling_rights &= !CastlingRights::BLACK_QUEEN_SIDE;
        }
        if from == Square::H8 || to == Square::H8 {
            self.castling_rights &= !CastlingRights::BLACK_KING_SIDE;
        }
    }

    pub fn after_move(&self, mv: Move) -> Self {
        let mut pos = *self;
        pos.do_move(mv);
        pos
    }
}
