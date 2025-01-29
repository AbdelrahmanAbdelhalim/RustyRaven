use crate::misc::Prng;
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

const SQNB: usize = Square::SquareNb as usize - 1;
const PTNB: usize = PieceType::PieceTypeNb as usize;
const COLORNB: usize = Color::ColorNb as usize;
const IS64BIT: bool = cfg!(target_pointer_width = "64");

static mut POPCNT: [u8; 1 << 16] = [0; 1 << 16];
static mut SQUARE_DISTANCE: [[u8; SQNB]; SQNB] = [[0; SQNB]; SQNB];
static mut LINE_BB: [[u8; SQNB]; SQNB] = [[0; SQNB]; SQNB];
static mut BETWEEN_BB: [[u8; SQNB]; SQNB] = [[0; SQNB]; SQNB];
static mut PSEUDO_ATTACKS: [[u8; SQNB]; PTNB] = [[0; SQNB]; PTNB];
static mut PAWN_ATTACKS: [[u8; SQNB]; COLORNB] = [[0; SQNB]; COLORNB];


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
struct Magic<'a> {
    mask: Bitboard,
    magic: Bitboard,
    attacks: &'a mut [Bitboard],
    shift: usize,
}

impl <'a> Magic<'a> {
    pub fn default() -> Self {
        Self {
            mask: 0,
            magic: 0,
            attacks: &mut [],
            shift: 0,
        }
    }
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

struct BbTables<'a> {
    SQUARE_DISTANCE: [[u8; SQNB]; SQNB],
    LINE_BB: [[Bitboard; SQNB]; SQNB],
    BETWEEN_BB: [[Bitboard; SQNB]; SQNB],
    PSEUDO_ATTACKS: [[Bitboard; SQNB]; PTNB],
    PAWN_ATTACKS: [[Bitboard; SQNB]; COLORNB],
    POPCNT: [u8; 1 << 16],
    RookMagics: [Magic<'a>; SQNB],
    BishopMagics: [Magic<'a>; SQNB],
}

impl <'a> BbTables <'a> {
    fn new() -> Self {
        let mut sqdt = [[0; SQNB]; SQNB];
        let mut pawn_attacks = [[0; SQNB]; COLORNB];
        let mut pseudo_attacks: [[Bitboard; SQNB]; PTNB] = [[0; SQNB]; PTNB];
        let mut popcnt = [0; 1 << 16];
        let mut linebb = [[0; SQNB]; SQNB];
        let mut betweenbb = [[0; SQNB]; SQNB];

        let mut rook_magics: [Magic; SQNB] = core::array::from_fn(|_| Magic::default());
        let mut bishop_magics: [Magic; SQNB] = core::array::from_fn(|_| Magic::default());
        let mut rook_table: [Bitboard; 0x19000] = [0;0x19000];
        let mut bishop_table: [Bitboard; 0x1480] = [0;0x1480];

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
        Self::init_magics(PieceType::Rook, &mut rook_table, &mut rook_magics);
        Self::init_magics(PieceType::Bishop, &mut bishop_table, &mut bishop_magics);

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
            RookMagics: rook_magics,
            BishopMagics: bishop_magics,
        }
    }

    fn init_magics<'b>(pt: PieceType, table: &'b mut [Bitboard], magics: &'b mut [Magic]) {
        //Optimal seeds to ipick the correct magic number in the shortest time
        let seeds = [
            [8977, 44560, 54343, 38998, 5731, 95205, 104912, 17020],
            [728, 10316, 55013, 32803, 12281, 15100, 16645, 255],
        ];

        let mut occupancy: [Bitboard;4096] = [0;4096];
        let mut reference: [Bitboard;4096] = [0;4096];
        let mut b: Bitboard;
        let mut epoch: [i32;4096] = [0;4096];
        let mut cnt: i32 = 0;
        let mut size: usize = 0;

        let i = Square::SqA1 as usize;
        let j = Square::SqH8 as usize;

        for s in i..=j {
            let sq: Square = Square::new_from_n(s as i32);
            let edges: Bitboard = ((RANK1BB | RANK8BB) & !sq.rank_bb()) | ((FILEABB | FILEHBB) & !sq.file_bb());
            let mut m = &mut magics[s]; 

            m.mask = Self::sliding_attack(&pt, sq, 0) & !edges;
            let mut arch = 32;
            if IS64BIT {
                arch = 64;
            }
            m.shift = (arch - m.mask.count_ones()) as usize;

            if sq == Square::SqA1 {
                m.attacks = &mut table[0..];
            }else {
                m.attacks = &mut magics[s - 1].attacks[size..];
            }
            b = 0;
            size = 0;

            'inner: loop {
                occupancy[size] = b;
                reference[size] = Self::sliding_attack(&pt, sq, b);
                size += 1;
                b = (b - m.mask) & m.mask;
                if b == 0 {
                    break 'inner
                }
            }

            let mut prng: Prng = Prng::new(seeds[IS64BIT as usize][sq.rank_of() as usize]);
            let mut k = 0;

            while k < size {
                m.magic = 0;
                while ((m.magic * m.mask) >> 56).count_ones() < 6 {
                    m.magic = prng.sparse_rand::<Bitboard>();
                }
                k = 0;
                'inner: while k < size {
                    cnt += 1;
                    k += 1;
                    let idx = m.index(occupancy[i]);
                    if epoch[idx] < cnt {
                        epoch[idx] = cnt;
                        m.attacks[idx] = reference[i];
                    }else if m.attacks[idx] != reference[i] {
                        break 'inner
                    }
                }
            }
        }
    }

    fn safe_destination(s1: Square, step: i32, dist: &[[u8; SQNB]; SQNB]) -> Bitboard {
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
