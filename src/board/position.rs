use crate::board::bitboard as bb;
use crate::board::bitboard::alligned;
use crate::board::bitboard::more_than_one;
use crate::board::bitboard::pawn_attacks_bb;
use crate::board::bitboard::RANK1BB;
use crate::board::bitboard::RANK8BB;
use crate::board::zobrist;
use crate::types::*;
use std::fmt;
use std::sync::OnceLock;
use std::vec::Vec;

use super::bitboard::attacks_bb;
use super::bitboard::pseudo_attacks_bb;
use super::bitboard::BETWEEN_BB;

const PIECE_TYPE_NB: usize = PieceType::PieceTypeNb as usize;
const PIECE_TO_CHAR: &str = " PNBRQK  pnbrqk";
const MAX_PLY: usize = 246; // Maximum search depth

pub static CUCKOO: OnceLock<[Key; 8192]> = OnceLock::new();
pub static CUCKOO_MOVE: OnceLock<[Key; 8192]> = OnceLock::new();

#[macro_export]
macro_rules! pieces_of_types {
    ($pos: expr, $pt: expr) => {
        $pos.pieces_by_piecetype($pt)
    };

    ($pos: expr, $pt: expr, $($rest_pts: expr),+) => {
        $pos.pieces_by_piecetype($pt) | pieces_of_types!($pos, $($rest_pts),+)
    }
}

#[macro_export]
macro_rules! pieces_by_color_and_pt {
    ($pos: expr, $color: expr, $pt: expr) => {
        pieces_of_types!($pos, $pt)
    };

    ($pos: expr, $color: expr, $pt: expr, $($rest_pt: expr),+) => {
        $pos.pieces_by_color($color) & (pieces_by_color_and_pt!($pos, $color, $pt) | pieces_by_color_and_pt!($pos, $color, $($rest_pt),+))
    };
}

#[macro_export]
macro_rules! all_pieces {
    ($pos: expr) => {
        pieces_of_types!($pos, PieceType::AllPieces)
    };
}

const pieces: [Piece; 12] = [
    Piece::WPawn,
    Piece::WKnight,
    Piece::WBishop,
    Piece::WRook,
    Piece::WQueen,
    Piece::WKing,
    Piece::BPawn,
    Piece::BKnight,
    Piece::BBishop,
    Piece::BRook,
    Piece::BQueen,
    Piece::BKing,
];

#[inline]
fn H1(h: Key) -> i32 {
    (h & 0x1fff) as i32
}

#[inline]
fn H2(h: Key) -> i32 {
    ((h >> 16) & 0x1fff) as i32
}

#[derive(Debug, Copy, Clone, Default)]
struct StateInfo {
    //Copied when making a move
    material_key: Key,
    pawn_key: Key,
    major_piece_key: Key,
    minor_piece_key: Key,
    non_pawn_key: [Key; COLORNB],
    non_pawn_material: [Value; COLORNB],
    castling_rights: CastlingRights,
    rule_50: i32,
    plies_from_null: i32,
    ep_square: Square,
    //Not copied when making a move
    key: Key,
    checkers_bb: Bitboard,
    blockers_for_king: [Bitboard; COLORNB],
    pinners: [Bitboard; COLORNB],
    check_squares: [Bitboard; PIECE_TYPE_NB],
    captured_piece: Piece,
    repition: i32,
}

impl StateInfo {
    pub fn copy_from_old_to_new(&self, newst: &mut StateInfo) {
        newst.material_key = self.material_key;
        newst.pawn_key = self.pawn_key;
        newst.non_pawn_material = self.non_pawn_material.clone();
        newst.castling_rights = self.castling_rights;
        newst.rule_50 = self.rule_50;
        newst.plies_from_null = self.plies_from_null;
        newst.ep_square = self.ep_square;
    }
}

#[derive(Default)]
struct StateStack {
    states: Vec<StateInfo>,
}

impl StateStack {
    fn new() -> Self {
        Self { states: vec![] }
    }

    #[inline(always)]
    fn push(&mut self, newst: StateInfo) {
        self.states.push(newst);
    }

    #[inline(always)]
    fn pop(&mut self) {
        self.states.pop();
    }
}

pub struct Position {
    board: [Piece; SQNB],
    by_type_bb: [Bitboard; PTNB],
    by_color_bb: [Bitboard; COLORNB],
    piece_count: [i32; PNB],
    castling_rights_mask: [CastlingRights; SQNB],
    castling_rook_square: [Square; CRNB],
    castling_path: [Bitboard; CRNB],
    state_stack: StateStack,
    game_ply: i32,
    side_to_move: Color,
    state_idx: usize,
}

impl Position {
    fn default() -> Self {
        Self {
            board: [Piece::NoPiece; SQNB],
            by_type_bb: [0; PTNB],
            by_color_bb: [0; COLORNB],
            piece_count: [0; PNB],
            castling_rights_mask: [CastlingRights::new_from_n(0); SQNB],
            castling_rook_square: [Square::default(); CRNB],
            castling_path: [0; CRNB],
            state_stack: StateStack::default(),
            game_ply: 0,
            side_to_move: Color::White,
            state_idx: 0,
        }
    }

    fn st(&self) -> &StateInfo {
        let idx: usize = self.state_idx;
        return &self.state_stack.states[idx];
    }

    fn st_mut<'a>(&mut self) -> &mut StateInfo {
        let idx: usize = self.state_idx;
        return &mut self.state_stack.states[idx];
    }
    fn set_castling_right(&mut self, c: Color, rfrom: Square) {
        let kfrom = self.square(c, PieceType::King);
        let side;
        if kfrom < rfrom {
            side = CastlingRights::KingSide;
        } else {
            side = CastlingRights::QueenSide;
        }
        let cr = c & side;
        self.st_mut().castling_rights |= cr;
        self.castling_rights_mask[kfrom as usize] |= cr;
        self.castling_rights_mask[rfrom as usize] |= cr;
        self.castling_rook_square[cr as usize] = rfrom;

        let mut kto: Square;
        let mut rto: Square;

        if (cr & CastlingRights::KingSide) as i32 != 0 {
            kto = Square::SqG1;
        } else {
            kto = Square::SqC1;
        }
        if (cr & CastlingRights::KingSide) as i32 != 0 {
            rto = Square::SqF1;
        } else {
            rto = Square::SqD1;
        }

        kto = kto.relative_square(c);
        rto = rto.relative_square(c);

        if let Some(between_bb) = BETWEEN_BB.get() {
            self.castling_path[cr as usize] = (between_bb[rfrom as usize][rto as usize]
                | between_bb[kfrom as usize][rto as usize])
                & !(kfrom | rfrom as u64); // Can't get why we have to cast here ?
        } else {
            panic!("Attempted to access BETWEEN_BB prior to initialization when setting castling rights");
        }
    }

    fn set_check_info(&mut self) {
        self.update_sliders_blockers(Color::White);
        self.update_sliders_blockers(Color::Black);

        let side_to_move = self.side_to_move;
        let ksq: Square = self.square(!side_to_move, PieceType::King);

        self.st_mut().check_squares[PieceType::Pawn as usize] =
            bb::get_pawn_attacks_bb(!side_to_move, ksq);
        self.st_mut().check_squares[PieceType::Knight as usize] =
            bb::get_pseudo_attacks(PieceType::Knight, ksq);
        self.st_mut().check_squares[PieceType::Bishop as usize] =
            bb::attacks_bb(PieceType::Bishop, ksq, all_pieces!(self));
        self.st_mut().check_squares[PieceType::Rook as usize] =
            bb::attacks_bb(PieceType::Rook, ksq, all_pieces!(self));
        self.st_mut().check_squares[PieceType::Queen as usize] = self.st_mut().check_squares
            [PieceType::Bishop as usize]
            | self.st_mut().check_squares[PieceType::Rook as usize];
        self.st_mut().check_squares[PieceType::King as usize] = 0;
    }

    fn update_sliders_blockers(&mut self, c: Color) {
        let ksq: Square = self.square(c, PieceType::King);
        self.st_mut().blockers_for_king[c as usize] = 0;
        self.st_mut().pinners[!c as usize] = 0;

        let mut snipers: Bitboard = (pseudo_attacks_bb(PieceType::Rook, ksq)
            & pieces_of_types!(&self, PieceType::Queen, PieceType::Rook))
            | (pseudo_attacks_bb(PieceType::Bishop, ksq)
                & pieces_of_types!(&self, PieceType::Queen, PieceType::Bishop))
                & self.pieces_by_color(!c);

        let occupancy: Bitboard = all_pieces!(self) ^ snipers;

        while snipers != 0 {
            let snipers_sq = bb::pop_lsb(&mut snipers);
            let b: Bitboard = bb::between_bb(snipers_sq, ksq) & occupancy;

            if b != 0 && bb::more_than_one(b) {
                self.st_mut().blockers_for_king[c as usize] |= b;
                if b & pieces_by_color_and_pt!(self, c, PieceType::AllPieces) != 0 {
                    self.st_mut().pinners[!c as usize] |= snipers_sq;
                }
            }
        }
    }

    fn set_state(&mut self) {
        if let Some(nopawns) = zobrist::NOPAWNS.get() {
            self.st_mut().pawn_key = *nopawns;
        } else {
            panic!("Attempted to access zobrist - nopawns before initialization");
        }
        self.st_mut().key = 0;
        self.st_mut().material_key = 0;
        self.st_mut().non_pawn_material[Color::White as usize] = 0;
        self.st_mut().non_pawn_material[Color::Black as usize] = 0;

        self.set_check_info();
    }

    fn attackers_to(&self, s: Square, occupied: Bitboard) -> Bitboard {
        return (bb::get_pawn_attacks_bb(Color::Black, s)
            & pieces_by_color_and_pt!(self, Color::White, PieceType::Pawn))
            | (bb::get_pawn_attacks_bb(Color::White, s)
                & pieces_by_color_and_pt!(self, Color::Black, PieceType::Pawn))
            | bb::get_pseudo_attacks(PieceType::Knight, s)
                & pieces_of_types!(self, PieceType::Knight)
            | bb::attacks_bb(PieceType::Rook, s, occupied)
                & pieces_of_types!(self, PieceType::Rook, PieceType::Queen)
            | bb::attacks_bb(PieceType::Bishop, s, occupied)
                & pieces_of_types!(self, PieceType::Bishop, PieceType::Queen)
            | bb::get_pseudo_attacks(PieceType::King, s) & pieces_of_types!(self, PieceType::King);
    }

    fn legal(self, m: Move) -> bool {
        assert!(&m.is_ok());
        let us: Color = self.side_to_move;
        let from = m.from_sq();
        let mut to = m.to_sq();

        assert!(self.moved_piece(m).color() == us);

        if m.type_of() == MoveType::EnPassant {
            let ksq: Square = self.square(us, PieceType::King);
            let capsq: Square = to - pawn_push(us);
            let occupied: Bitboard = (all_pieces!(self) ^ from ^ capsq) | to;
            return false;
        }

        
        if m.type_of() == MoveType::Castling {
            to = if to > from {
                Square::SqG1
            } else {
                Square::SqC1
            }.relative_square(us);

            let step: Direction = if to > from {
                Direction::West
            } else {
                Direction::East
            };

            let mut s = to;
            while s != from {
                if self.attackers_to(s, all_pieces!(self))
                    & pieces_by_color_and_pt!(self, !us, PieceType::AllPieces)
                    != 0
                {
                    return false;
                }
                s += step;
            }
        }

        if self.piece_on(from).type_of() == PieceType::King {
            return self.attackers_to(to, all_pieces!(self) ^ from)
                & pieces_by_color_and_pt!(self, c, PieceType::AllPieces)
                == 0;
        }

        return (self.blockers_for_king(us) & from) == 0
            || bb::alligned(from, to, self.square(us, PieceType::King));
    }

    pub fn all_pieces(&self) -> Bitboard {
        return all_pieces!(self);
    }

    fn pseudo_legal(&self, m: Move, state_stack: &StateStack) -> bool {
        assert!(m.is_ok());

        let us: Color = self.side_to_move;
        let from: Square = m.from_sq();
        let to: Square = m.to_sq();
        let pc: Piece = self.moved_piece(m);

        if m.type_of() != MoveType::Normal {
            todo!()
        }

        if pc != Piece::NoPiece || pc.color() != us {
            return false;
        }

        if pieces_by_color_and_pt!(self, us, PieceType::AllPieces) & to != 0 {
            return false;
        }

        if pc.type_of() == PieceType::Pawn {
            if (RANK8BB | RANK1BB) & to != 0 {
                return false;
            }
            if (bb::get_pawn_attacks_bb(us, from)
                & pieces_by_color_and_pt!(self, !us, PieceType::AllPieces)
                & to
                == 0)
                && !(from + pawn_push(us) == to && self.empty(to))
                && !(from + pawn_push(us) * 2 == to && self.empty(to))
                && !(relative_rank_of_square(us, from) == Rank::Rank2
                    && self.empty(to)
                    && self.empty(to - pawn_push(us)))
            {
                return false;
            }
        }

        if self.checkers() != 0 {
            if pc.type_of() != PieceType::King {
                if more_than_one(self.checkers()) {
                    return false;
                }
                if bb::between_bb(
                    self.square(us, PieceType::King),
                    Square::new_from_n(self.checkers().trailing_zeros() as i32),
                ) & to
                    == 0
                {
                    return false;
                }
            } else if self.attackers_to(to, all_pieces!(self) ^ from)
                & pieces_by_color_and_pt!(self, !us, PieceType::AllPieces)
                != 0
            {
                return false;
            }
        }
        true
    }

    fn gives_check(&self, m: Move) -> bool {
        assert!(m.is_ok());
        assert!(self.moved_piece(m).color() == self.side_to_move);
        let from: Square = m.from_sq();
        let to: Square = m.to_sq();

        // Direct Check
        if self.check_squares(self.piece_on(from).type_of()) & to != 0 {
            return true;
        }

        // Discovered Check
        if self.blockers_for_king(!self.side_to_move) & from != 0 {
            return !alligned(from, to, self.square(!self.side_to_move, PieceType::King))
                || m.type_of() == MoveType::Castling;
        }

        match m.type_of() {
            MoveType::Normal => return false,
            MoveType::Promotion => {
                return attacks_bb(m.promotion_type(), to, all_pieces!(self) ^ from)
                    & self.square(!self.side_to_move, PieceType::King)
                    != 0
            }
            MoveType::EnPassant => {
                let capsq: Square = make_square(to.file_of() as usize, from.rank_of() as usize);
                let b: Bitboard = all_pieces!(self) ^ from ^ capsq | to;
                return (attacks_bb(
                    PieceType::Rook,
                    self.square(!self.side_to_move, PieceType::King),
                    b,
                ) & pieces_by_color_and_pt!(
                    self,
                    !self.side_to_move,
                    PieceType::Queen,
                    PieceType::Rook
                )) | (attacks_bb(
                    PieceType::Bishop,
                    self.square(!self.side_to_move, PieceType::King),
                    b,
                )) & pieces_by_color_and_pt!(
                    self,
                    self.side_to_move,
                    PieceType::Queen,
                    PieceType::Bishop
                ) != 0;
            }
            MoveType::Castling => {
                let rto: Square = if to > from {
                    Square::SqF1
                } else {
                    Square::SqD1
                }
                .relative_square(self.side_to_move);
                return self.check_squares(PieceType::Rook) & rto != 0;
            }
        }
    }

    pub fn do_move(&mut self, m: Move, new_state: &mut StateInfo, gives_check: bool) {
        assert!(m.is_ok());

        //Get Different Zobrist Tables
        let zobrist_side = zobrist::get_zobrist_side();
        let zobrist_psq = zobrist::get_zobrist_psq();
        let zobrist_enpassant = zobrist::get_zobrist_enpassant();

        let mut k: Key = zobrist_side ^ self.st().key;
        //Partial copy of some of the fields of the old state. The rest is recomputed anyway

        //Setting up to add the new state
        self.st().copy_from_old_to_new(new_state);
        self.state_idx = self.state_stack.states.len();
        self.state_stack.push(*new_state);
        self.game_ply += 1;
        self.st_mut().rule_50 += 1;
        self.st_mut().plies_from_null += 1;

        let us: Color = self.side_to_move;
        let them: Color = !us;
        let mut from: Square = m.from_sq();
        let mut to: Square = m.to_sq();
        let pc: Piece = self.piece_on(from);

        assert!(self.piece_on(from) != Piece::NoPiece);

        //Special Case if the move is Enpassant
        let mut captured: Piece = if m.type_of() == MoveType::EnPassant {
            make_piece(them, PieceType::Pawn)
        } else {
            self.piece_on(to)
        };

        assert!(pc.color() == us);
        assert!(captured.type_of() != PieceType::King);

        //Handle Castling
        if m.type_of() == MoveType::Castling {
            let mut rfrom: Square = Square::SqNone;
            let mut rto: Square = Square::SqNone;
            self.do_castling(us, &mut from, &mut to, &mut rfrom, &mut rto);
            k ^= zobrist_psq[captured as usize][rfrom as usize]
                ^ zobrist_psq[captured as usize][rto as usize];
            captured = Piece::NoPiece;
        }

        //Handling Captures
        if captured != Piece::NoPiece {
            let mut capsq: Square = to;
            if captured.type_of() == PieceType::Pawn {
                capsq -= if m.type_of() == MoveType::EnPassant {
                    pawn_push(us)
                } else {
                    Direction::None
                };
                self.st_mut().pawn_key ^= zobrist_psq[captured as usize][capsq as usize];
            } else {
                self.st_mut().non_pawn_material[them as usize] -= PIECEVALUE[captured as usize];
                self.st_mut().non_pawn_key[them as usize] ^=
                    zobrist_psq[captured as usize][capsq as usize];
                if pc.type_of() == PieceType::Queen || pc.type_of() == PieceType::Rook {
                    self.st_mut().major_piece_key ^= zobrist_psq[captured as usize][capsq as usize];
                } else {
                    self.st_mut().minor_piece_key ^= zobrist_psq[captured as usize][capsq as usize];
                }
            }
            self.remove_piece(capsq);
            k ^= zobrist_psq[captured as usize][capsq as usize];
            self.st_mut().material_key ^=
                zobrist_psq[captured as usize][self.piece_count[captured as usize] as usize];
            self.st_mut().rule_50 = 0;
        }
        k ^= zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];

        if self.st().ep_square != Square::SqNone {
            k ^= zobrist_enpassant[self.st().ep_square.file_of() as usize];
            self.st_mut().ep_square = Square::SqNone;
        }

        if self.st().castling_rights != CastlingRights::NoCastling
            && self.castling_rights_mask[from as usize] | self.castling_rights_mask[to as usize]
                != CastlingRights::NoCastling
        {
            let castling_rights_mask_from = self.castling_rights_mask[from as usize];
            let castling_rights_mask_to = self.castling_rights_mask[to as usize];
            let zobrist_castling = zobrist::get_zobrist_castling();

            k ^= zobrist_castling[self.st().castling_rights as usize];
            self.st_mut().castling_rights &= !(castling_rights_mask_from | castling_rights_mask_to);
            k ^= zobrist_castling[self.st().castling_rights as usize];
        }

        //Actually Move the Piece
        if m.type_of() != MoveType::Castling {
            self.move_piece(from, to);
        }

        //Special Hnadling if the Moved. Handle Promotion and Some Enpassant Stuff.
        if pc.type_of() == PieceType::Pawn {
            if to as i32 ^ from as i32 == 16
            // todo: I Don't like the cast to u64 when calling pawn_push here
                && (pawn_attacks_bb((to - pawn_push(us)) as u64, us)
                    & pieces_by_color_and_pt!(self, them, PieceType::Pawn)
                    != 0)
            {
                self.st_mut().ep_square = to - pawn_push(us);
                k ^= zobrist_enpassant[self.st().ep_square.file_of() as usize];
            }

            if m.type_of() == MoveType::Promotion {
                let promotion: Piece = make_piece(us, m.promotion_type());
                let promotion_type: PieceType = promotion.type_of();

                //Remove the Pawn and add the New Piece
                self.remove_piece(to);
                self.put_piece(promotion, to);

                //Update the Hash Keys
                k ^= zobrist_psq[pc as usize][to as usize]
                    ^ zobrist_psq[promotion as usize][to as usize];
                self.st_mut().pawn_key ^= zobrist_psq[pc as usize][to as usize];

                self.st_mut().material_key ^= zobrist_psq[promotion as usize]
                    [self.piece_count[promotion as usize] as usize - 1]
                    ^ zobrist_psq[pc as usize][self.piece_count[pc as usize] as usize];
                if promotion_type == PieceType::Queen || promotion_type == PieceType::Rook {
                    self.st_mut().major_piece_key ^= zobrist_psq[promotion as usize][to as usize];
                } else {
                    self.st_mut().minor_piece_key ^= zobrist_psq[promotion as usize][to as usize];
                }

                self.st_mut().non_pawn_material[us as usize] += PIECEVALUE[promotion as usize];
            }

            self.st_mut().pawn_key ^=
                zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];

            self.st_mut().rule_50 = 0;
        } else {
            self.st_mut().non_pawn_key[us as usize] ^=
                zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];
            if pc.type_of() == PieceType::King {
                self.st_mut().major_piece_key ^=
                    zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];

                self.st_mut().minor_piece_key ^=
                    zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];
            } else if pc.type_of() == PieceType::Queen || pc.type_of() == PieceType::Rook {
                self.st_mut().major_piece_key ^=
                    zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];
            } else {
                self.st_mut().minor_piece_key ^=
                    zobrist_psq[pc as usize][from as usize] ^ zobrist_psq[pc as usize][to as usize];
            }
        }
        self.st_mut().captured_piece = captured;
        self.st_mut().key = k;

        self.st_mut().checkers_bb = if gives_check {
            self.attackers_to(
                self.square(them, PieceType::King),
                pieces_by_color_and_pt!(self, us, PieceType::AllPieces),
            )
        } else {
            0
        };

        self.side_to_move = !self.side_to_move;
        //Todo:: Test this Function Fully.
        self.set_check_info();
        self.st_mut().repition = 0;

        //Three-Fold Repitition
        let end = std::cmp::min(self.st().rule_50, self.st().plies_from_null);

        if end >= 4 {
            todo!()
        }
    }

    pub fn undo_move(&mut self, mv: Move) {
        assert!(mv.is_ok());

        self.side_to_move = !self.side_to_move;
        let us: Color = self.side_to_move;
        let mut from: Square = mv.from_sq();
        let mut to: Square = mv.to_sq();

        assert!(self.empty(from) || mv.type_of() == MoveType::Castling);
        assert!(self.st().captured_piece.type_of() != PieceType::King);

        if mv.type_of() == MoveType::Promotion {
            self.remove_piece(to);
            let pc = make_piece(us, PieceType::Pawn);
            self.put_piece(pc, from);
        } else if mv.type_of() == MoveType::Castling {
            let mut rfrom: Square = Square::SqNone;
            let mut rto: Square = Square::SqNone;
            self.undo_castling(us, &mut from, &mut to, &mut rfrom, &mut rto);
        } else {
            self.move_piece(to, from);

            if self.st().captured_piece != Piece::NoPiece {
                let mut capsq: Square = to;
                if mv.type_of() == MoveType::EnPassant {
                    capsq -= pawn_push(us);
                }
                self.put_piece(self.st().captured_piece, capsq);
            }
        }
        self.state_idx -= 1;
        self.game_ply -= 1;
    }

    fn do_castling(
        &mut self,
        side: Color,
        from: &mut Square,
        to: &mut Square,
        rfrom: &mut Square,
        rto: &mut Square,
    ) {
        let kingside: bool = to > from;
        *rfrom = *to;
        *rto = if kingside {
            Square::SqF1.relative_square(side)
        } else {
            Square::SqD1.relative_square(side)
        };

        *to = if kingside {
            Square::SqG1.relative_square(side)
        } else {
            Square::SqC1.relative_square(side)
        };

        self.remove_piece(*from);
        self.remove_piece(*rfrom);
        self.board[*from as usize] = Piece::NoPiece;
        self.board[*rfrom as usize] = Piece::NoPiece;
        self.put_piece(make_piece(side, PieceType::King), *to);
        self.put_piece(make_piece(side, PieceType::King), *to);
    }

    fn undo_castling(
        &mut self,
        side: Color,
        from: &mut Square,
        to: &mut Square,
        rfrom: &mut Square,
        rto: &mut Square,
    ) {
        let kingside: bool = to > from;
        *rfrom = *to;
        *rto = if kingside {
            Square::SqF1.relative_square(side)
        } else {
            Square::SqD1.relative_square(side)
        };

        *to = if kingside {
            Square::SqG1.relative_square(side)
        } else {
            Square::SqC1.relative_square(side)
        };

        self.remove_piece(*to);
        self.remove_piece(*rto);
        self.board[*to as usize] = Piece::NoPiece;
        self.board[*rto as usize] = Piece::NoPiece;
        self.put_piece(make_piece(side, PieceType::King), *from);
        self.put_piece(make_piece(side, PieceType::King), *rfrom);
    }

    #[inline]
    pub fn square(&self, c: Color, pt: PieceType) -> Square {
        return Square::new_from_n(pieces_by_color_and_pt!(&self, c, pt).trailing_zeros() as i32);
    }
    #[inline(always)]
    // fn st_mut(&mut self) -> &mut StateInfo {
    //     self.state_stack.current_mut()
    // }
    
    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    fn piece_on(&self, s: Square) -> Piece {
        self.board[s as usize]
    }

    #[inline]
    fn empty(&self, s: Square) -> bool {
        self.board[s as usize] == Piece::NoPiece
    }

    #[inline]
    fn moved_piece(&self, m: Move) -> Piece {
        self.piece_on(m.from_sq())
    }

    #[inline]
    pub fn pieces_by_piecetype(&self, pt: PieceType) -> Bitboard {
        self.by_type_bb[pt as usize]
    }

    #[inline]
    pub fn pieces_by_color(&self, color: Color) -> Bitboard {
        self.by_color_bb[color as usize]
    }

    #[inline]
    pub fn ep_square(&self) -> Square {
        self.st().ep_square
    }

    #[inline]
    fn can_castle(&self, cr: CastlingRights) -> bool {
        self.st().castling_rights as i32 & cr as i32 != 0
    }

    #[inline]
    pub fn checkers(&self) -> Bitboard {
        self.st().checkers_bb
    }

    #[inline]
    pub fn blockers_for_king(&self, c: Color) -> Bitboard {
        self.st().blockers_for_king[c as usize]
    }

    #[inline]
    fn pinners(&self, c: Color) -> Bitboard {
        self.st().pinners[c as usize]
    }

    #[inline]
    fn check_squares(&self, pt: PieceType) -> Bitboard {
        self.st().check_squares[pt as usize]
    }

    #[inline]
    fn pawn_key(&self) -> Key {
        self.st().pawn_key
    }

    #[inline]
    fn material_key(&self) -> Key {
        self.st().material_key
    }

    #[inline]
    fn non_pawn_material(&self, c: Color) -> Value {
        self.st().non_pawn_material[c as usize]
    }

    #[inline]
    fn game_ply(&self) -> i32 {
        self.game_ply
    }

    #[inline]
    fn rule50_count(&self) -> i32 {
        self.st().rule_50
    }

    #[inline]
    fn captured_piece(&self) -> Piece {
        self.st().captured_piece
    }

    fn put_piece(&mut self, pc: Piece, s: Square) {
        let pt = pc.type_of();
        self.board[s as usize] = pc;
        self.by_type_bb[pt as usize] |= s;
        self.by_type_bb[PieceType::AllPieces as usize] |= self.by_type_bb[pt as usize];
        self.by_color_bb[pc.color() as usize] |= s;
        self.piece_count[pc as usize] += 1;
        self.piece_count[make_piece(pc.color(), pc.type_of()) as usize] += 1;
    }

    fn remove_piece(&mut self, s: Square) {
        let pc = self.board[s as usize];
        self.by_type_bb[(PieceType::AllPieces as i32 + 1) as usize] ^= s;
        self.by_type_bb[pc.type_of() as usize] ^= s;
        self.by_color_bb[pc.color() as usize] ^= s;
        self.board[s as usize] = Piece::NoPiece;
        self.piece_count[pc as usize] -= 1;
        self.piece_count[make_piece(pc.color(), pc.type_of()) as usize] -= 1;
    }

    fn move_piece(&mut self, f: Square, t: Square) {
        let pc = self.board[f as usize];
        let from_to: Bitboard = f.bb() | t;
        self.by_type_bb[(PieceType::AllPieces as i32 + 1) as usize] ^= from_to;
        self.by_type_bb[pc.type_of() as usize] ^= from_to;
        self.by_type_bb[pc.color() as usize] ^= from_to;
        self.board[f as usize] = Piece::NoPiece;
        self.board[t as usize] = pc;
    }
    //Initialize various tables used for cycle detection and zobrist hashing
    pub fn init() {
        zobrist::init_zobrist();
        if let Some(psq) = zobrist::PSQ.get() {
        } else {
            panic!("Error Initializing zobrist PSQ table");
        }

        if let Some(psq) = zobrist::ENPASSANT.get() {
        } else {
            panic!("Error Initializing zobrist Enpassant table");
        }

        if let Some(psq) = zobrist::CASTLING.get() {
        } else {
            panic!("Error Initializing zobrist Castling table");
        }

        if let Some(psq) = zobrist::SIDE.get() {
        } else {
            panic!("Error Initializing zobrist Side table");
        }

        if let Some(psq) = zobrist::NOPAWNS.get() {
        } else {
            panic!("Error Initializing zobrist Nopawns table");
        }

        let mut cuckoo: [Key; 8192] = [0; 8192];
        let mut cuckoomove: [Move; 8192] = [Move::none(); 8192];
        let zpsq = zobrist::get_zobrist_psq();
        let zside = zobrist::get_zobrist_side();
        let mut count = 0;
        for &pc in &pieces {
            for i in SQA1..=SQH8 {
                let s1 = Square::new_from_n(i as i32);
                for j in i + 1..=SQH8 {
                    let s2 = Square::new_from_n(j as i32);
                    if (pc.type_of() as usize != Piece::BPawn as usize
                        && pc.type_of() as usize != Piece::BPawn as usize)
                        && (bb::attacks_bb(pc.type_of(), s1, 0) & s2) != 0
                    {
                        let mut mv = Move::new_from_to_sq(s1, s2);
                        let mut key = zpsq[pc as usize][i] ^ zpsq[pc as usize][j] ^ zside;
                        let mut m = H1(key);
                        'inner: loop {
                            std::mem::swap(&mut cuckoo[m as usize], &mut key);
                            std::mem::swap(&mut cuckoomove[m as usize], &mut mv);
                            if mv == Move::none() {
                                break 'inner;
                            }
                            m = if m == H1(key) { H2(key) } else { H1(key) };
                        }
                        count += 1;
                    }
                }
            }
        }
        assert!(count == 3668);
    }

    #[inline]
    fn pieces_by_type(&self, pt: PieceType) -> Bitboard {
        return self.by_type_bb[pt as usize];
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n +---+---+---+---+---+---+---+---+")?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let piece = self.piece_on(make_square(file, rank));
                let c = PIECE_TO_CHAR.chars().nth(piece as usize).unwrap();
                write!(f, " | {}", c)?;
            }
            writeln!(f, " | {}", rank + 1)?;
            writeln!(f, " +---+---+---+---+---+---+---+---+")?;
        }
        writeln!(f, "   a   b   c   d   e   f   g   h")?;
        Ok(())
    }
}

mod test {
    use super::*;

    #[test]
    fn test_position_display() {
        let mut position = Position::default();
        position.put_piece(Piece::BBishop, Square::SqA1);
        position.put_piece(Piece::WBishop, Square::SqA8);
        position.put_piece(Piece::WRook, Square::SqE4);
        println!("{}", position);
    }

    //@todo: Expand this function's testing
    #[test]
    fn test_pieces_by_piece_types_macro() {
        let mut position = Position::default();
        position.put_piece(Piece::BBishop, Square::SqA1);
        position.put_piece(Piece::WBishop, Square::SqA8);
        position.put_piece(Piece::WRook, Square::SqE4);
        let res = pieces_of_types!(&position, PieceType::Bishop, PieceType::Rook);
        let r = bb::pretty(res);
        println!("{}", r);
    }

    #[test]
    fn test_do_move() {
        bb::init();
        Position::init();
        let mut position = Position::default();
        position.put_piece(Piece::WPawn, Square::SqA2);
        position.put_piece(Piece::WPawn, Square::SqB2);
        position.put_piece(Piece::WPawn, Square::SqC2);
        position.put_piece(Piece::WPawn, Square::SqD2);
        position.put_piece(Piece::WPawn, Square::SqE2);
        position.put_piece(Piece::WPawn, Square::SqF2);
        position.put_piece(Piece::WPawn, Square::SqG2);
        position.put_piece(Piece::WPawn, Square::SqH2);
        position.put_piece(Piece::WRook, Square::SqA1);
        position.put_piece(Piece::WRook, Square::SqH1);
        position.put_piece(Piece::WKnight, Square::SqB1);
        position.put_piece(Piece::WKnight, Square::SqG1);
        position.put_piece(Piece::WBishop, Square::SqC1);
        position.put_piece(Piece::WBishop, Square::SqF1);
        position.put_piece(Piece::WKing, Square::SqE1);
        position.put_piece(Piece::WQueen, Square::SqD1);

        position.put_piece(Piece::BPawn, Square::SqA7);
        position.put_piece(Piece::BPawn, Square::SqB7);
        position.put_piece(Piece::BPawn, Square::SqC7);
        position.put_piece(Piece::BPawn, Square::SqD7);
        position.put_piece(Piece::BPawn, Square::SqE7);
        position.put_piece(Piece::BPawn, Square::SqF7);
        position.put_piece(Piece::BPawn, Square::SqG7);
        position.put_piece(Piece::BPawn, Square::SqH7);

        position.put_piece(Piece::BRook, Square::SqA8);
        position.put_piece(Piece::BRook, Square::SqH8);
        position.put_piece(Piece::BKnight, Square::SqB8);
        position.put_piece(Piece::BKnight, Square::SqG8);
        position.put_piece(Piece::BBishop, Square::SqC8);
        position.put_piece(Piece::BBishop, Square::SqF8);
        position.put_piece(Piece::BKing, Square::SqE8);
        position.put_piece(Piece::BQueen, Square::SqD8);

        println!("{}", position);
        let mut st = StateStack::default();
        st.push(StateInfo::default());
        let mut newst = StateInfo::default();

        //@Todo: Separate these three functions neatly
        test_normal_move_do_undo(&mut position, &mut newst, &mut st);
        test_enpassant_do_undo(&mut position, &mut newst, &mut st);
        test_promotion_moves(&mut position, &mut newst, &mut st);
    }

    fn test_enpassant_do_undo(position: &mut Position, newst: &mut StateInfo, st: &mut StateStack) {
        let e2e4: Move = Move::new_from_to_sq(Square::SqE2, Square::SqE4);
        let d7d5: Move = Move::new_from_to_sq(Square::SqD7, Square::SqD5);
        let e4e5: Move = Move::new_from_to_sq(Square::SqE4, Square::SqE5);
        let f7f5: Move = Move::new_from_to_sq(Square::SqF7, Square::SqF5);
        let mut ef: Move = Move::new_from_to_sq(Square::SqE5, Square::SqF6);

        ef.set_move_to_variant(MoveType::EnPassant);
        assert!(ef.type_of() == MoveType::EnPassant);

        position.do_move(e2e4, newst, false);
        println!("1.E4\n{}", &position);

        position.do_move(d7d5, newst, false);
        println!("1..D5\n{}", &position);

        position.do_move(e4e5, newst, false);
        println!("2.E5\n{}", &position);

        position.do_move(f7f5, newst, false);
        println!("2..F5\n{}", &position);

        position.do_move(ef, newst, false);
        println!("3..EF\n{}", &position);
    }

    fn test_normal_move_do_undo(
        position: &mut Position,
        newst: &mut StateInfo,
        st: &mut StateStack,
    ) {
        let e4: Move = Move::new_from_to_sq(Square::SqE2, Square::SqE4);
        let d5: Move = Move::new_from_to_sq(Square::SqD7, Square::SqD5);
        let ed: Move = Move::new_from_to_sq(Square::SqE4, Square::SqD5);
        let nf6: Move = Move::new_from_to_sq(Square::SqG8, Square::SqF6);
        let d4: Move = Move::new_from_to_sq(Square::SqD2, Square::SqD4);
        let nxd5: Move = Move::new_from_to_sq(Square::SqF6, Square::SqD5);

        position.do_move(e4, newst, false);
        println!("1.E4\n{}", &position);
        position.do_move(d5, newst, false);
        println!("1...D5\n{}", &position);
        position.do_move(ed, newst, false);
        println!("2.ED\n{}", &position);
        position.do_move(nf6, newst, false);
        println!("2..Nf6\n{}", &position);
        position.do_move(d4, newst, false);
        println!("3.D4\n{}", &position);
        position.do_move(nxd5, newst, false);
        println!("3...NxD5\n{}", &position);

        position.undo_move(nxd5);
        position.undo_move(d4);
        position.undo_move(nf6);

        println!("{}", position);
    }

    fn test_promotion_moves(position: &mut Position, newst: &mut StateInfo, st: &mut StateStack) {
        position.remove_piece(Square::SqG7);
        position.remove_piece(Square::SqG8);

        position.put_piece(Piece::WPawn, Square::SqG7);
        println!("1.Position Before Promotion\n{}", &position);

        //Test Promotion Without Capturing
        let mut promote_no_capture = Move::new_from_to_sq(Square::SqG7, Square::SqG8);
        promote_no_capture.set_move_to_variant(MoveType::Promotion);

        //Test Knight Promotion No Capture
        promote_no_capture.set_promotion_type(PieceType::Knight);
        position.do_move(promote_no_capture, newst, false);
        println!("1.Knight Promote\n{}", &position);
        position.undo_move(promote_no_capture);

        //Test Bishop Promotion No Capture
        promote_no_capture.set_promotion_type(PieceType::Bishop);
        position.do_move( promote_no_capture, newst, false);
        println!("1.Bishop Promote\n{}", &position);
        position.undo_move(promote_no_capture);

        //Test Rook Promotion No Capture
        promote_no_capture.set_promotion_type(PieceType::Rook);
        position.do_move(promote_no_capture, newst, false);
        println!("1.Rook Promote\n{}", &position);
        position.undo_move(promote_no_capture);

        //Test Queen Promotion No Capture
        promote_no_capture.set_promotion_type(PieceType::Queen);
        position.do_move(promote_no_capture, newst, false);
        println!("1.Queen Promote\n{}", &position);
        position.undo_move(promote_no_capture);

        //Promoting While Capturing a Piece
        let mut promote_capture = Move::new_from_to_sq(Square::SqG7, Square::SqH8);
        promote_capture.set_move_to_variant(MoveType::Promotion);

        //Test Knight Promotion Capture
        promote_capture.set_promotion_type(PieceType::Knight);
        position.do_move(promote_capture, newst, false);
        println!("1.Knight Promote\n{}", &position);
        position.undo_move(promote_capture);

        //Test Bishop Promotion Capture
        promote_capture.set_promotion_type(PieceType::Bishop);
        position.do_move( promote_capture, newst, false);
        println!("1.Bishop Promote\n{}", &position);
        position.undo_move(promote_capture);

        //Test Rook Promotion Capture
        promote_capture.set_promotion_type(PieceType::Rook);
        position.do_move( promote_capture, newst, false);
        println!("1.Rook Promote\n{}", &position);
        position.undo_move(promote_capture);

        //Test Queen Promotion Capture
        promote_capture.set_promotion_type(PieceType::Queen);
        position.do_move(promote_capture, newst, false);
        println!("1.Queen Promote\n{}", &position);
        position.undo_move(promote_capture);
    }

    #[test]
    fn test_set_check_info() {
        todo!()
    }

    fn test_update_sliders_blockers() {
        todo!()
    }

    #[test]
    fn test_macros() {
        bb::init();
        Position::init();

        let mut position = Position::default();
        let pieces_of_types = pieces_of_types!(&position, PieceType::Queen, PieceType::Rook);
        assert_eq!(pieces_of_types, 0);

        position.put_piece(Piece::WQueen, Square::SqA1);
        position.put_piece(Piece::WRook, Square::SqE4);
        let res: u64 = 1 << Square::SqE4 as i32 | 1 << Square::SqA1 as i32;
        let pieces_of_types = pieces_of_types!(&position, PieceType::Queen, PieceType::Rook);
        assert_eq!(pieces_of_types, res);

        position.put_piece(Piece::BRook, Square::SqE5);
        let res: u64 =
            1 << Square::SqE4 as i32 | 1 << Square::SqA1 as i32 | 1 << Square::SqE5 as i32;
        let pieces_of_types = pieces_of_types!(&position, PieceType::Queen, PieceType::Rook);
        assert_eq!(pieces_of_types, res);

        let white_pieces =
            pieces_by_color_and_pt!(&position, Color::White, PieceType::Rook, PieceType::Queen);
        let res: u64 = 1 << Square::SqE4 as i32 | 1 << Square::SqA1 as i32;
        assert_eq!(white_pieces, res);

        let black_pieces =
            pieces_by_color_and_pt!(&position, Color::Black, PieceType::Rook, PieceType::Queen);
        let res = 1 << Square::SqE5 as i32;
        assert_eq!(res, black_pieces);

        let all_pieces = all_pieces!(&position);
        let res: u64 =
            1 << Square::SqE4 as i32 | 1 << Square::SqA1 as i32 | 1 << Square::SqE5 as i32;
        assert_eq!(all_pieces, res);
    }
}
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
