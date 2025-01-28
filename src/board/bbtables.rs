use crate::types::*;
use std::cmp::*;

#[cfg(target_arch = "x86")]
use std::arch::x86::_pext_u32;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_pext_u64;

const FILEABB: Bitboard = 0x0101010101010101;
const FILEBBB: Bitboard = FILEABB << 1;
const FILECBB: Bitboard = FILEABB << 2;
const FILEDBB: Bitboard = FILEABB << 3;
const FILEEBB: Bitboard = FILEABB << 4;
const FILEFBB: Bitboard = FILEABB << 5;
const FILEGBB: Bitboard = FILEABB << 6;
const FILEHBB: Bitboard = FILEABB << 7;

const RANK1BB: Bitboard = 0xFF;
const RANK2BB: Bitboard = RANK1BB << (8 * 1);
const RANK3BB: Bitboard = RANK1BB << (8 * 2);
const RANK4BB: Bitboard = RANK1BB << (8 * 2);
const RANK5BB: Bitboard = RANK1BB << (8 * 4);
const RANK6BB: Bitboard = RANK1BB << (8 * 5);
const RANK7BB: Bitboard = RANK1BB << (8 * 6);
const RANK8BB: Bitboard = RANK1BB << (8 * 7);

const sqnb: usize = Square::SquareNb as usize - 1;
const ptnb: usize = PieceType::PieceTypeNb as usize;
const clornb: usize = Color::ColorNb as usize;
const IS64BIT: bool = cfg!(target_pointer_width = "64");

static mut POPCNT: [u8; 1 << 16] = [0; 1 << 16];
static mut SQUARE_DISTANCE: [[u8; sqnb]; sqnb] = [[0; sqnb]; sqnb];
static mut LINE_BB: [[u8; sqnb]; sqnb] = [[0; sqnb]; sqnb];
static mut BETWEEN_BB: [[u8; sqnb]; sqnb] = [[0; sqnb]; sqnb];
static mut PSEUDO_ATTACKS: [[u8; sqnb]; ptnb] = [[0; sqnb]; ptnb];
static mut PAWN_ATTACKS: [[u8; sqnb]; clornb] = [[0; sqnb]; clornb];

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
        Direction::NorthWest => (b & !FILEABB) << 7,
        Direction::SouthEast => (b & !FILEHBB) >> 7,
        Direction::SouthWest => (b & !FILEABB) >> 9,
    }
}

const fn shift_twice(b: Bitboard, d: Direction) -> Bitboard {
    match d {
        Direction::North => b >> 16,
        Direction::South => b << 16,
        _ => 0,
    }
}

const fn pawn_attacks_bb(bb: Bitboard, c: Color) -> Bitboard {
    match c {
        Color::White => shift(bb, Direction::NorthWest) | shift(bb, Direction::NorthEast),
        Color::Black => shift(bb, Direction::SouthWest) | shift(bb, Direction::SouthEast),
        _ => 0,
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
fn least_significant_square_bb(bb: Bitboard) -> Bitboard {
    assert!(bb != 0, "bitboard is empty");
    bb & (!bb + 1)
}

fn sliding_attack(pt: PieceType, sq: Square, occupied: Bitboard) -> Bitboard {
    let attacks = 0;
    let RookDirections = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];
    let BishopDirections = [
        Direction::NorthEast,
        Direction::SouthEast,
        Direction::NorthWest,
        Direction::SouthWest,
    ];
    let direction;

    if pt == PieceType::Rook {
        direction = &RookDirections;
    } else {
        direction = &BishopDirections;
    }

    for d in direction {
        let s = sq;
    }
    0
}

//This holds all magic bitboards relevant data for a single square
struct Magic {
    mask: Bitboard,
    magic: Bitboard,
    // attacks: &'a [Bitboard],
    shift: usize,
}

impl Magic {
    pub fn index(&self, occupied: Bitboard) -> usize {
        #[cfg(target_arch = "x86_64")]
        if std::is_x86_feature_detected!("bmi2") {
            return _pext_u64(occupied, self.mask) as usize;
        }

        if IS64BIT {
            return (((occupied & self.mask) * self.magic) >> self.shift) as usize;
        }
        let lo = occupied as usize * self.mask as usize;
        let hi = (occupied >> 32) as usize & (self.mask >> 32) as usize;
        let magic = self.magic as usize;
        return (lo * magic ^ hi * (magic >> 32)) >> self.shift;
    }
}

struct BbTables {
    SQUARE_DISTANCE: [[u8; sqnb]; sqnb],
    LINE_BB: [[u8; sqnb]; sqnb],
    BETWEEN_BB: [[u8; sqnb]; sqnb],
    PSEUDO_ATTACKS: [[Bitboard; sqnb]; ptnb],
    PAWN_ATTACKS: [[u8; sqnb]; clornb],
    POPCNT: [u8; 1 << 16],
    // RookMagics: [Magic; sqnb],
    // BishopMagics: [Magic; sqnb],
}

impl BbTables {
    fn new() -> Self {
        let mut sqdt = [[0; sqnb]; sqnb];
        let mut pawn_attacks = [[0; sqnb]; clornb];
        let mut pseudo_attacks: [[Bitboard; sqnb]; ptnb] = [[0; sqnb]; ptnb];
        let mut popcnt = [0; 1 << 16];
        let mut linebb = [[0; sqnb]; sqnb];
        let mut betweenbb = [[0; sqnb]; sqnb];

        let mut RookMagics: [Magic; sqnb];
        let mut BishopMagics: [Magic; sqnb];
        let mut RookTable: [Magic; 0x19000];
        let mut BishopTable: [Magic; 0x1480];

        //init popcount
        for i in 0..(1 << 16) {
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
        // Self::init_magics(PieceType::Rook, &RookTable, &RookMagics);
        // Self::init_magics(PieceType::Bishop, &BishopTable, &BishopMagics);

        //init pawn attacks, pseudo attacks
        for i in a..=b {
            let s1 = Square::new_from_n(i as i32);
            pawn_attacks[Color::White as usize][i] = pawn_attacks_bb(s1.bb(), Color::White) as u8;
            pawn_attacks[Color::Black as usize][i] = pawn_attacks_bb(s1.bb(), Color::Black) as u8;

            //init pseudo attacks for king
            for step in [-9, -8, -7, -1, 1, 7, 8, 9] {
                pseudo_attacks[PieceType::King as usize][i] |=
                    Self::safe_destination(s1, step, &sqdt);
            }

            //init pseudo attacks for knight
            for step in [-17, -15, -10, -6, 6, 10, 15, 17] {
                pseudo_attacks[PieceType::Knight as usize][i] |=
                    Self::safe_destination(s1, step, &sqdt);
            }
        }

        BbTables {
            POPCNT: popcnt,
            SQUARE_DISTANCE: sqdt,
            PAWN_ATTACKS: pawn_attacks,
            PSEUDO_ATTACKS: pseudo_attacks,
            LINE_BB: linebb,
            BETWEEN_BB: betweenbb,
            // RookMagics: RookMagics,
            // BishopMagics: BishopMagics,
        }
    }

    fn init_magics(pt: PieceType, table: &[Bitboard], magics: &mut [Magic]) {
        //Optimal seeds to ipick the correct magic number in the shortest time
        let seeds = [
            [8977, 44560, 54343, 38998, 5731, 95205, 104912, 17020],
            [728, 10316, 55013, 32803, 12281, 15100, 16645, 255],
        ];

        let occupancy: [Bitboard;4096];
        let reference: [Bitboard;4096];
        let edges: Bitboard;
        let b: Bitboard;
        let epoch: [i32;4096] = [0;4096];
        let mut cnt: i32 = 0;
        let mut size: i32 = 0;

        let a = Square::SqA1 as usize;
        let b = Square::SqH8 as usize;

        for s in a..=b {
            let sq: Square = Square::new_from_n(s as i32);
            let edges: Bitboard = ((RANK1BB | RANK8BB) & !sq.rank_bb()) | ((FILEABB | FILEHBB) & !sq.file_bb());
            let mut m = &mut magics[s]; 
            m.mask = Self::sliding_attack(&pt, sq, 0);

            let mut arch = 32;
            if IS64BIT {
                arch = 64;
            }
            m.shift = (arch - m.mask.count_ones()) as usize;
        }
    }

    fn safe_destination(s1: Square, step: i32, dist: &[[u8; sqnb]; sqnb]) -> Bitboard {
        let res = s1 as i32 + step;
        if !Square::is_square_valid(res) {
            0
        } else {
            let s2 = Square::new_from_n(res);
            let dist = dist[s1 as usize][s2 as usize];
            if dist <= 2 {
                s2.bb()
            } else {
                0
            }
        }
    }

    fn sliding_attack(pt: &PieceType, sq: Square, occupied: Bitboard) -> Bitboard {
        let attacks = 0;
        let RookDirections = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        let BishopDirections = [
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::NorthWest,
            Direction::SouthWest,
        ];
        let direction;

        if *pt == PieceType::Rook {
            direction = &RookDirections;
        } else {
            direction = &BishopDirections;
        }

        for d in direction {
            let s = sq;
        }
        0
    }

    fn distance(&self, s1: Square, s2: Square) -> u8 {
        self.SQUARE_DISTANCE[s1 as usize][s2 as usize]
    }
}
