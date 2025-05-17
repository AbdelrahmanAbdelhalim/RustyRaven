use crate::board::bitboard as bb;
use crate::board::zobrist;
use crate::misc::*;
use crate::types::*;
use std::fmt;
use std::sync::OnceLock;
use std::vec::Vec;

const PIECE_TYPE_NB: usize = PieceType::PieceTypeNb as usize;
const PIECE_TO_CHAR: &str = " PNBRQK pnbrqk";
const MAX_PLY: usize = 246; // Maximum search depth

pub static CUCKOO: OnceLock<[Key; 8192]> = OnceLock::new();
pub static CUCKOO_MOVE: OnceLock<[Key; 8192]> = OnceLock::new();

#[inline]
fn H1(h: Key) -> i32 {
    (h & 0x1fff) as i32
}
#[inline]
fn H2(h: Key) -> i32 {
    ((h >> 16) & 0x1fff) as i32
}

#[derive(Debug, Copy, Clone)]
struct StateInfo {
    //Copied when making a move
    material_key: Key,
    pawn_key: Key,
    non_pawn_material: [Value; COLORNB],
    castling_rights: i32,
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

struct StatePool {
    states: Vec<StateInfo>,
    current: usize,
}

impl StatePool {
    fn new() -> Self {
        Self {
            states: vec![StateInfo {
                material_key: 0,
                pawn_key: 0,
                non_pawn_material: [0; COLORNB],
                castling_rights: 0,
                rule_50: 0,
                plies_from_null: 0,
                ep_square: Square::SqNone,
                key: 0,
                checkers_bb: 0,
                blockers_for_king: [0; COLORNB],
                pinners: [0; COLORNB],
                check_squares: [0; PIECE_TYPE_NB],
                captured_piece: Piece::NoPiece,
                repition: 0,
            }; MAX_PLY],
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
    state_pool: StatePool,
    game_ply: i32,
    side_to_move: Color,
}

impl Position {
    // pub const fn default() -> Self {
    //     // let prng = Prng::new(1070372);

    // }
    #[inline(always)]
    fn st(&self) -> &StateInfo {
        self.state_pool.current()
    }

    #[inline(always)]
    fn st_mut(&mut self) -> &mut StateInfo {
        self.state_pool.current_mut()
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
    pub fn ep_square(&self) -> Square {
        self.st().ep_square
    }

    #[inline]
    pub fn can_castle(&self, cr: CastlingRights) -> bool {
        self.st().castling_rights & cr as i32 != 0
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
        self.board[s as usize] = pc;
        self.by_type_bb[(PieceType::AllPieces as i32 + 1) as usize] |=
            self.by_type_bb[pc.type_of() as usize];
        self.by_type_bb[(PieceType::AllPieces as i32 + 1) as usize] |= s;
        self.by_color_bb[pc.color() as usize] |= s;
        self.piece_count[pc as usize] += 1;
        self.piece_count[make_piece(pc.color(), PieceType::AllPieces) as usize] += 1;
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
        let psq = zobrist::PSQ.get().unwrap();
        let enpassant = zobrist::ENPASSANT.get().unwrap();
        let castling = zobrist::CASTLING.get().unwrap();
        let side = zobrist::SIDE.get().unwrap();
        let no_pawns = zobrist::NOPAWNS.get().unwrap();

        let mut cuckoo: [Key; 8192] = [0; 8192];
        let mut cuckoomove: [Key; 8192] = [0; 8192];
        let zpsq = zobrist::PSQ.get().unwrap();
        let zside =  zobrist::SIDE.get().unwrap();
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
                                break 'inner
                            }
                            m = ((m == H1(key)) as i32 * H2(key)) + ((m != H1(key)) as i32 * H1(key));
                        }
                        count += 1;
                    }
                }
            }
        }
        assert!(count == 3668);
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
    fn test_position_display() {}
}
