use crate::board::bitboard::more_than_one;
use crate::board::bitboard as bb;
use crate::board::bitboard::pawn_attacks_bb;
use crate::board::bitboard::RANK1BB;
use crate::board::bitboard::RANK8BB;
use crate::board::position_macros;
use crate::board::zobrist;
use crate::board::zobrist::ENPASSANT;
use crate::misc::*;
use crate::types::*;
use std::fmt;
use std::sync::OnceLock;
use std::vec::Vec;

use super::bitboard::attacks_bb;
use super::bitboard::get_pseudo_attacks;
use super::bitboard::pseudo_attacks_bb;
use super::bitboard::BETWEEN_BB;

const PIECE_TYPE_NB: usize = PieceType::PieceTypeNb as usize;
const PIECE_TO_CHAR: &str = " PNBRQK pnbrqk";
const MAX_PLY: usize = 246; // Maximum search depth

pub static CUCKOO: OnceLock<[Key; 8192]> = OnceLock::new();
pub static CUCKOO_MOVE: OnceLock<[Key; 8192]> = OnceLock::new();

macro_rules! pieces_of_types {
    ($pos: expr, $pt: expr) => {
        $pos.pieces_by_piecetype($pt)
    };

    ($pos: expr, $pt: expr, $($rest_pts: expr),+) => {
        $pos.pieces_by_piecetype($pt) | pieces_of_types!($pos, $($rest_pts),+)
    }
}

macro_rules! pieces_by_color_and_pt {
    ($pos: expr, $color: expr, $pt: expr) => {
        pieces_of_types!($pos, $pt)
    };

    ($pos: expr, $color: expr, $pt: expr, $($rest_pt: expr),+) => {
        $pos.pieces_by_color($color) & (pieces_by_color_and_pt!($pos, $color, $pt) | pieces_by_color_and_pt!($pos, $color, $($rest_pt),+))
    };
}

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
    non_pawn_material: [Value; COLORNB],
    castling_rights: CastlingRights,
    rule_50: i32,
    plies_from_null: i32,
    ep_square: Square,
    state_idx: usize,
    //Not copied when making a move
    key: Key,
    checkers_bb: Bitboard,
    blockers_for_king: [Bitboard; COLORNB],
    pinners: [Bitboard; COLORNB],
    check_squares: [Bitboard; PIECE_TYPE_NB],
    captured_piece: Piece,
    repition: i32,
}

#[derive(Default)]
struct StateStack {
    states: Vec<StateInfo>,
    current: usize,
}

impl StateStack {
    fn new() -> Self {
        Self {
            states: vec![
                StateInfo {
                    material_key: 0,
                    pawn_key: 0,
                    non_pawn_material: [0; COLORNB],
                    castling_rights: CastlingRights::AnyCastling,
                    rule_50: 0,
                    plies_from_null: 0,
                    ep_square: Square::SqNone,
                    state_idx: 0,
                    key: 0,
                    checkers_bb: 0,
                    blockers_for_king: [0; COLORNB],
                    pinners: [0; COLORNB],
                    check_squares: [0; PIECE_TYPE_NB],
                    captured_piece: Piece::NoPiece,
                    repition: 0,
                };
                MAX_PLY
            ],
            current: 0,
        }
    }

    #[inline(always)]
    fn push(&mut self) -> &mut StateInfo {
        self.current += 1;
        &mut self.states[self.current - 1]
    }

    #[inline(always)]
    fn pop(&mut self) {
        self.current -= 1;
    }

    #[inline(always)]
    fn current(&self) -> &StateInfo {
        &self.states[self.current - 1]
    }

    #[inline(always)]
    fn current_mut(&mut self) -> &mut StateInfo {
        &mut self.states[self.current - 1]
    }
}

struct Position {
    board: [Piece; SQNB],
    by_type_bb: [Bitboard; PTNB],
    by_color_bb: [Bitboard; COLORNB],
    piece_count: [i32; PNB],
    castling_rights_mask: [i32; SQNB],
    castling_rook_square: [Square; CRNB],
    castling_path: [Bitboard; CRNB],
    state_stack: StateStack,
    game_ply: i32,
    side_to_move: Color,
    st: StateInfo,
}

impl Position {
    // pub const fn default() -> Self {
    //     // let prng = Prng::new(1070372);

    // }
    fn default() -> Self {
        Self {
            st: StateInfo::default(),
            board: [Piece::NoPiece; SQNB],
            by_type_bb: [0; PTNB],
            by_color_bb: [0; COLORNB],
            piece_count: [64; PNB],
            castling_rights_mask: [0; SQNB],
            castling_rook_square: [Square::default(); CRNB],
            castling_path: [0; CRNB],
            state_stack: StateStack::default(),
            game_ply: 0,
            side_to_move: Color::White,
        }
    }

    pub fn set_castling_right(&mut self, c: Color, rfrom: Square) {
        let kfrom = self.square(c, PieceType::King);
        let side;
        if kfrom < rfrom {
            side = CastlingRights::KingSide;
        } else {
            side = CastlingRights::QueenSide;
        }
        let cr = c & side;
        self.st.castling_rights |= cr;
        self.castling_rights_mask[kfrom as usize] |= cr as i32;
        self.castling_rights_mask[rfrom as usize] |= cr as i32;
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

    pub fn set_check_info(&mut self) {
        self.update_sliders_blockers(Color::White);
        self.update_sliders_blockers(Color::Black);

        let side_to_move = self.side_to_move;
        let ksq: Square = self.square(!side_to_move, PieceType::King);

        self.st.check_squares[PieceType::Pawn as usize] =
            bb::get_pawn_attacks_bb(!side_to_move, ksq);
        self.st.check_squares[PieceType::Knight as usize] =
            bb::get_pseudo_attacks(PieceType::Knight, ksq);
        self.st.check_squares[PieceType::Bishop as usize] = bb::attacks_bb(
            PieceType::Bishop,
            ksq,
            pieces_of_types!(self, PieceType::AllPieces),
        );
        self.st.check_squares[PieceType::Rook as usize] = bb::attacks_bb(
            PieceType::Rook,
            ksq,
            pieces_of_types!(self, PieceType::AllPieces),
        );
        self.st.check_squares[PieceType::Queen as usize] = self.st.check_squares
            [PieceType::Bishop as usize]
            | self.st.check_squares[PieceType::Rook as usize];
        self.st.check_squares[PieceType::King as usize] = 0;
    }

    pub fn update_sliders_blockers(&mut self, c: Color) {
        let ksq: Square = self.square(c, PieceType::King);
        self.st.blockers_for_king[c as usize] = 0;
        self.st.pinners[!c as usize] = 0;

        let mut snipers: Bitboard = (pseudo_attacks_bb(PieceType::Rook, ksq)
            & pieces_of_types!(&self, PieceType::Queen, PieceType::Rook))
            | (pseudo_attacks_bb(PieceType::Bishop, ksq)
                & pieces_of_types!(&self, PieceType::Queen, PieceType::Bishop))
                & self.pieces_by_color(!c);

        let occupancy: Bitboard = self.pieces(PieceType::AllPieces) ^ snipers;

        while snipers != 0 {
            let snipers_sq = bb::pop_lsb(&mut snipers);
            let b: Bitboard = bb::between_bb(snipers_sq, ksq);

            if b != 0 && bb::more_than_one(b) {
                self.st.blockers_for_king[c as usize] |= b;
                if b & pieces_by_color_and_pt!(self, c, PieceType::AllPieces) != 0 {
                    self.st.pinners[!c as usize] |= snipers_sq;
                }
            }
        }
    }

    pub fn set_state(&mut self) {
        if let Some(nopawns) = zobrist::NOPAWNS.get() {
            self.st.pawn_key = *nopawns;
        } else {
            panic!("Attempted to access zobrist - nopawns before initialization");
        }
        self.st.key = 0;
        self.st.material_key = 0;
        self.st.non_pawn_material[Color::White as usize] = 0;
        self.st.non_pawn_material[Color::Black as usize] = 0;

        self.set_check_info();
    }

    #[inline(always)]
    fn st(&self) -> &StateInfo {
        self.state_stack.current()
    }

    pub fn attackers_to(&self, s: Square, occupied: Bitboard) -> Bitboard {
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

    pub fn legal(self, m: Move) -> bool {
        assert!(&m.is_ok());
        let us: Color = self.side_to_move;
        let from = m.from_sq();
        let mut to = m.to_sq();

        assert!(self.moved_piece(m).color() == us);

        if m.type_of() == MoveType::EnPassant {
            let ksq: Square = self.square(us, PieceType::King);
            let capsq: Square = to - pawn_push(us);
            let occupied: Bitboard =
                (pieces_of_types!(self, PieceType::AllPieces) ^ from ^ capsq) | to;
            return false;
        }

        if m.type_of() == MoveType::Castling {
            to = if to > from {
                Square::SqG1
            } else {
                Square::SqC1
            }
            .relative_square(us);

            let step: Direction = if to > from {
                Direction::West
            } else {
                Direction::East
            };

            let mut s = to;
            while s != from {
                if self.attackers_to(s, pieces_of_types!(self, PieceType::AllPieces))
                    & pieces_by_color_and_pt!(self, !us, PieceType::AllPieces)
                    != 0
                {
                    return false;
                }
                s += step;
            }
        }

        if self.piece_on(from).type_of() == PieceType::King {
            return self.attackers_to(to, pieces_of_types!(self, PieceType::AllPieces) ^ from)
                & pieces_by_color_and_pt!(self, c, PieceType::AllPieces)
                == 0;
        }

        return (self.blockers_for_king(us) & from) == 0
            || bb::alligned(from, to, self.square(us, PieceType::King));
    }

    pub fn pseudo_legal(&self, m: Move) -> bool {
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
                && !(relative_rank_of_square(us, from) == Rank::Rank2 && self.empty(to) && self.empty(to - pawn_push(us)))
            {
                return false;
            }
        }

        if self.checkers() != 0 {
            if pc.type_of() != PieceType::King {
                if more_than_one(self.checkers()) {
                    return false
                }
                if bb::between_bb(self.square(us, PieceType::King), Square::new_from_n(self.checkers().trailing_zeros() as i32)) & to == 0 {
                    return false
                }
            }else if self.attackers_to(to, pieces_of_types!(self, PieceType::AllPieces) ^ from) & pieces_by_color_and_pt!(self, !us, PieceType::AllPieces) != 0 {
                return false
            }

        }
        true
    }

    pub fn gives_check(&self, m: Move) {
        assert!(m.is_ok());
        assert!(self.moved_piece(m).color() == self.side_to_move);
        let from: Square = m.from_sq(); 
        let to: Square = m.to_sq(); 

        
    }
    #[inline]
    fn square(&self, c: Color, pt: PieceType) -> Square {
        return Square::new_from_n(pieces_by_color_and_pt!(&self, c, pt).trailing_zeros() as i32);
    }
    #[inline(always)]
    fn st_mut(&mut self) -> &mut StateInfo {
        self.state_stack.current_mut()
    }

    #[inline]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    pub fn piece_on(&self, s: Square) -> Piece {
        self.board[s as usize]
    }

    #[inline]
    pub fn empty(&self, s: Square) -> bool {
        self.board[s as usize] == Piece::NoPiece
    }

    #[inline]
    pub fn moved_piece(&self, m: Move) -> Piece {
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
    pub fn can_castle(&self, cr: CastlingRights) -> bool {
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
    pub fn pinners(&self, c: Color) -> Bitboard {
        self.st().pinners[c as usize]
    }

    #[inline]
    pub fn check_squares(&self, pt: PieceType) -> Bitboard {
        self.st().check_squares[pt as usize]
    }

    #[inline]
    pub fn pawn_key(&self) -> Key {
        self.st().pawn_key
    }

    #[inline]
    pub fn material_key(&self) -> Key {
        self.st().material_key
    }

    #[inline]
    pub fn non_pawn_material(&self, c: Color) -> Value {
        self.st().non_pawn_material[c as usize]
    }

    #[inline]
    pub fn game_ply(&self) -> i32 {
        self.game_ply
    }

    #[inline]
    pub fn rule50_count(&self) -> i32 {
        self.st().rule_50
    }

    #[inline]
    pub fn captured_piece(&self) -> Piece {
        self.st().captured_piece
    }

    pub fn put_piece(&mut self, pc: Piece, s: Square) {
        let pt = pc.type_of();
        self.board[s as usize] = pc;
        self.by_type_bb[(PieceType::AllPieces as i32 + 1) as usize] |=
            self.by_type_bb[pc.type_of() as usize];
        self.by_type_bb[pt as usize] |= s;
        self.by_color_bb[pc.color() as usize] |= s;
        self.piece_count[pc as usize] += 1;
        self.piece_count[make_piece(pc.color(), pc.type_of()) as usize] += 1;
    }

    pub fn remove_piece(&mut self, s: Square) {
        let pc = self.board[s as usize];
        self.by_type_bb[(PieceType::AllPieces as i32 + 1) as usize] ^= s;
        self.by_type_bb[pc.type_of() as usize] ^= s;
        self.by_color_bb[pc.color() as usize] ^= s;
        self.board[s as usize] = Piece::NoPiece;
        self.piece_count[pc as usize] -= 1;
        self.piece_count[make_piece(pc.color(), PieceType::AllPieces) as usize] -= 1;
    }

    pub fn move_piece(&mut self, f: Square, t: Square) {
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
        let mut cuckoomove: [Key; 8192] = [0; 8192];
        let zpsq = zobrist::PSQ.get().unwrap();
        let zside = zobrist::SIDE.get().unwrap();
        for j in 0..8192 {
            cuckoo[j] = 0;
            cuckoomove[j] = 0;
        }
        let mut count = 0;
        for i in 0..PNB {
            for j in SQA1..=SQH8 {
                let s1 = Square::new_from_n(j as i32);
                let s2 = Square::new_from_n(s1 as i32 + 1);
                for k in s2 as usize..=SQH8 {
                    let pc = Piece::new_from_n(i);
                    if (pc.type_of() as usize != Piece::WPawn as usize
                        && pc.type_of() as usize != Piece::BPawn as usize)
                        && (bb::attacks_bb(pc.type_of(), s1, 0) & s2) != 0
                    {
                        let mv = Move::new_from_to_sq(s1, s2);
                        let mut key = zpsq[i][j] ^ zpsq[i][k] ^ zside;
                        let mut m = H1(key);
                        'inner: loop {
                            std::mem::swap(&mut cuckoo[m as usize], &mut key);
                            std::mem::swap(&mut cuckoo[m as usize], &mut key);
                            if mv == Move::none() {
                                break 'inner;
                            }
                            m = ((m == H1(key)) as i32 * H2(key))
                                + ((m != H1(key)) as i32 * H1(key));
                        }
                        count += 1;
                    }
                }
            }
        }
        assert!(count == 3668);
    }

    #[inline]
    pub fn pieces(&self, pt: PieceType) -> Bitboard {
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

    // @todo: Expand this function's testing
    #[test]
    fn test_pieces_by_color_and_piece_types_macro() {
        let mut position = Position::default();
        position.put_piece(Piece::BBishop, Square::SqA1);
        position.put_piece(Piece::WBishop, Square::SqA8);
        position.put_piece(Piece::WRook, Square::SqE4);
        let res_b =
            pieces_by_color_and_pt!(&position, Color::Black, PieceType::Bishop, PieceType::Rook);
        let res_w =
            pieces_by_color_and_pt!(&position, Color::White, PieceType::Bishop, PieceType::Rook);
        let r_w = bb::pretty(res_w);
        let r_b = bb::pretty(res_b);
        println!("{}", r_w);
        println!("{}", r_b);
    }
}
