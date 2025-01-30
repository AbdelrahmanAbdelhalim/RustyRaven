use crate::misc::Prng;
use crate::types::*;
use std::cmp::max;
use std::sync::OnceLock;

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

pub static POPCNT: OnceLock<[u8; 1 << 16]> = OnceLock::new();
pub static SQUARE_DISTANCE: OnceLock<[[u8; SQNB]; SQNB]> = OnceLock::new();
pub static LINE_BB: OnceLock<[[Bitboard; SQNB]; SQNB]> = OnceLock::new();
pub static BETWEEN_BB: OnceLock<[[Bitboard; SQNB]; SQNB]> = OnceLock::new();
pub static PSEUDO_ATTACKS: OnceLock<[[Bitboard; SQNB]; PTNB]> = OnceLock::new();
pub static PAWN_ATTACKS: OnceLock<[[Bitboard; SQNB]; COLORNB]> = OnceLock::new();

pub static ROOK_MAGICS: OnceLock<[Magic; SQNB]> = OnceLock::new();
pub static BISHOP_MAGICS: OnceLock<[Magic; SQNB]> = OnceLock::new();

static ROOK_TABLE: OnceLock<[Bitboard; 0x19000]> = OnceLock::new();
static BISHOP_TABLE: OnceLock<[Bitboard; 0x1480]> = OnceLock::new();

const fn more_than_one(bb: Bitboard) -> bool {
    bb & bb - 1 == 0 // Resets the highest bit
}

fn distance(x: Square, y: Square) -> u8 {
    let sqdt = SQUARE_DISTANCE.get().unwrap();
    sqdt[x as usize][y as usize]
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

fn attacks_bb(pt: PieceType, s: Square, occupied: Bitboard) -> Bitboard {
    let pseudo_attacks = PSEUDO_ATTACKS.get().unwrap();
    match pt {
        PieceType::Bishop => bishop_attacks_bb(s, occupied),
        PieceType::Rook => rook_attacks_bb(s, occupied),
        PieceType::Queen => bishop_attacks_bb(s, occupied) | rook_attacks_bb(s, occupied),
        _ => pseudo_attacks[pt as usize][s as usize],
    }
}

#[inline]
fn bishop_attacks_bb(s: Square, occupied: Bitboard) -> Bitboard {
    let bishop_table = BISHOP_TABLE.get().unwrap();
    let bishop_magics = BISHOP_MAGICS.get().unwrap();
    let idx = bishop_magics[s as usize].index(occupied) + bishop_magics[s as usize].base;
    bishop_table[idx]
}

#[inline]
fn rook_attacks_bb(s: Square, occupied: Bitboard) -> Bitboard {
    let rook_table = ROOK_TABLE.get().unwrap();
    let rook_magics = ROOK_MAGICS.get().unwrap();
    let idx = rook_magics[s as usize].index(occupied) + rook_magics[s as usize].base;
    rook_table[idx]
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

fn safe_destination(s: Square, step: i32) -> Bitboard {
    if Square::is_square_valid(s as i32 + step) {
        let to = Square::new_from_n(s as i32 + step);
        if distance(s, to) <= 2 {
            to.bb()
        } else {
            0
        }
    } else {
        0
    }
}

struct Magic {
    mask: Bitboard,
    magic: Bitboard,
    base: usize,
    shift: usize,
    lastidx: usize,
}

impl Magic {
    pub fn default() -> Self {
        Self {
            mask: 0,
            magic: 0,
            base: 0,
            shift: 0,
            lastidx: 0,
        }
    }
    pub fn index(&self, occupied: Bitboard) -> usize {
        if IS64BIT {
            return (((occupied & self.mask) * self.magic) >> self.shift) as usize;
        }
        let lo = (occupied as usize & self.mask as usize) as usize;
        let hi = (occupied >> 32 as usize & self.mask >> 32) as usize;
        (lo * (self.magic as usize) ^ hi * ((self.magic >> 32) as usize)) >> self.shift
    }
}

// pub static LINE_BB: OnceLock<[[u8; SQNB]; SQNB]> = OnceLock::new();
// pub static BETWEEN_BB: OnceLock<[[u8; SQNB]; SQNB]> = OnceLock::new();
// pub static PSEUDO_ATTACKS: OnceLock<[[u8; SQNB]; PTNB]> = OnceLock::new();
// pub static PAWN_ATTACKS: OnceLock<[[u8; SQNB]; COLORNB]> = OnceLock::new();
pub fn init() {
    POPCNT.get_or_init(|| init_popcnt());
    SQUARE_DISTANCE.get_or_init(|| init_square_distance());

    let mut line_bb: [[Bitboard; SQNB]; SQNB] = [[0; SQNB]; SQNB];
    let mut between_bb: [[Bitboard; SQNB]; SQNB] = [[0; SQNB]; SQNB];
    let mut pseudo_attacks: [[Bitboard; SQNB]; PTNB] = [[0; SQNB]; PTNB];
    let mut pawn_attacks: [[Bitboard; SQNB]; COLORNB] = [[0; SQNB]; COLORNB];

    let mut rook_magics: [Magic; SQNB] = std::array::from_fn(|_| Magic::default());
    let mut bishop_magics: [Magic; SQNB] = std::array::from_fn(|_| Magic::default());
    let mut rook_table: [Bitboard; 0x19000] = [0; 0x19000];
    let mut bishop_table: [Bitboard; 0x1480] = [0; 0x1480];
    init_magics(PieceType::Rook, &mut rook_table, &mut rook_magics);
    init_magics(PieceType::Bishop, &mut bishop_table, &mut bishop_magics);
    ROOK_TABLE.get_or_init(|| rook_table);
    BISHOP_TABLE.get_or_init(|| bishop_table);
    ROOK_MAGICS.get_or_init(|| rook_magics);
    BISHOP_MAGICS.get_or_init(|| bishop_magics);

    init_other_tables(
        &mut line_bb,
        &mut between_bb,
        &mut pseudo_attacks,
        &mut pawn_attacks,
    );

    LINE_BB.get_or_init(|| line_bb);
    BETWEEN_BB.get_or_init(|| between_bb);
    PSEUDO_ATTACKS.get_or_init(|| pseudo_attacks);
    PAWN_ATTACKS.get_or_init(|| pawn_attacks);
}

fn init_other_tables(
    line_bb: &mut [[Bitboard; SQNB]; SQNB],
    between_bb: &mut [[Bitboard; SQNB]; SQNB],
    pseudo_attacks: &mut [[Bitboard; SQNB]; PTNB],
    pawn_attacks: &mut [[Bitboard; SQNB]; COLORNB],
) {
    let a = Square::SqA1 as usize;
    let b = Square::SqH8 as usize;

    for k in a..=b {
        let s1 = Square::new_from_n(k as i32);
        pawn_attacks[Color::White as usize][k] = pawn_attacks_bb(s1.bb(), Color::White);
        pawn_attacks[Color::Black as usize][k] = pawn_attacks_bb(s1.bb(), Color::Black);

        for step in [-9, -8, -7, -1, 1, 7, 8, 9] {
            pseudo_attacks[PieceType::King as usize][k] |= safe_destination(s1, step);
        }

        for step in [-17, -15, -10, -6, 6, 10, 15, 17] {
            pseudo_attacks[PieceType::Knight as usize][k] |= safe_destination(s1, step);
        }
        pseudo_attacks[PieceType::Bishop as usize][k] = bishop_attacks_bb(s1, 0);
        pseudo_attacks[PieceType::Rook as usize][k] = rook_attacks_bb(s1, 0);
        pseudo_attacks[PieceType::Queen as usize][k] = pseudo_attacks[PieceType::Bishop as usize][k]
            | pseudo_attacks[PieceType::Rook as usize][k];

        for piece in [PieceType::Bishop, PieceType::Rook] {
            for j in a..=b {
                let s2 = Square::new_from_n(j as i32);
                if pseudo_attacks[piece as usize][k] & s2.bb() != 0 {
                    line_bb[k][j] = (attacks_bb(piece, s1, 0) &
                                     attacks_bb(piece, s2, 0)) | s1 | s2;
                    between_bb[k][j] = (attacks_bb(piece, s1, s2.bb()) &
                                        attacks_bb(piece, s2, s1.bb()));
                }
                between_bb[k][j] |= s2
            }
        }
    }
}

fn init_popcnt() -> [u8; 1 << 16] {
    std::array::from_fn(|x| x.count_ones() as u8)
}

fn init_square_distance() -> [[u8; SQNB]; SQNB] {
    let mut sqdist: [[u8; SQNB]; SQNB] = [[0; SQNB]; SQNB];
    for i in Square::SqA1 as usize..Square::SqH8 as usize {
        for j in Square::SqA1 as usize..Square::SqH8 as usize {
            let s1 = Square::new_from_n(i as i32);
            let s2 = Square::new_from_n(j as i32);
            sqdist[i][j] = max(
                s1.rank_distance_from(s2) as u8,
                s1.file_distance_from(s2) as u8,
            );
        }
    }
    sqdist
}

fn init_magics(pt: PieceType, table: &mut [Bitboard], magics: &mut [Magic]) {
    let seeds = [
        [8977, 44560, 54343, 38998, 5731, 95205, 104912, 17020],
        [728, 10316, 55013, 32803, 12281, 15100, 16645, 255],
    ];

    let mut occupancy: [Bitboard; 4096] = [0; 4096];
    let mut reference: [Bitboard; 4096] = [0; 4096];
    let mut b: Bitboard = 0;
    let mut edges: Bitboard = 0;
    let mut epoch: [i32; 4096] = [0; 4096];
    let mut cnt: i32 = 0;
    let mut size: usize = 0;
    let mut prev_base = 0;
    let a = Square::SqA1 as usize;
    let c = Square::SqH8 as usize;

    for i in a..=c {
        let sq = Square::new_from_n(i as i32);
        edges = ((RANK1BB | RANK8BB) & !sq.rank_bb()) | ((FILEABB | FILEHBB) & !sq.file_bb());
        let m: &mut Magic = &mut magics[i];
        m.mask = sliding_attack(&pt, sq, 0);

        if IS64BIT {
            m.shift = 64 - m.mask.count_ones() as usize;
        } else {
            m.shift = 32 - m.mask.count_ones() as usize;
        }

        if sq == Square::SqA1 {
            m.base = 0;
            prev_base = m.base;
        } else {
            m.base = prev_base + size as usize;
            prev_base = m.base;
        }

        b = 0;
        size = 0;

        'carry: loop {
            occupancy[size] = b;
            reference[size] = sliding_attack(&pt, sq, b);
            size += 1;
            b = (b - m.mask) & m.mask;
            if b == 0 {
                break 'carry;
            }
        }

        let seed = seeds[IS64BIT as usize][sq.rank_of() as usize];
        let mut rng = Prng::new(seed);

        let mut k = 0;
        while k < size {
            m.magic = 0;
            while ((m.magic * m.mask) >> 56).count_ones() < 6 {
                m.magic = rng.sparse_rand::<Bitboard>();
            }

            cnt += 1;
            let mut j = 0;
            'inner: while j < size {
                let idx = m.index(occupancy[i]);
                if epoch[idx] < cnt {
                    epoch[idx] = cnt;
                    table[m.base + idx] = reference[j];
                } else if table[m.base + idx] != reference[j] {
                    break 'inner;
                }

                j += 1;
            }
        }
    }
}
