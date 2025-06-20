use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Sub,
    SubAssign,
};
pub type Bitboard = u64;
pub type Value = i32;
pub type Key = u64;
pub type Depth = i32;

const MAX_MOVES: i32 = 256;
const MAX_PLY: i32 = 246;

const VALUE_ZERO: Value = 0;
const VALUE_DRAW: Value = 0;
const VALUE_NONE: Value = 32002;
const VALUE_INFINITE: Value = 32001;
const VALUE_MATE: Value = 32000;
const VALUE_MATE_IN_MAX_PLY: Value = VALUE_MATE - MAX_PLY;
const VALUE_MATED_IN_MAX_PLY: Value = -VALUE_MATE_IN_MAX_PLY;
const VALUE_TB: Value = VALUE_MATE_IN_MAX_PLY - 1;
const VALUE_TB_WIN_IN_MAX_PLY: Value = VALUE_TB - MAX_PLY;
const VALUE_TB_LOSS_IN_MAX_PLY: Value = -VALUE_TB_WIN_IN_MAX_PLY;

const PAWNVALUE: Value = 208;
const KNIGHTVALUE: Value = 781;
const BISHOPVALUE: Value = 825;
const ROOKVALUE: Value = 1276;
const QUEENVALUE: Value = 2538;

const PIECEVALE: [Value; Piece::PieceNb as usize] = [
    VALUE_ZERO,
    PAWNVALUE,
    KNIGHTVALUE,
    BISHOPVALUE,
    ROOKVALUE,
    QUEENVALUE,
    VALUE_ZERO,
    VALUE_ZERO,
    VALUE_ZERO,
    PAWNVALUE,
    KNIGHTVALUE,
    BISHOPVALUE,
    ROOKVALUE,
    QUEENVALUE,
    VALUE_ZERO,
    VALUE_ZERO,
];

const RANK1BB: Bitboard = 0xFF;
const FILEABB: Bitboard = 0x0101010101010101;

pub const SQNB: usize = Square::SquareNb as usize - 1; //Poissibly move these constants to the types file
pub const PNB: usize = Piece::PieceNb as usize;
pub const PTNB: usize = PieceType::PieceTypeNb as usize;
pub const COLORNB: usize = Color::ColorNb as usize;
pub const CRNB: usize = CastlingRights::CastlingRightsNb as usize;
pub const FNB: usize = 8;
pub const RNB: usize = 8;

pub const SQA1: usize = Square::SqA1 as usize;
pub const SQH8: usize = Square::SqH8 as usize;

pub const PawnValue: Value = 208;
pub const KnightValue: Value = 781;
pub const BishopValue: Value = 825;
pub const RookValue: Value = 1276;
pub const QueenValue: Value = 2538;

pub const PIECEVALUE: [Value; PNB] = [
    VALUE_ZERO,
    PawnValue,
    KnightValue,
    BishopValue,
    RookValue,
    QueenValue,
    VALUE_ZERO,
    VALUE_ZERO,
    VALUE_ZERO,
    PawnValue,
    KnightValue,
    BishopValue,
    RookValue,
    QueenValue,
    VALUE_ZERO,
    VALUE_ZERO,
];

#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum Color {
    #[default]
    White = 0,
    Black = 1,
    ColorNb = 2,
}

#[repr(u8)]
pub enum Bound {
    BoundNone = 0,
    BoundUpper,
    BoundLower,
    BoundExact = 1 | 2,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Piece {
    #[default]
    NoPiece = 0,
    WPawn,
    WKnight,
    WBishop,
    WRook,
    WQueen,
    WKing,
    BPawn = 9,
    BKnight = 10,
    BBishop = 11,
    BRook = 12,
    BQueen = 13,
    BKing = 14,
    PieceNb = 16,
}

impl Piece {
    pub const fn type_of(&self) -> PieceType {
        match self {
            Piece::WPawn => PieceType::Pawn,
            Piece::BPawn => PieceType::Pawn,
            Piece::WKnight => PieceType::Knight,
            Piece::BKnight => PieceType::Knight,
            Piece::WBishop => PieceType::Bishop,
            Piece::BBishop => PieceType::Bishop,
            Piece::WRook => PieceType::Rook,
            Piece::BRook => PieceType::Rook,
            Piece::WQueen => PieceType::Queen,
            Piece::BQueen => PieceType::Queen,
            Piece::WKing => PieceType::King,
            Piece::BKing => PieceType::King,
            Piece::NoPiece => PieceType::NoPieceType,
            _ => panic!(),
        }
    }

    pub const fn color(&self) -> Color {
        match self {
            Piece::WPawn => Color::White,
            Piece::WKnight => Color::White,
            Piece::WBishop => Color::White,
            Piece::WRook => Color::White,
            Piece::WQueen => Color::White,
            Piece::WKing => Color::White,

            Piece::BPawn => Color::Black,
            Piece::BKnight => Color::Black,
            Piece::BBishop => Color::Black,
            Piece::BRook => Color::Black,
            Piece::BQueen => Color::Black,
            Piece::BKing => Color::Black,
            _ => panic!(),
        }
    }

    pub const fn new_from_n(i: usize) -> Self {
        match i {
            0 => Piece::NoPiece,
            1 => Piece::WPawn,
            2 => Piece::WKnight,
            3 => Piece::WBishop,
            4 => Piece::WRook,
            5 => Piece::WQueen,
            6 => Piece::WKing,
            9 => Piece::BPawn,
            10 => Piece::BKnight,
            11 => Piece::BBishop,
            12 => Piece::BRook,
            13 => Piece::BQueen,
            14 => Piece::BKing,
            _ => panic!("Invalid piece index"),
        }
    }
}
impl Not for Piece {
    type Output = Self;
    fn not(self) -> Self {
        let result = self as i8 ^ 8;
        if (result >= 0 && result <= 6) || (result >= 9 && result <= 14) {
            unsafe { std::mem::transmute(result) }
        } else {
            panic!()
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    North = 8,
    East = 1,
    South = -8,
    West = -1,

    NorthWest = 8 - 1,
    NorthEast = 8 + 1,
    SouthEast = -8 + 1,
    SouthWest = -8 - 1,
}

impl Direction {
    pub fn to_num(&self) -> i32 {
        match self {
            Direction::North => 8,
            Direction::East => 1,
            Direction::South => -8,
            Direction::West => -1,
            Direction::NorthWest => 8 - 1,
            Direction::NorthEast => 8 + 1,
            Direction::SouthEast => -8 + 1,
            Direction::SouthWest => -8 - 1,
        }
    }

    pub fn from_num(n: i32) -> Self {
        match n {
            8 => Direction::North,
            1 => Direction::East,
            -8 => Direction::South,
            -1 => Direction::West,
            7 => Direction::NorthWest,
            9 => Direction::NorthEast,
            -7 => Direction::SouthEast,
            -9 => Direction::SouthWest,
            _ => panic!("Invalid Direction Nunmber"),
        }
    }
}
// Overloading Addition operator between Square and Direction
// Two Variants to allow for addition from either side
impl Add<Direction> for Square {
    type Output = Self;
    fn add(self, b: Direction) -> Self {
        let result = self as i32 + b as i32;
        if Square::is_square_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            Square::SqNone
        }
    }
}
impl Add<i32> for Square {
    type Output = Self;
    fn add(self, rhs: i32) -> Self {
        let result = self as i32 + rhs;
        if Square::is_square_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            Square::SqNone
        }
    }
}
impl Add<Square> for Direction {
    type Output = Square;
    fn add(self, rhs: Square) -> Square {
        let result = self as i32 + rhs as i32;
        if Square::is_square_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            Square::SqNone
        }
    }
}

impl Add<Direction> for Direction {
    type Output = Self;
    fn add(self, rhs: Direction) -> Self {
        let result = self as i32 + rhs as i32;
        unsafe { std::mem::transmute(result) }
    }
}

//Overloading subtraction between two directions
impl Sub<Direction> for Direction {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self {
        let result = self as i32 - rhs as i32;
        unsafe { std::mem::transmute(result) }
    }
}

impl Mul<i32> for Direction {
    type Output = i32;
    fn mul(self, rhs: i32) -> i32 {
        let val = self.to_num();
        return rhs * val;
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum File {
    FileA = 0,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank {
    Rank1 = 0,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
    RankNb,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Default, PartialOrd)]
pub enum Square {
    SqA1 = 0,
    SqB1,
    SqC1,
    SqD1,
    SqE1,
    SqF1,
    SqG1,
    SqH1,
    SqA2,
    SqB2,
    SqC2,
    SqD2,
    SqE2,
    SqF2,
    SqG2,
    SqH2,
    SqA3,
    SqB3,
    SqC3,
    SqD3,
    SqE3,
    SqF3,
    SqG3,
    SqH3,
    SqA4,
    SqBA4,
    SqC4,
    SqD4,
    SqE4,
    SqF4,
    SqG4,
    SqH4,
    SqA5,
    SqB5,
    SqC5,
    SqD5,
    SqE5,
    SqF5,
    SqG5,
    SqH5,
    SqA6,
    SqB6,
    SqC6,
    SqD6,
    SqE6,
    SqF6,
    SqG6,
    SqH6,
    SqA7,
    SqB7,
    SqC7,
    SqD7,
    SqE7,
    SqF7,
    SqG7,
    SqH7,
    SqA8,
    SqB8,
    SqC8,
    SqD8,
    SqE8,
    SqF8,
    SqG8,
    SqH8,
    SqNone,
    #[default]
    SquareNb,
    SquareZero = -1,
}

// Overloading subtraction of a direction from a square
impl Sub<Direction> for Square {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self {
        let result = self as i32 - rhs as i32;
        if result >= 0 && result <= Square::SqH8 as i32 {
            return Square::new_from_n(result);
        } else {
            Square::SqNone
        }
    }
}

//Overloading += between square and direction.
impl AddAssign<Direction> for Square {
    fn add_assign(&mut self, rhs: Direction) {
        let result = *self + rhs;
        if result as i32 >= Square::SqA1 as i32 && result as i32 <= Square::SqH8 as i32 {
            *self = *self + rhs;
        } else {
            *self = Square::SqNone;
        }
    }
}

//Overloading -= between square and direction
impl SubAssign<Direction> for Square {
    fn sub_assign(&mut self, rhs: Direction) {
        *self = *self - rhs;
    }
}

impl BitAnd<Bitboard> for Square {
    type Output = Bitboard;
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        rhs & self
    }
}

impl BitOr<Bitboard> for Square {
    type Output = Bitboard;
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        rhs | self
    }
}

impl Square {
    pub const fn new_from_n(n: i32) -> Self {
        if Square::is_square_valid(n) {
            unsafe { std::mem::transmute(n) }
        } else {
            Self::SqNone
        }
    }
    pub const fn flip_rank(&self) -> Self {
        let result = *self as i32 ^ Square::SqA8 as i32;
        if Self::is_square_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            panic!()
        }
    }

    pub const fn flip_file(&self) -> Self {
        let result = *self as i32 ^ Square::SqH1 as i32;
        if Self::is_square_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            panic!()
        }
    }

    pub const fn file_of(&self) -> File {
        let result = *self as i32 & 7;
        if is_file_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            panic!()
        }
    }

    pub const fn rank_of(&self) -> Rank {
        let result = *self as i32 >> 3;
        if is_rank_valid(result) {
            unsafe { std::mem::transmute(result) }
        } else {
            panic!()
        }
    }

    pub const fn relative_rank(&self, c: Color) -> Rank {
        relative_rank(c, self.rank_of())
    }

    pub const fn bb(&self) -> Bitboard {
        1 << *self as i32
    }

    pub const fn rank_bb(&self) -> Bitboard {
        RANK1BB << 8 * (self.rank_of() as i32)
    }

    pub const fn file_bb(&self) -> Bitboard {
        FILEABB << self.file_of() as i32
    }

    pub const fn rank_distance_from(&self, s: Square) -> i32 {
        let result = self.rank_of() as i32 - s.rank_of() as i32;
        let r = result.abs();
        r
    }

    pub const fn file_distance_from(&self, s: Square) -> i32 {
        let result = self.file_of() as i32 - s.file_of() as i32;
        let r = result.abs();
        r
    }

    pub const fn is_square_valid(square: i32) -> bool {
        return square >= Square::SqA1 as i32 && square <= Square::SqH8 as i32;
    }

    pub const fn relative_square(&self, c: Color) -> Square {
        let k = *self as i32 ^ (c as i32 * 56);
        return Square::new_from_n(k);
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PieceType {
    NoPieceType = -1,
    AllPieces,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    PieceTypeNb = 8,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MoveType {
    Normal,
    Promotion = 1 << 14,
    EnPassant = 2 << 14,
    Castling = 3 << 14,
}

//Overloading addition between two directions

//Overloading Not for Colors
impl Not for Color {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
            Color::ColorNb => Color::ColorNb,
        }
    }
}

impl BitAnd<Square> for Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: Square) -> Bitboard {
        self & rhs.bb()
    }
}

impl BitOr<Square> for Bitboard {
    type Output = Bitboard;
    fn bitor(self, rhs: Square) -> Bitboard {
        self | rhs.bb()
    }
}
impl BitXor<Square> for Bitboard {
    type Output = Bitboard;
    fn bitxor(self, rhs: Square) -> Bitboard {
        self ^ rhs.bb()
    }
}
impl BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        let result = *self | rhs.bb();
        *self = result;
    }
}
impl BitXorAssign<Square> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Square) {
        let result = *self ^ rhs.bb();
        *self = result;
    }
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum CastlingRights {
    NoCastling = 0,
    WhiteOO = 1,
    WhiteOOO = 1 << 1,
    BlackOO = 1 << 2,
    BlackOOO = 1 << 3,

    KingSide = 1 | (1 << 2),
    QueenSide = (1 << 1) | (1 << 3),
    WhiteCastling = 1 | (1 << 1),
    BlackCastling = (1 << 2) | (1 << 3),
    #[default]
    AnyCastling = 1 | (1 << 1) | (1 << 2) | (1 << 3),

    CastlingRightsNb = 16,
}

impl CastlingRights {
    pub fn new_from_n(n: i32) -> Self {
        match n {
            0 => Self::NoCastling,
            1 => Self::WhiteOO,
            2 => Self::WhiteOOO,
            4 => Self::BlackOO,
            8 => Self::BlackOOO,
            5 => Self::KingSide,
            10 => Self::QueenSide,
            3 => Self::WhiteCastling,
            12 => Self::BlackCastling,
            15 => Self::AnyCastling,
            16 => Self::CastlingRightsNb,
            _ => panic!(
                "Cannot create castling rights from {} Invalid Castling Rights number",
                n
            ),
        }
    }
}
//Overload BitAnd Between Color and Castling Rights
impl BitAnd<Color> for CastlingRights {
    type Output = Self;
    fn bitand(self, rhs: Color) -> CastlingRights {
        match rhs {
            Color::White => {
                CastlingRights::new_from_n(CastlingRights::WhiteCastling as i32 & self as i32)
            }
            Color::Black => {
                CastlingRights::new_from_n(CastlingRights::BlackCastling as i32 & self as i32)
            }
            _ => panic!(),
        }
    }
}

impl BitOr for CastlingRights {
    type Output = Self;
    fn bitor(self, rhs: Self) -> CastlingRights {
        let nw = self as i32 | rhs as i32;
        return CastlingRights::new_from_n(nw);
    }
}

impl BitAndAssign for CastlingRights {
    fn bitand_assign(&mut self, rhs: Self) {
        let nw = *self as i32 & rhs as i32;
        let res = CastlingRights::new_from_n(nw);
        *self = res
    }
}

impl Not for CastlingRights {
    type Output = Self;
    fn not(self) -> CastlingRights {
        let nw = self as i32;
        return CastlingRights::new_from_n(nw);
    }
}

impl BitAnd for CastlingRights {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        let res = self as i32 & rhs as i32;
        return CastlingRights::new_from_n(res);
    }
}

impl BitOrAssign<CastlingRights> for CastlingRights {
    fn bitor_assign(&mut self, rhs: CastlingRights) {
        let res = *self as i32 | rhs as i32;
        let nw: CastlingRights = CastlingRights::new_from_n(res);
        *self = nw;
    }
}
//Overload putting CastlingRights on rhs
impl BitAnd<CastlingRights> for Color {
    type Output = CastlingRights;
    fn bitand(self, rhs: CastlingRights) -> CastlingRights {
        match self {
            Color::White => {
                CastlingRights::new_from_n(CastlingRights::WhiteCastling as i32 & rhs as i32)
            }
            Color::Black => {
                CastlingRights::new_from_n(CastlingRights::BlackCastling as i32 & rhs as i32)
            }
            _ => panic!(),
        }
    }
}

pub const fn mate_in(ply: i32) -> Value {
    VALUE_MATE - ply
}
pub const fn mated_in(ply: i32) -> Value {
    -VALUE_MATE + ply
}

pub const fn make_square(f: usize, r: usize) -> Square {
    let result = ((r as i32) << 3) + f as i32;
    if Square::is_square_valid(result) {
        unsafe { std::mem::transmute(result) }
    } else {
        panic!()
    }
}

pub const fn make_piece(c: Color, pt: PieceType) -> Piece {
    match c {
        Color::White => make_white_piece(pt),
        Color::Black => make_black_piece(pt),
        Color::ColorNb => panic!(),
    }
}

pub const fn make_white_piece(pt: PieceType) -> Piece {
    match pt {
        PieceType::Pawn => Piece::WPawn,
        PieceType::Knight => Piece::WKnight,
        PieceType::Bishop => Piece::WBishop,
        PieceType::Rook => Piece::WRook,
        PieceType::Queen => Piece::WQueen,
        PieceType::King => Piece::WKing,
        _ => panic!(),
    }
}

pub const fn make_black_piece(pt: PieceType) -> Piece {
    match pt {
        PieceType::Pawn => Piece::BPawn,
        PieceType::Knight => Piece::BKnight,
        PieceType::Bishop => Piece::BBishop,
        PieceType::Rook => Piece::BRook,
        PieceType::Queen => Piece::BQueen,
        PieceType::King => Piece::BKing,
        _ => panic!(),
    }
}

pub const fn is_valid_move_type(data: u16) -> bool {
    data == 0 || data == (1 << 14) || data == (2 << 14) || data == (3 << 14)
}

//Check if the i32 value falls within fhe file values
pub const fn is_file_valid(s: i32) -> bool {
    s >= File::FileA as i32 && s <= File::FileH as i32
}

//Check if the rank falls within the valid rank values
pub const fn is_rank_valid(s: i32) -> bool {
    s >= Rank::Rank1 as i32 && s <= Rank::Rank8 as i32
}

pub const fn pawn_push(color: Color) -> Direction {
    match color {
        Color::White => Direction::North,
        Color::Black => Direction::South,
        _ => panic!("Pawn Push Called with an invalid Color"),
    }
}

pub const fn make_key(seed: u64) -> Key {
    return (seed * 6364136223846793005 + 1442695040888963407) as Key;
}

pub const fn relative_rank(color: Color, rank: Rank) -> Rank {
    let result = (rank as i32) ^ (color as i32 * 7);
    if is_rank_valid(result) {
        unsafe { std::mem::transmute(result) }
    } else {
        panic!()
    }
}

pub const fn relative_rank_of_square(c: Color, s: Square) -> Rank {
    return relative_rank(c, s.rank_of());
}

// A move needs 16 bits
// bits 0-5: destination square
// 6-11 origin square
// 12-13 promotion piece type - 2 [Knight - 2, Queen - 2]
// 14-15 special move flag: promotion(1), en_passant(2), castling(3)
// en_passant bit is only set if a pawn can be captured
// Special cases are move::none() and move::null()
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Move {
    data: u16,
}

impl Move {
    pub const fn new(data: u16) -> Self {
        Move { data: data }
    }

    pub const fn new_from_to_sq(from: Square, to: Square) -> Self {
        let data = ((from as usize) << 6) + to as usize;
        Move { data: data as u16 }
    }
    pub const fn from_to(&self) -> u16 {
        self.data & 0xFFF
    }

    pub const fn from_sq(&self) -> Square {
        let result = (self.data >> 6) & 0x3F;
        if Square::is_square_valid(result as i32) {
            unsafe { std::mem::transmute(result as i32) }
        } else {
            panic!()
        }
    }

    pub const fn to_sq(&self) -> Square {
        let result = self.data & 0x3F;
        if Square::is_square_valid(result as i32) {
            unsafe { std::mem::transmute(result as i32) }
        } else {
            panic!()
        }
    }

    pub const fn type_of(&self) -> MoveType {
        let result = (3 << 14) & self.data;
        let promotion = 1 << 14;
        let enPassant = 2 << 14;
        let castling = 3 << 14;
        match result {
            0 => MoveType::Normal,
            promotion => MoveType::Promotion,
            enpassant => MoveType::EnPassant,
            castling => MoveType::Castling,
            _ => panic!(),
        }
    }

    pub const fn promotion_type(&self) -> PieceType {
        let res = ((self.data >> 12) & 3) as i32;
        match res {
            0 => PieceType::Knight,
            1 => PieceType::Bishop,
            2 => PieceType::Rook,
            3 => PieceType::Queen,
            _ => panic!(),
        }
    }

    pub const fn raw(&self) -> u16 {
        self.data as u16
    }

    pub const fn is_nonzero(&self) -> bool {
        self.data != 0
    }

    pub const fn null() -> Self {
        Move { data: 0 }
    }

    pub const fn none() -> Self {
        Move { data: 65 }
    }

    pub const fn is_ok(&self) -> bool {
        self.data != Self::none().data && self.data != Self::null().data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_add_direction() {
        assert_eq!(Square::SqA1 + Direction::North, Square::SqA2);
        //Recheck this to see if that is defined behavior
        // assert_eq!(Square::SqH1 + Direction::NorthEast, Square::SqNone);
    }

    #[test]
    fn test_square_sub_direction() {
        assert_eq!(Square::SqA2 - Direction::North, Square::SqA1);
        assert_eq!(Square::SqA1 - Direction::South, Square::SqA2);
    }

    #[test]
    fn test_square_add_assign_direction() {
        let mut square = Square::SqA1;
        square += Direction::North;
        assert_eq!(square, Square::SqA2);
    }

    #[test]
    fn test_square_sub_assign_direction() {
        let mut square = Square::SqA2;
        square -= Direction::North;
        assert_eq!(square, Square::SqA1);
    }

    #[test]
    fn test_direction_add_direction() {
        assert_eq!(Direction::North + Direction::East, Direction::NorthEast);
        assert_eq!(Direction::South + Direction::West, Direction::SouthWest);
    }

    #[test]
    fn test_direction_sub_direction() {
        assert_eq!(Direction::North - Direction::East, Direction::NorthWest);
        assert_eq!(Direction::South - Direction::West, Direction::SouthEast);
    }

    #[test]
    fn test_color_not() {
        assert_eq!(!Color::White, Color::Black);
        assert_eq!(!Color::Black, Color::White);
    }

    #[test]
    fn test_piece_not() {
        assert_eq!(!Piece::WPawn, Piece::BPawn);
        assert_eq!(!Piece::BQueen, Piece::WQueen);
    }

    #[test]
    fn test_bitboard_operations() {
        let bitboard: Bitboard = 0xFF;
        assert_eq!(bitboard & Square::SqA1, 0x01);
        assert_eq!(bitboard | Square::SqA1, 0xFF);
        assert_eq!(bitboard ^ Square::SqA1, 0xFE);
    }

    #[test]
    fn test_make_square() {
        assert_eq!(
            make_square(File::FileA as usize, Rank::Rank1 as usize),
            Square::SqA1
        );
        assert_eq!(
            make_square(File::FileH as usize, Rank::Rank8 as usize),
            Square::SqH8
        );
    }

    #[test]
    fn test_make_piece() {
        assert_eq!(make_piece(Color::White, PieceType::Queen), Piece::WQueen);
        assert_eq!(make_piece(Color::Black, PieceType::Knight), Piece::BKnight);
    }

    #[test]
    fn test_move_creation() {
        let mv = Move::new(0b110110011011);
        assert_eq!(mv.raw(), 0b110110011011);
        assert_eq!(mv.from_sq(), Square::new_from_n(0b110110));
        assert_eq!(mv.to_sq(), Square::new_from_n(0b011011));
    }

    #[test]
    fn test_move_type() {
        let mv = Move::new(11 << 14);
        assert_eq!(mv.type_of(), MoveType::Promotion);
    }

    #[test]
    fn test_move_promotion_type() {
        let mv = Move::new(0x3000);
        assert_eq!(mv.promotion_type(), PieceType::Queen);
    }

    #[test]
    fn test_move_is_ok() {
        let mv = Move::new(0x1234);
        assert!(mv.is_ok());
        assert!(!Move::null().is_ok());
        assert!(!Move::none().is_ok());
    }
    #[test]
    fn test_square_flip_rank() {
        assert_eq!(Square::SqA1.flip_rank(), Square::SqA8);
        assert_eq!(Square::SqH1.flip_rank(), Square::SqH8);
    }

    #[test]
    fn test_square_flip_file() {
        assert_eq!(Square::SqA1.flip_file(), Square::SqH1);
        assert_eq!(Square::SqA8.flip_file(), Square::SqH8);
    }

    #[test]
    fn test_square_file_of() {
        assert_eq!(Square::SqA1.file_of(), File::FileA);
        assert_eq!(Square::SqH8.file_of(), File::FileH);
    }

    #[test]
    fn test_square_rank_of() {
        assert_eq!(Square::SqA1.rank_of(), Rank::Rank1);
        assert_eq!(Square::SqH8.rank_of(), Rank::Rank8);
    }

    #[test]
    fn test_square_relative_rank() {
        assert_eq!(Square::SqA1.relative_rank(Color::White), Rank::Rank1);
        assert_eq!(Square::SqA1.relative_rank(Color::Black), Rank::Rank8);
    }

    #[test]
    fn test_square_bb() {
        assert_eq!(Square::SqA1.bb(), 1);
        assert_eq!(Square::SqH8.bb(), 1 << 63);
    }

    #[test]
    fn test_square_rank_bb() {
        assert_eq!(Square::SqA1.rank_bb(), 0xFF);
        assert_eq!(Square::SqA8.rank_bb(), 0xFF << 56);
    }

    #[test]
    fn test_square_file_bb() {
        assert_eq!(Square::SqA1.file_bb(), 0x0101010101010101);
        assert_eq!(Square::SqH1.file_bb(), 0x8080808080808080);
    }

    #[test]
    fn test_square_rank_distance_from() {
        assert_eq!(Square::SqA1.rank_distance_from(Square::SqA2), 1);
        assert_eq!(Square::SqA1.rank_distance_from(Square::SqA8), 7);
    }

    #[test]
    fn test_square_file_distance_from() {
        assert_eq!(Square::SqA1.file_distance_from(Square::SqB1), 1);
        assert_eq!(Square::SqA1.file_distance_from(Square::SqH1), 7);
    }

    #[test]
    fn test_castling_rights_bitand_color() {
        assert_eq!(
            CastlingRights::AnyCastling & Color::White,
            CastlingRights::WhiteCastling
        );
        assert_eq!(
            CastlingRights::AnyCastling & Color::Black,
            CastlingRights::BlackCastling
        );
    }

    #[test]
    fn test_color_bitand_castling_rights() {
        assert_eq!(
            Color::White & CastlingRights::AnyCastling,
            CastlingRights::WhiteCastling
        );
        assert_eq!(
            Color::Black & CastlingRights::AnyCastling,
            CastlingRights::BlackCastling
        );
    }

    #[test]
    fn test_mate_in() {
        assert_eq!(mate_in(1), VALUE_MATE - 1);
        assert_eq!(mate_in(MAX_PLY), VALUE_MATE - MAX_PLY);
    }

    #[test]
    fn test_mated_in() {
        assert_eq!(mated_in(1), -VALUE_MATE + 1);
        assert_eq!(mated_in(MAX_PLY), -VALUE_MATE + MAX_PLY);
    }

    #[test]
    fn test_make_key() {
        assert_eq!(make_key(0), 1442695040888963407);
        assert_eq!(make_key(1), 7806831264735756412);
    }

    #[test]
    fn test_pawn_push() {
        assert_eq!(pawn_push(Color::White), Direction::North);
        assert_eq!(pawn_push(Color::Black), Direction::South);
    }

    #[test]
    fn test_piece_type_of() {
        assert_eq!(Piece::WPawn.type_of(), PieceType::Pawn);
        assert_eq!(Piece::WQueen.type_of(), PieceType::Queen);
        assert_eq!(Piece::BKing.type_of(), PieceType::King);
    }

    #[test]
    fn test_piece_color() {
        assert_eq!(Piece::WPawn.color(), Color::White);
        assert_eq!(Piece::BQueen.color(), Color::Black);
    }

    #[test]
    fn test_square_new_from_n() {
        assert_eq!(Square::new_from_n(0), Square::SqA1);
        assert_eq!(Square::new_from_n(1), Square::SqB1);
        assert_eq!(Square::new_from_n(63), Square::SqH8);
        assert_eq!(Square::new_from_n(64), Square::SqNone);
    }

    #[test]
    fn test_square_is_square_valid() {
        assert!(Square::is_square_valid(0));
        assert!(Square::is_square_valid(63));
        assert!(!Square::is_square_valid(64));
    }

    #[test]
    fn test_is_valid_move_type() {
        assert!(is_valid_move_type(0));
        assert!(is_valid_move_type(1 << 14));
        assert!(is_valid_move_type(2 << 14));
        assert!(is_valid_move_type(3 << 14));
        assert!(!is_valid_move_type(4 << 9));
    }

    #[test]
    fn test_is_file_valid() {
        assert!(is_file_valid(File::FileA as i32));
        assert!(is_file_valid(File::FileH as i32));
        assert!(!is_file_valid(File::FileH as i32 + 1));
    }

    #[test]
    fn test_is_rank_valid() {
        assert!(is_rank_valid(Rank::Rank1 as i32));
        assert!(is_rank_valid(Rank::Rank8 as i32));
        assert!(!is_rank_valid(Rank::Rank8 as i32 + 1));
    }

    #[test]
    fn test_make_white_piece() {
        assert_eq!(make_white_piece(PieceType::Pawn), Piece::WPawn);
        assert_eq!(make_white_piece(PieceType::Knight), Piece::WKnight);
    }

    #[test]
    fn test_make_black_piece() {
        assert_eq!(make_black_piece(PieceType::Pawn), Piece::BPawn);
        assert_eq!(make_black_piece(PieceType::Knight), Piece::BKnight);
    }
    #[test]
    fn test_square_add_direction_overflow() {
        assert_eq!(Square::SqH8 + Direction::North, Square::SqNone);
        assert_eq!(Square::SqA1 + Direction::South, Square::SqNone);
    }

    #[test]
    fn test_square_sub_direction_underflow() {
        assert_eq!(Square::SqA1 - Direction::North, Square::SqNone);
        assert_eq!(Square::SqH8 - Direction::South, Square::SqNone);
    }

    #[test]
    fn test_square_add_assign_direction_overflow() {
        let mut square = Square::SqH8;
        square += Direction::North;
        assert_eq!(square, Square::SqNone);
    }

    #[test]
    fn test_square_sub_assign_direction_underflow() {
        let mut square = Square::SqA1;
        square -= Direction::North;
        assert_eq!(square, Square::SqNone);
    }

    #[test]
    fn test_piece_type_values() {
        assert_eq!(PieceType::Pawn as i32, 1);
        assert_eq!(PieceType::Knight as i32, 2);
        assert_eq!(PieceType::Bishop as i32, 3);
        assert_eq!(PieceType::Rook as i32, 4);
        assert_eq!(PieceType::Queen as i32, 5);
        assert_eq!(PieceType::King as i32, 6);
    }

    #[test]
    fn test_piece_values() {
        assert_eq!(Piece::WPawn as i32, 1);
        assert_eq!(Piece::WKnight as i32, 2);
        assert_eq!(Piece::WBishop as i32, 3);
        assert_eq!(Piece::WRook as i32, 4);
        assert_eq!(Piece::WQueen as i32, 5);
        assert_eq!(Piece::WKing as i32, 6);
        assert_eq!(Piece::BPawn as i32, 9);
        assert_eq!(Piece::BKnight as i32, 10);
        assert_eq!(Piece::BBishop as i32, 11);
        assert_eq!(Piece::BRook as i32, 12);
        assert_eq!(Piece::BQueen as i32, 13);
        assert_eq!(Piece::BKing as i32, 14);
    }

    #[test]
    fn test_direction_values() {
        assert_eq!(Direction::North as i32, 8);
        assert_eq!(Direction::East as i32, 1);
        assert_eq!(Direction::South as i32, -8);
        assert_eq!(Direction::West as i32, -1);
        assert_eq!(Direction::NorthWest as i32, 7);
        assert_eq!(Direction::NorthEast as i32, 9);
        assert_eq!(Direction::SouthEast as i32, -7);
        assert_eq!(Direction::SouthWest as i32, -9);
    }

    #[test]
    fn test_file_values() {
        assert_eq!(File::FileA as i32, 0);
        assert_eq!(File::FileB as i32, 1);
        assert_eq!(File::FileC as i32, 2);
        assert_eq!(File::FileD as i32, 3);
        assert_eq!(File::FileE as i32, 4);
        assert_eq!(File::FileF as i32, 5);
        assert_eq!(File::FileG as i32, 6);
        assert_eq!(File::FileH as i32, 7);
    }

    #[test]
    fn test_rank_values() {
        assert_eq!(Rank::Rank1 as i32, 0);
        assert_eq!(Rank::Rank2 as i32, 1);
        assert_eq!(Rank::Rank3 as i32, 2);
        assert_eq!(Rank::Rank4 as i32, 3);
        assert_eq!(Rank::Rank5 as i32, 4);
        assert_eq!(Rank::Rank6 as i32, 5);
        assert_eq!(Rank::Rank7 as i32, 6);
        assert_eq!(Rank::Rank8 as i32, 7);
    }

    #[test]
    fn test_square_values() {
        assert_eq!(Square::SqA1 as i32, 0);
        assert_eq!(Square::SqH8 as i32, 63);
        assert_eq!(Square::SqNone as i32, 64);
    }

    #[test]
    fn test_move_type_values() {
        assert_eq!(MoveType::Normal as i32, 0);
        assert_eq!(MoveType::Promotion as i32, 1 << 14);
        assert_eq!(MoveType::EnPassant as i32, 2 << 14);
        assert_eq!(MoveType::Castling as i32, 3 << 14);
    }

    #[test]
    fn test_castling_rights_values() {
        assert_eq!(CastlingRights::NoCastling as i32, 0);
        assert_eq!(CastlingRights::WhiteOO as i32, 1);
        assert_eq!(CastlingRights::WhiteOOO as i32, 2);
        assert_eq!(CastlingRights::BlackOO as i32, 4);
        assert_eq!(CastlingRights::BlackOOO as i32, 8);
        assert_eq!(CastlingRights::KingSide as i32, 5);
        assert_eq!(CastlingRights::QueenSide as i32, 10);
        assert_eq!(CastlingRights::WhiteCastling as i32, 3);
        assert_eq!(CastlingRights::BlackCastling as i32, 12);
        assert_eq!(CastlingRights::AnyCastling as i32, 15);
    }

    #[test]
    fn test_bound_values() {
        assert_eq!(Bound::BoundNone as i32, 0);
        assert_eq!(Bound::BoundUpper as i32, 1);
        assert_eq!(Bound::BoundLower as i32, 2);
        assert_eq!(Bound::BoundExact as i32, 3);
    }
}
