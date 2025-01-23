use crate::types::*;
use std::ops::{BitAnd, BitOr, BitXor, BitOrAssign, BitXorAssign};
use std::cmp::*;

const  FILEABB: Bitboard = 0x0101010101010101;
const  FILEBBB: Bitboard = FILEABB << 1;
const  FILECBB: Bitboard = FILEABB << 2;
const  FILEDBB: Bitboard = FILEABB << 3;
const  FILEEBB: Bitboard = FILEABB << 4;
const  FILEFBB: Bitboard = FILEABB << 5;
const  FILEGBB: Bitboard = FILEABB << 6;
const  FILEHBB: Bitboard = FILEABB << 7;

const  RANK1BB: Bitboard = 0xFF;
const  RANK2BB: Bitboard = RANK1BB << (8 * 1);
const  RANK3BB: Bitboard = RANK1BB << (8 * 2);
const  RANK4BB: Bitboard = RANK1BB << (8 * 2);
const  RANK5BB: Bitboard = RANK1BB << (8 * 4);
const  RANK6BB: Bitboard = RANK1BB << (8 * 5);
const  RANK7BB: Bitboard = RANK1BB << (8 * 6);
const  RANK8BB: Bitboard = RANK1BB << (8 * 7);

static POPCNT: [u8;1 << 16] = [0;1 << 16];

const sqnb: usize = Square::SquareNb as usize - 1;
const ptnb: usize = PieceType::PieceTypeNb as usize;
const clornb: usize = Color::ColorNb as usize;

static mut SQUARE_DISTANCE: [[u8;sqnb];sqnb] = [[0;sqnb]; sqnb];
static mut LINE_BB: [[u8;sqnb];sqnb] = [[0;sqnb]; sqnb];
static mut BETWEEN_BB: [[u8;sqnb];sqnb] = [[0;sqnb]; sqnb];
static mut PSEUDO_ATTACKS: [[u8;ptnb];sqnb] = [[0;ptnb]; sqnb];
static mut PAWN_ATTACKS: [[u8;sqnb];clornb] = [[0;sqnb]; clornb];

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

const fn more_than_one(bb: Bitboard) -> bool {
    bb & bb - 1 == 0 // Resets the highest bit
}


const fn shift(b: Bitboard, d: Direction) -> Bitboard {
    match d {
        Direction::North => b << 8,
        Direction::South => b >> 8,
        Direction::East => (b & !FILEHBB) << 1,
        Direction::West => (b & !FILEABB) >> 1,
        Direction::NorthEast => (b & !FILEHBB) << 9, 
        Direction::NorthWest =>  (b & !FILEABB) << 7,
        Direction::SouthEast => (b & !FILEHBB) >> 7, 
        Direction::SouthWest => (b & !FILEABB) >> 9
    }
}

const fn shift_twice(b: Bitboard, d: Direction) -> Bitboard {
    match d {
        Direction::North => b >> 16,
        Direction::South => b << 16,
        _ => 0
    }
}

const fn pawn_attacks_bb(bb: Bitboard, c: Color) -> Bitboard {
    match c {
        Color::White => shift(bb, Direction::NorthWest) | shift(bb, Direction::NorthEast),
        Color::Black => shift(bb, Direction::SouthWest) | shift(bb, Direction::SouthEast),
        _ => 0
    }
}

//These two functions may not be needed
const fn rank_bb(r: Rank) -> Bitboard {
    RANK1BB << (8 * r as i32)
}

const fn file_bb(f: File) -> Bitboard {
    FILEABB << f as i32
}

#[inline]
fn least_significant_square_bb(bb :Bitboard) -> Bitboard {
    assert!(bb != 0, "bitboard is empty");
    bb & (!bb + 1)
}

// fn safe_destination(s: Square, step: i32) -> Bitboard {
//     let res = s as i32 + step as i32;
//     let mut s2 = Square::SqA1;
//     if Square::is_square_valid(res){
//         s2 = unsafe {std::mem::transmute(result)};
//     }
// }

struct Magic<'a> {
    mask: Bitboard,
    magic: Bitboard,
    attacks: &'a Bitboard,
    shift: u32,
}

impl<'a> Magic<'a> {
    pub fn index(occupied: Bitboard) -> u32 {
        todo!()
    }
}

pub fn init() {
    // @todo:
    // init_magics(rooks)
    // init_magics(Bishop)
}