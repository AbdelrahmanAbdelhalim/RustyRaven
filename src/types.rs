use std::ops::{Add, AddAssign, Sub, SubAssign, Not, BitAnd, BitOr, BitXor, BitOrAssign, BitXorAssign};
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
VALUE_ZERO, PAWNVALUE, KNIGHTVALUE, BISHOPVALUE, ROOKVALUE, QUEENVALUE, VALUE_ZERO, VALUE_ZERO, 
VALUE_ZERO, PAWNVALUE, KNIGHTVALUE, BISHOPVALUE, ROOKVALUE, QUEENVALUE, VALUE_ZERO, VALUE_ZERO ];

const RANK1BB: Bitboard = 0xFF;
const FILEABB: Bitboard = 0x0101010101010101;

#[repr(i32)]
pub enum Color {
    White = 0,
    Black = 1,
    ColorNb = 2,
}

#[repr(u8)]
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
    AnyCastling = 1 | (1 << 1) | (1 << 2) | (1 << 3),

    CastlingRightsNb = 16,
}

#[repr(u8)]
pub enum Bound {
    BoundNone = 0,
    BoundUpper,
    BoundLower,
    BoundExact = 1 | 2,
}

#[repr(u8)]
pub enum Piece {
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

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
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

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub enum Square {
    SqA1 = 0, SqB1, SqC1, SqD1, SqE1, SqF1, SqG1, SqH1,
    SqA2, SqB2, SqC2, SqD2, SqE2, SqF2, SqG2, SqH2,
    SqA3, SqB3, SqC3, SqD3, SqE3, SqF3, SqG3, SqH3,
    SqA4, SqBA4, SqC4, SqD4, SqE4, SqF4, SqG4, SqH4,
    SqA5, SqB5, SqC5, SqD5, SqE5, SqF5, SqG5, SqH5,
    SqA6, SqB6, SqC6, SqD6, SqE6, SqF6, SqG6, SqH6, 
    SqA7, SqB7, SqC7, SqD7, SqE7, SqF7, SqG7, SqH7,
    SqA8, SqB8, SqC8, SqD8, SqE8, SqF8, SqG8, SqH8,
    SqNone, SquareNb,
    SquareZero = -1,
}

#[derive(PartialEq)]
pub enum PieceType {
    AllPieces = -1,
    NoPieceType,
    Pawn, Knight, Bishop, Rook, Queen, King,
    PieceTypeNb = 8,
}

#[repr(i32)]
pub enum MoveType {
    Normal,
    Promotion = 1 << 14,
    EnPassant = 2 << 14,
    Castling = 3 << 14,
}

impl Piece {
    const fn type_of(&self) -> PieceType {
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
            _ => panic!()
        }
    }

    const fn color(&self) -> Color {
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

            _ => panic!()
        }
    }
}

// Overloading Addition operator between Square and Direction
// Two Variants to allow for addition from either side
impl Add<Direction> for Square {
    type Output = Self;
    fn add(self, b: Direction) -> Self {
        let result = self as i32 + b as i32;
        if Square::is_square_valid(result){
            unsafe {std::mem::transmute(result)}
        }else {
            Square::SqNone
        }
    }
}
impl Add<Square> for Direction {
    type Output = Square;
    fn add(self, rhs: Square) -> Square {
        let result = self as i32 + rhs as i32;
        if Square::is_square_valid(result){
            unsafe {std::mem::transmute(result)}
        }else {
            Square::SqNone
        }
    }
}

// Overloading subtractino of a direction from a square
impl Sub<Direction> for Square {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self {
        let result = self as i32 - rhs as i32;
        if result >=0 && result <= Square::SqH8 as i32 {
            unsafe {std::mem::transmute(result)}
        }else {
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
        }
    }
}

//Overloading -= between square and direction
impl SubAssign<Direction> for Square {
    fn sub_assign(&mut self, rhs: Direction) {
       let result = *self - rhs;
            *self = *self - rhs;
        }
    }

//Overloading addition between two directions
impl Add<Direction> for Direction {
    type Output = Self;
    fn add(self, rhs: Direction) -> Self {
        let result = self as i32 + rhs as i32;
        unsafe {std::mem::transmute(result)}
    }
}

//Overloading subtraction between two directions
impl Sub<Direction> for Direction {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self {
        let result = self as i32 - rhs as i32;
        unsafe{std::mem::transmute(result)}
    }
}

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

impl Not for Piece {
    type Output = Self;
    fn not(self) -> Self {
        let result = self as i8 ^ 8;
        if result >= 0 && result <= 6 && result >= 9 && result <= 14 {
            unsafe {std::mem::transmute(result)}
        }else {
            panic!()
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
        self & rhs.bb()
    }
}
impl BitXor<Square> for Bitboard {
    type Output = Bitboard;
    fn bitxor(self, rhs: Square) -> Bitboard {
        self & rhs.bb()
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
        if Square::is_square_valid(n){
            unsafe {std::mem::transmute(n)}
        }else {
            Self::SqNone
        }
    }
    pub const fn flip_rank(&self) -> Self {
        let result = *self as i32 ^ Square::SqA8 as i32;
        if Self::is_square_valid(result){
            unsafe {std::mem::transmute(result)}
        }else {
            panic!()
        }
    }

    pub const fn flip_file(&self) -> Self {
        let result = *self as i32 ^ Square::SqH1 as i32;
        if Self::is_square_valid(result){
            unsafe {std::mem::transmute(result)}
        }else {
            panic!()
        }
    }

    pub const fn file_of(&self) -> File {
        let result = *self as i32 & 7;
        if is_file_valid(result) {
            unsafe {std::mem::transmute(result)}
        }else {
            panic!()
        }
    }

    pub const fn rank_of(&self) -> Rank {
        let result = *self as i32 >> 3;
        if is_rank_valid(result) {
            unsafe {std::mem::transmute(result)} 
        }else {
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

    #[inline]
    pub const fn rank_distance_from(&self, s: Square) -> i32 {
        let result = self.rank_of() as i32 - s.rank_of() as i32;
        let r = result.abs();
        r
    }

    #[inline]
    pub const fn file_distance_from(&self, s: Square) -> i32 {
        let result = self.file_of() as i32 - s.file_of() as i32;
        let r = result.abs();
        r
    }

    pub const fn is_square_valid(square: i32) -> bool {
        return square >= Square::SqA1 as i32 && square <= Square::SqH8 as i32
    }
}



//Overload BitAnd Between Color and Castling Rights
impl BitAnd<Color> for CastlingRights {
    type Output = Self;
    fn bitand(self, rhs: Color) -> CastlingRights {
        match rhs {
            Color::White => unsafe {std::mem::transmute(CastlingRights::WhiteCastling as i8 & self as i8)},
            Color::Black => unsafe {std::mem::transmute(CastlingRights::BlackCastling as i8 & self as i8)},
            _ => panic!()
        }
    }
}

//Overload putting CastlingRights on rhs
impl BitAnd<CastlingRights> for Color {
    type Output = CastlingRights;
    fn bitand(self, rhs: CastlingRights) -> CastlingRights {
        match self {
            Color::White => unsafe {std::mem::transmute(CastlingRights::WhiteCastling as i8 & rhs as i8)},
            Color::Black => unsafe {std::mem::transmute(CastlingRights::BlackCastling as i8 & rhs as i8)},
            _ => panic!()
        }
    }
}

const fn mate_in(ply: i32) -> Value {
    VALUE_MATE - ply
}
const fn mated_in(ply: i32) -> Value {
    -VALUE_MATE + ply
}

const fn make_square(f: File, r: Rank) -> Square {
    let result = ((r as i32) << 3) + f as i32;
    if Square::is_square_valid(result){
        unsafe {std::mem::transmute(result)}
    }else {
        panic!()
    }
}

const fn make_piece(c: Color, pt: PieceType) -> Piece {
    match c {
        Color::White => make_white_piece(pt),
        Color::Black => make_black_piece(pt),
        Color::ColorNb => panic!()
    }
}

const fn make_white_piece(pt: PieceType) -> Piece {
    match pt {
        PieceType::Pawn => Piece::WPawn,
        PieceType::Knight => Piece::WKnight,
        PieceType::Bishop => Piece::WBishop,
        PieceType::Rook => Piece::WRook,
        PieceType::Queen => Piece::WQueen,
        PieceType::King => Piece::WKing,
        _ => panic!()
    }
}

const fn make_black_piece(pt: PieceType) -> Piece {
    match pt {
        PieceType::Pawn => Piece::BPawn,
        PieceType::Knight => Piece::BKnight,
        PieceType::Bishop => Piece::BBishop,
        PieceType::Rook => Piece::BRook,
        PieceType::Queen => Piece::BQueen,
        PieceType::King => Piece::BKing,
        _ => panic!()
    }
}



const fn is_valid_move_type(data: u16) -> bool {
    data == 0 || data == 1 << 14 || data == 2 << 14 || data == 3 << 14
}

//Check if the i32 value falls within fhe file values
const fn is_file_valid(s: i32) -> bool {
    s >= File::FileA as i32 && s <= File::FileH as i32
}

//Check if the rank falls within the valid rank values
const fn is_rank_valid(s: i32) -> bool {
    s >= Rank::Rank1 as i32 && s <= Rank::Rank8 as i32
}

const fn pawn_push(color: Color) -> Direction {
    match color {
        Color::White => Direction::North,
        Color::Black => Direction::South,
        _ => panic!(),
    }
} 

const fn make_key(seed: u64) -> Key {
    return (seed * 6364136223846793005 + 1442695040888963407) as Key;
}

const fn relative_rank(color: Color, rank: Rank) -> Rank {
    let result = (rank as i32) ^ (color as i32 * 7);
    if is_rank_valid(result){
        unsafe{std::mem::transmute(result)}
    }else {
        panic!()
    }
}

// A move needs 16 bits
// bits 0-5: destination square
// 6-11 origin square
// 12-13 promotion piece type - 2 [Knight - 2, Queen - 2]
// 14-15 special move flag: promotion(1), en_passant(2), castling(3)
// en_passant bit is only set if a pawn can be captured
// Special cases are move::none() and move::null()
#[derive(Debug)]
struct Move {
    data: u16,
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Move {
    pub const fn new(data: u16) -> Self {
        Move {
            data: data,
        }
    }

    pub const fn from_to(&self) -> u16 {
        self.data & 0xFFF
    }

    pub const fn from_sq(&self) -> Square {
        let result = (self.data >> 6) & 0x3F;
        if Square::is_square_valid(result as i32) {
            unsafe {std::mem::transmute(result as i32)}
        }else {
            panic!()
        }
    }

    pub const fn to_sq(&self) -> Square {
        let result = self.data  & 0x3F;
        if Square::is_square_valid(result as i32) {
            unsafe {std::mem::transmute(result as i32)}
        }else {
            panic!()
        }
    }

    pub const fn type_of(&self) -> MoveType {
        let result = (3 << 14) & self.data;
        match result {
            0 => MoveType::Normal,
            1 => MoveType::Promotion,
            2 => MoveType::EnPassant,
            3 => MoveType::Castling,
            _ => panic!()
        }
    }

    pub const fn promotion_type(&self) -> PieceType {
        let res = ((self.data >> 12) & 3) as i32;
        match res {
            0 => PieceType::Knight,
            1 => PieceType::Bishop,
            2 => PieceType::Rook,
            3 => PieceType::Queen,
            _ => panic!()
        }
    }

    pub const fn raw(&self) -> u16 {
        self.data as u16
    }

    pub const fn is_nonzero(&self) -> bool {
        self.data != 0
    }

    pub const fn null() -> Self {
        Move {
            data: 0,
        }
    }

    pub const fn none() -> Self {
        Move {
            data: 65,
        }
    }

    pub const fn is_ok(&self) -> bool {
        self.data != Self::none().data && self.data != Self::null().data
    }
}
