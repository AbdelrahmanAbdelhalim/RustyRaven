use std::fmt;
use std::sync::OnceLock;
use crate::types::*;
use crate::board::bitboard as bb;
use crate::misc::*;
use crate::board::zobrist;

const PIECE_TYPE_NB: usize = PieceType::PieceTypeNb as usize;
const PIECE_TO_CHAR: &str = " PNBRQK pnbrqk";

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
struct StateInfo<'a> {
    //Copied when making a move
    materialKey: Key,
    pawnKey: Key,
    nonPawnMaterial: [Value; COLORNB],
    castlingRights: i32,
    rule50: i32,
    pliesFromNull: i32,
    epSquare: Square,

    //Not copied when making a move
    key: Key,
    checkersBB: Bitboard,
    previous: &'a StateInfo<'a>,
    blockersForKing: [Bitboard; COLORNB],
    pinners: [Bitboard; COLORNB],
    checkSquares: [Bitboard; PIECE_TYPE_NB],
    capturedPiece: Piece,
    repition: i32,
}

struct Position<'a> {
    board: [Piece; SQNB],
    byTypeBB: [Bitboard; PTNB],
    byColorBB: [Bitboard; COLORNB],
    pieceCount: [i32; PNB],
    castlingRIghtMast: [i32; SQNB],
    castlingRookSquare: [Square; CRNB],
    castlingPath: [Bitboard; CRNB],
    st: &'a StateInfo<'a>,
    gamePly: i32,
    sideToMove: Color,
}

impl <'a> Position<'a> {
    // pub const fn default() -> Self {
    //     // let prng = Prng::new(1070372); 

    // }
    #[inline]
    pub const fn side_to_move(&self) -> Color {
        self.sideToMove        
    }

    #[inline]
    pub const fn piece_on(&self, s: Square) -> Piece {
        self.board[s as usize]
    } 

    #[inline]
    pub fn empty(&self, s: Square) -> bool {
        self.board[s as usize] == Piece::NoPiece
    }

    #[inline]
    pub const fn moved_piece(&self, m: Move) -> Piece {
        self.piece_on(m.from_sq())
    }

    #[inline]
    pub const fn pieces_by_piecetype(&self, pt: PieceType) -> Bitboard {
        self.byTypeBB[pt as usize]
    }

    #[inline]
    pub const fn ep_square(&self) -> Square {
        self.st.epSquare
    }

    #[inline]
    pub const fn can_castle(&self, cr: CastlingRights) -> bool {
        self.st.castlingRights & cr as i32 != 0
    }

    #[inline]
    pub const fn cherckers(&self) -> Bitboard {
        self.st.checkersBB
    }

    #[inline]
    pub const fn blockers_for_king(&self, c: Color) -> Bitboard {
        self.st.blockersForKing[c as usize]
    }

    #[inline]
    pub const fn pinners(&self, c: Color) -> Bitboard {
        self.st.pinners[c as usize]
    }

    #[inline]
    pub const fn check_squares(&self, pt: PieceType) -> Bitboard {
        self.st.checkSquares[pt as usize]
    }

    #[inline]
    pub const fn pawn_key(&self) -> Key {
        self.st.pawnKey
    }

    #[inline]
    pub const fn material_key(&self) -> Key {
        self.st.materialKey
    }

    #[inline]
    pub const fn non_pawn_material(&self, c: Color) -> Value {
        self.st.nonPawnMaterial[c as usize]
    }

    #[inline]
    pub const fn game_ply(&self) -> i32 {
        self.gamePly
    }

    #[inline]
    pub const fn rule50_count(&self) -> i32 {
        self.st.rule50
    }

    #[inline]
    pub const fn captured_piece(&self) -> Piece {
        self.st.capturedPiece
    }

    pub fn put_piece(&mut self, pc: Piece, s: Square) {
        self.board[s as usize] = pc;
        self.byTypeBB[(PieceType::AllPieces as i32 + 1) as usize] |= self.byTypeBB[pc.type_of() as usize];
        self.byTypeBB[(PieceType::AllPieces as i32 + 1) as usize] |= s ;
        self.byColorBB[pc.color() as usize] |= s;
        self.pieceCount[pc as usize] += 1;
        self.pieceCount[make_piece(pc.color(), PieceType::AllPieces)as usize] += 1;
    }


    pub fn remove_piece(&mut self, s: Square) {
        let pc = self.board[s as usize];
        self.byTypeBB[(PieceType::AllPieces as i32 + 1) as usize] ^= s ;
        self.byTypeBB[pc.type_of() as usize] ^= s ;
        self.byColorBB[pc.color() as usize] ^= s;
        self.board[s as usize] = Piece::NoPiece;
        self.pieceCount[pc as usize] -= 1;
        self.pieceCount[make_piece(pc.color(), PieceType::AllPieces)as usize] -= 1;
    }

    pub fn move_piece(&mut self, f: Square, t: Square) {
        let pc = self.board[f as usize];
        let from_to: Bitboard = f.bb() | t;
        self.byTypeBB[(PieceType::AllPieces as i32 + 1) as usize] ^= from_to;
        self.byTypeBB[pc.type_of() as usize] ^= from_to;
        self.byTypeBB[pc.color() as usize] ^= from_to;
        self.board[f as usize] = Piece::NoPiece;
        self.board[t as usize] = pc;
    }

    pub fn init() -> Self {
        zobrist::init_zobrist();

        let psq = zobrist::PSQ.get().unwrap();
        let enpassant = zobrist::ENPASSANT.get().unwrap();
        let castling = zobrist::CASTLING.get().unwrap();
        let side = zobrist::SIDE.get().unwrap();
        let no_pawns = zobrist::NOPAWNS.get().unwrap();

        let mut cuckoo:[Key; 8192] = [0; 8192];
        let mut cuckoomove: [Key; 8192] = [0; 8192];



    }
    //Present in the header file, and calls the overloaded function in the .cpp file.
    // #[inline]
    // pub fn do_moe(m: Move, new_st: &StateInfo) {

    // }
}

impl<'a> fmt::Display for Position<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,"\n +---+---+---+---+---+---+---+---+")?;
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

    }
}
