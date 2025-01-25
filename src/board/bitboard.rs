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

const sqnb: usize = Square::SquareNb as usize - 1;
const ptnb: usize = PieceType::PieceTypeNb as usize;
const clornb: usize = Color::ColorNb as usize;

static mut POPCNT: [u8;1 << 16] = [0;1 << 16];
static mut SQUARE_DISTANCE: [[u8;sqnb];sqnb] = [[0;sqnb]; sqnb];
static mut LINE_BB: [[u8;sqnb];sqnb] = [[0;sqnb]; sqnb];
static mut BETWEEN_BB: [[u8;sqnb];sqnb] = [[0;sqnb]; sqnb];
static mut PSEUDO_ATTACKS: [[u8;sqnb];ptnb] = [[0;sqnb]; ptnb];
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

//This holds all magic bitboards relevant data for a single square
struct Magic<'a> {
    mask: Bitboard,
    magic: Bitboard,
    attacks: &'a [Bitboard],
    shift: u32,
}

impl<'a> Magic<'a> {
    // pub fn init() -> Self {

    // }
}

struct BbTables {
    SQUARE_DISTANCE: [[u8;sqnb];sqnb],
    LINE_BB: [[u8;sqnb];sqnb],
    BETWEEN_BB: [[u8;sqnb];sqnb],
    PSEUDO_ATTACKS: [[Bitboard;sqnb];ptnb],
    PAWN_ATTACKS: [[u8;sqnb];clornb], 
    POPCNT: [u8;1 << 16],
    // RookMagics: [Magic; sqnb],
    // BishopMagics: [Magic; sqnb],
}

impl BbTables {
    fn new() -> Self {
        let mut sqdt = [[0;sqnb];sqnb];
        let mut pawn_attacks = [[0;sqnb];clornb];
        let mut pseudo_attacks: [[Bitboard;sqnb];ptnb] = [[0;sqnb]; ptnb];
        let mut popcnt = [0; 1<<16];
        let mut linebb = [[0;sqnb];sqnb];
        let mut betweenbb = [[0;sqnb];sqnb];

        //init popcount
        for i in 0..(1<<16) {
            popcnt[i] = i.count_ones() as u8;
        }

        //init square distance
        let a = Square::SqA1 as usize;
        let b = Square::SqH8 as usize;
        for i in a..=b {
            for j in a..=b {
                let s1 = Square::new_from_n(i as i32);
                let s2 = Square::new_from_n(j as i32);
                sqdt[i][j] = max(s1.rank_distance_from(s2), s1.file_distance_from(s2)) as u8;
            }
        }

        //init pawn attacks, pseudo attacks
        for i in a..=b {
            let s1 = Square::new_from_n(i as i32);
            pawn_attacks[Color::White as usize][i] = pawn_attacks_bb(s1.bb(), Color::White) as u8;
            pawn_attacks[Color::Black as usize][i] = pawn_attacks_bb(s1.bb(), Color::Black) as u8;

            //init pseudo attacks for king
            for step in [-9, -8, -7, -1, 1, 7, 8, 9] {
                pseudo_attacks[PieceType::King as usize][i] |=  Self::safe_destination(s1, step, &sqdt);
            }
            
            //init pseudo attacks for knight
            for step in [-17, -15, -10, -6, 6, 10, 15, 17] {
                pseudo_attacks[PieceType::Knight as usize][i] |=  Self::safe_destination(s1, step, &sqdt);
            }


        }
        
        BbTables {
            POPCNT: popcnt,
            SQUARE_DISTANCE: sqdt,
            PAWN_ATTACKS: pawn_attacks,
            PSEUDO_ATTACKS: pseudo_attacks,
            LINE_BB: linebb,
            BETWEEN_BB: betweenbb,
        }
    }


    fn safe_destination(s1: Square, step: i32, dist: &[[u8;sqnb];sqnb]) -> Bitboard {
        let res = s1 as i32 + step;
        if !Square::is_square_valid(res){
            0
        }else {
            let s2 = Square::new_from_n(res);
            let dist = dist[s1 as usize][s2 as usize];
            if dist <= 2 {
                s2.bb()
            }else {
                0
            }
        }
    }
    // fn init_popcnt() -> [u8; 1<<16] {
    //     let mut popcnt = [0; 1<<16];
    //     for i in 0..(1<<16) {
    //         popcnt[i] = i.count_ones() as u8;
    //     }
    //     popcnt
    // }

    // fn init_square_distance() -> [[u8; sqnb]; sqnb] {
    //     let mut sqdt = [[0;sqnb];sqnb];
    //     let a = Square::SqA1 as usize;
    //     let b = Square::SqH8 as usize;
    //     for i in a..=b {
    //         for j in a..=b {
    //             let s1 = Square::new_from_n(i as i32);
    //             let s2 = Square::new_from_n(j as i32);
    //             sqdt[i][j] = max(s1.rank_distance_from(s2), s1.file_distance_from(s2)) as u8;
    //         }
    //     }
    //     sqdt
    // }

    // fn init_pawn_and_pseudo_attacks()  {
    //     let mut pawn_attacks = [[0;sqnb];clornb];
    //     let mut pseudo_attacks: [[u8;sqnb];ptnb] = [[0;sqnb]; ptnb];
    //     let a = Square::SqA1 as usize;
    //     let b = Square::SqH8 as usize;
    //     for i in a..=b {
    //         let s1 = Square::new_from_n(i as i32);
    //         pawn_attacks[Color::White as usize][i] = pawn_attacks_bb(s1.bb(), Color::White) as u8;
    //         pawn_attacks[Color::Black as usize][i] = pawn_attacks_bb(s1.bb(), Color::Black) as u8;
            
            // for step in [-9, -8, -7, -1, 1, 7, 8, 9].iter() {
            //     pseudo_attacks[PieceType::King as usize][i] |= 
            // }
            // for 
            // for
        // }
    // }

    fn distance(&self, s1: Square, s2: Square) -> u8 {
        self.SQUARE_DISTANCE[s1 as usize][s2 as usize]
    }

}