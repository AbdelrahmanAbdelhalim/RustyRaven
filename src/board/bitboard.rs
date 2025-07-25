use crate::misc::Prng;
use crate::types::*;
use std::cmp::max;
use std::sync::OnceLock;

#[cfg(target_arch = "x86")]
use std::arch::x86::_pext_u32;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_pext_u64;

pub const FILEABB: Bitboard = 0x0101010101010101;
pub const FILEBBB: Bitboard = FILEABB << 1;
pub const FILECBB: Bitboard = FILEABB << 2;
pub const FILEDBB: Bitboard = FILEABB << 3;
pub const FILEEBB: Bitboard = FILEABB << 4;
pub const FILEFBB: Bitboard = FILEABB << 5;
pub const FILEGBB: Bitboard = FILEABB << 6;
pub const FILEHBB: Bitboard = FILEABB << 7;

pub const RANK1BB: Bitboard = 0xFF;
pub const RANK2BB: Bitboard = RANK1BB << (8 * 1);
pub const RANK3BB: Bitboard = RANK1BB << (8 * 2);
pub const RANK4BB: Bitboard = RANK1BB << (8 * 2);
pub const RANK5BB: Bitboard = RANK1BB << (8 * 4);
pub const RANK6BB: Bitboard = RANK1BB << (8 * 5);
pub const RANK7BB: Bitboard = RANK1BB << (8 * 6);
pub const RANK8BB: Bitboard = RANK1BB << (8 * 7);

// pub const SQNB: usize = Square::SquareNb as usize - 1; //Poissibly move these constants to the types file
// pub const PNB: usize = Piece::PieceNb as usize;
// pub const PTNB: usize = PieceType::PieceTypeNb as usize;
// pub const COLORNB: usize = Color::ColorNb as usize;
// pub const CRNB: usize = CastlingRights::CastlingRightsNb as usize;
const IS64BIT: bool = cfg!(target_pointer_width = "64");

pub static POPCNT: OnceLock<[u8; 1 << 16]> = OnceLock::new();
pub static SQUARE_DISTANCE: OnceLock<[[u8; SQNB]; SQNB]> = OnceLock::new();
pub static LINE_BB: OnceLock<[[Bitboard; SQNB]; SQNB]> = OnceLock::new();
pub static BETWEEN_BB: OnceLock<[[Bitboard; SQNB]; SQNB]> = OnceLock::new();
pub static PSEUDO_ATTACKS: OnceLock<[[Bitboard; SQNB]; PTNB]> = OnceLock::new();
pub static PAWN_ATTACKS: OnceLock<[[Bitboard; SQNB]; COLORNB]> = OnceLock::new();

pub static ROOK_MAGICS: OnceLock<[Magic; SQNB]> = OnceLock::new();
pub static BISHOP_MAGICS: OnceLock<[Magic; SQNB]> = OnceLock::new();

static ROOK_TABLE: OnceLock<Vec<Bitboard>> = OnceLock::new();
static BISHOP_TABLE: OnceLock<Vec<Bitboard>> = OnceLock::new();

pub const fn more_than_one(bb: Bitboard) -> bool {
    bb & (bb - 1) != 0 // Resets the highest bit
}

fn distance(x: Square, y: Square) -> u8 {
    let sqdt = SQUARE_DISTANCE.get().unwrap();
    sqdt[x as usize][y as usize]
}

pub const fn shift(b: Bitboard, d: Direction) -> Bitboard {
    match d {
        Direction::North => b << 8,
        Direction::South => b >> 8,
        Direction::East => (b & !FILEHBB) << 1,
        Direction::West => (b & !FILEABB) >> 1,
        Direction::NorthEast => (b & !FILEHBB) << 9,
        Direction::NorthWest => (b & !FILEABB) << 7,
        Direction::SouthEast => (b & !FILEHBB) >> 7,
        Direction::SouthWest => (b & !FILEABB) >> 9,
        Direction::None => b,
    }
}

const fn shift_twice(b: Bitboard, d: Direction) -> Bitboard {
    match d {
        Direction::North => b >> 16,
        Direction::South => b << 16,
        _ => 0,
    }
}

pub const fn pawn_attacks_bb(bb: Bitboard, c: Color) -> Bitboard {
    match c {
        Color::White => shift(bb, Direction::NorthWest) | shift(bb, Direction::NorthEast),
        Color::Black => shift(bb, Direction::SouthWest) | shift(bb, Direction::SouthEast),
        _ => 0,
    }
}

pub fn get_pawn_attacks_bb(c: Color, s: Square) -> Bitboard {
    if let Some(pawn_attacks) = PAWN_ATTACKS.get() {
        return pawn_attacks[c as usize][s as usize];
    }else {
        panic!("Attempt to access pawn attacks table prior to initialization");
    }
}

pub fn get_pseudo_attacks(pt: PieceType, s: Square) -> Bitboard {
    if let Some(pseudo_attacks) = PSEUDO_ATTACKS.get() {
        return pseudo_attacks[pt as usize][s as usize]
    }else {
        panic!("Attempt to access pseudo attacs prior to initializaiton")
    }
}

pub fn alligned(s1: Square, s2: Square, s3: Square) -> bool {
    if let Some(line_bb) = LINE_BB.get() {
        let a = line_bb[s1 as usize][s2 as usize];
        return a & s3 != 0
    }else {
        panic!("Attempt to use Line BB table prior to initialization");
    }
}

pub fn attacks_bb(pt: PieceType, s: Square, occupied: Bitboard) -> Bitboard {
    let pseudo_attacks = PSEUDO_ATTACKS.get().unwrap();
    match pt {
        PieceType::Bishop => bishop_attacks_bb(s, occupied),
        PieceType::Rook => rook_attacks_bb(s, occupied),
        PieceType::Queen => bishop_attacks_bb(s, occupied) | rook_attacks_bb(s, occupied),
        _ => pseudo_attacks[pt as usize][s as usize],
    }
}

pub fn attacks_bb_helper(
    pt: PieceType,
    s: Square,
    occupied: Bitboard,
    pseudo_attacks: &[[u64; 64]; 8],
) -> Bitboard {
    match pt {
        PieceType::Bishop => bishop_attacks_bb(s, occupied),
        PieceType::Rook => rook_attacks_bb(s, occupied),
        PieceType::Queen => bishop_attacks_bb(s, occupied) | rook_attacks_bb(s, occupied),
        _ => pseudo_attacks[pt as usize][s as usize],
    }
}

#[inline]
pub fn bishop_attacks_bb(s: Square, occupied: Bitboard) -> Bitboard {
    let bishop_table = BISHOP_TABLE.get().unwrap();
    let bishop_magics = BISHOP_MAGICS.get().unwrap();
    let idx = bishop_magics[s as usize].index(occupied);
    bishop_table[idx]
}

#[inline]
pub fn rook_attacks_bb(s: Square, occupied: Bitboard) -> Bitboard {
    let rook_table = ROOK_TABLE.get().unwrap();
    let rook_magics = ROOK_MAGICS.get().unwrap();
    let idx = rook_magics[s as usize].index(occupied);
    rook_table[idx]
}

#[inline]
pub fn pseudo_attacks_bb(pt: PieceType, s: Square) -> Bitboard {
    if let Some(pseudo_attacks) = PSEUDO_ATTACKS.get() {
        return pseudo_attacks[pt as usize][s as usize];
    }else {
        panic!("Attempt to access pseudo attacks table prior to initialization")
    }
}
//These two functions may not be needed
const fn rank_bb(r: Rank) -> Bitboard {
    RANK1BB << (8 * r as i32)
}

//@TODO: Possibly change this to a regular mut. Bitboard is a u64.
pub fn pop_lsb(bb: &mut Bitboard) -> Square {
    let ret: Square = Square::new_from_n(bb.trailing_zeros() as i32);
    let qbb = *bb as u64 - 1;
    *bb &= qbb;
    ret
}

const fn file_bb(f: File) -> Bitboard {
    FILEABB << f as i32
}

#[inline]
pub fn between_bb(s1: Square, s2: Square) -> Bitboard {
    if let Some(b_bb) = BETWEEN_BB.get() {
        return b_bb[s1 as usize][s2 as usize]
    }else {
        panic!("Attempt to access BetweenBB prior to initialization");
    }
}
#[inline]
fn least_significant_square_bb(bb: Bitboard) -> Bitboard {
    assert!(bb != 0, "bitboard is empty");
    bb & (!bb + 1)
}

fn sliding_attack(pt: &PieceType, sq: Square, occupied: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let rook_directions = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];
    let bishop_directions = [
        Direction::NorthEast,
        Direction::SouthEast,
        Direction::NorthWest,
        Direction::SouthWest,
    ];
    let direction;

    if *pt == PieceType::Rook {
        direction = &rook_directions;
    } else {
        direction = &bishop_directions;
    }

    for d in direction {
        let mut s = sq;
        'inner: while safe_destination(s, *d as i32) != 0 {
            s += *d; //@todo: Define addassign for square and direction
            attacks |= s;
            if occupied & s != 0 {
                break 'inner;
            }
        }
    }
    attacks
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

#[derive(Debug, Clone, Copy)]
struct Magic {
    mask: Bitboard,
    magic: Bitboard,
    base: usize,
    shift: usize,
}

impl Magic {
    pub fn default() -> Self {
        Self {
            mask: 0,
            magic: 0,
            base: 0,
            shift: 0,
        }
    }
    pub fn index(&self, occupied: Bitboard) -> usize {
        if IS64BIT {
            return (((occupied & self.mask).wrapping_mul(self.magic)) >> self.shift) as usize
                + self.base; //Not sure if it should be a wrapping mult
        }
        let lo = (occupied as usize & self.mask as usize) as usize;
        let hi = (occupied >> 32 as usize & self.mask >> 32) as usize;
        (lo * (self.magic as usize) ^ hi * ((self.magic >> 32) as usize)) >> self.shift
    }
}

pub fn init() {
    init_square_distance();
    init_popcnt();

    let mut rook_magics: [Magic; SQNB] = [Magic::default(); SQNB];
    let mut bishop_magics: [Magic; SQNB] = [Magic::default(); SQNB];
    let mut rook_table: Vec<Bitboard> = vec![0; 0x19000];
    let mut bishop_table: Vec<Bitboard> = vec![0; 0x1480];
    init_magics(PieceType::Rook, &mut rook_table, &mut rook_magics);
    init_magics(PieceType::Bishop, &mut bishop_table, &mut bishop_magics);
    ROOK_TABLE.get_or_init(|| rook_table);
    BISHOP_TABLE.get_or_init(|| bishop_table);
    ROOK_MAGICS.get_or_init(|| rook_magics);
    BISHOP_MAGICS.get_or_init(|| bishop_magics);

    init_other_tables();
}

fn init_popcnt() {
    let arr: [u8; 1 << 16] = std::array::from_fn(|x| x.count_ones() as u8);
    POPCNT.get_or_init(|| arr);
}

fn init_square_distance() {
    let mut sqdist: [[u8; SQNB]; SQNB] = [[0; SQNB]; SQNB];
    let a = Square::SqA1 as usize;
    let b = Square::SqH8 as usize;
    for i in a..=b {
        for j in a..=b {
            let s1 = Square::new_from_n(i as i32);
            let s2 = Square::new_from_n(j as i32);
            sqdist[i][j] = max(
                s1.rank_distance_from(s2) as u8,
                s1.file_distance_from(s2) as u8,
            );
        }
    }
    SQUARE_DISTANCE.get_or_init(|| sqdist);
}

fn init_magics(pt: PieceType, table: &mut Vec<Bitboard>, magics: &mut [Magic; SQNB]) {
    let seeds = [
        [8977, 44560, 54343, 38998, 5731, 95205, 104912, 17020],
        [728, 10316, 55013, 32803, 12281, 15100, 16645, 255],
    ];

    let mut occupancy: [Bitboard; 4096] = [0; 4096];
    let mut reference: [Bitboard; 4096] = [0; 4096];
    let mut b: Bitboard;
    let mut edges: Bitboard;
    let mut epoch: [i32; 4096] = [0; 4096];
    let mut cnt: i32 = 0;
    let mut size: usize = 0;
    let mut prev_base = 0;

    let a = Square::SqA1 as usize;
    let c = Square::SqH8 as usize;
    for i in a..=c {
        let sq = Square::new_from_n(i as i32);
        let m: &mut Magic = &mut magics[i];
        edges = ((RANK1BB | RANK8BB) & !sq.rank_bb()) | ((FILEABB | FILEHBB) & !sq.file_bb());
        m.mask = sliding_attack(&pt, sq, 0) & !edges;
        m.shift = ((IS64BIT as usize * 32) + 32) - m.mask.count_ones() as usize;
        m.base = (sq != Square::SqA1) as usize * prev_base;
        b = 0;
        size = 0;

        'carry: loop {
            occupancy[size] = b;
            reference[size] = sliding_attack(&pt, sq, b);
            size += 1;
            b = (b.wrapping_sub(m.mask)) & m.mask;
            if b == 0 {
                break 'carry;
            }
        }

        let seed = seeds[IS64BIT as usize][sq.rank_of() as usize];
        let mut rng = Prng::new(seed);
        let mut k = 0;
        cnt += 1;

        while k < size {
            m.magic = 0;
            while ((m.magic.wrapping_mul(m.mask)) >> 56).count_ones() < 6 {
                m.magic = rng.sparse_rand::<Bitboard>();
            }

            cnt += 1;
            k = 0;
            'inner: while k < size {
                let idx = m.index(occupancy[k]) - m.base;
                if epoch[idx] < cnt {
                    epoch[idx] = cnt;
                    table[m.base + idx] = reference[k];
                } else if table[m.base + idx] != reference[k] {
                    break 'inner;
                }
                k += 1;
            }
            k += 1;
        }
        prev_base += size;
    }
}

fn init_other_tables() {
    let mut line_bb: [[Bitboard; SQNB]; SQNB] = [[0; SQNB]; SQNB];
    let mut between_bb: [[Bitboard; SQNB]; SQNB] = [[0; SQNB]; SQNB];
    let mut pseudo_attacks: [[Bitboard; SQNB]; PTNB] = [[0; SQNB]; PTNB];
    let mut pawn_attacks: [[Bitboard; SQNB]; COLORNB] = [[0; SQNB]; COLORNB];
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
        pseudo_attacks[PieceType::Queen as usize][k] = pseudo_attacks[PieceType::Bishop as usize]
            [k]
            | pseudo_attacks[PieceType::Rook as usize][k];

        for piece in [PieceType::Bishop, PieceType::Rook] {
            for j in a..=b {
                let s2 = Square::new_from_n(j as i32);
                if pseudo_attacks[piece as usize][k] & s2.bb() != 0 {
                    line_bb[k][j] = (attacks_bb_helper(piece, s1, 0, &pseudo_attacks)
                        & attacks_bb_helper(piece, s2, 0, &pseudo_attacks))
                        | s1
                        | s2;
                    between_bb[k][j] = attacks_bb_helper(piece, s1, s2.bb(), &pseudo_attacks)
                        & attacks_bb_helper(piece, s2, s1.bb(), &pseudo_attacks);
                }
                between_bb[k][j] |= s2
            }
        }
    }
    LINE_BB.get_or_init(|| line_bb);
    BETWEEN_BB.get_or_init(|| between_bb);
    PSEUDO_ATTACKS.get_or_init(|| pseudo_attacks);
    PAWN_ATTACKS.get_or_init(|| pawn_attacks);
}

//Useful Debugging Function
pub fn pretty(b: Bitboard) -> String {
    let mut s = String::from("+---+---+---+---+---+---+---+---+\n");

    for r in (0..8).rev() {
        for f in 0..8 {
            if b & make_square(f, r) != 0 {
                s.push_str("| X ");
            } else {
                s.push_str("|   ");
            }
        }
        s.push_str(&format!("| {}\n+---+---+---+---+---+---+---+---+\n", r + 1));
    }
    s.push_str("  a   b   c   d   e   f   g   h\n");

    s
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn count_bishop_attacks() {
        init_popcnt();
        init_square_distance();
        let mut total = 0;
        for i in Square::SqA1 as usize..Square::SqH8 as usize {
            let sq = Square::new_from_n(i as i32);
            let m = sliding_attack(&PieceType::Bishop, sq, 0);
            let mut b: Bitboard = 0;

            'carry: loop {
                total += 1;
                b = (b.wrapping_sub(m)) & m;
                if b == 0 {
                    break 'carry;
                }
            }
        }
        println!("{}", total);
    }

    // Relies on correctness of sliding_attack function
    #[test]
    fn test_rook_magics() {
        init();
        let a = Square::SqA1 as usize;
        let b = Square::SqH8 as usize;
        for i in a..=b {
            let sq = Square::new_from_n(i as i32);
            let mut blocker: Bitboard = !0;
            let mut empty_tested = false;
            while !empty_tested {
                empty_tested = blocker == 0;
                let block: Bitboard = blocker & !sq.bb();
                let magic: Bitboard = attacks_bb(PieceType::Rook, sq, block);
                let manual: Bitboard = sliding_attack(&PieceType::Rook, sq, block);
                assert_eq!(magic, manual);
                blocker >>= 8;
            }
        }
    }

    // Relies on correctness of sliding_attack function
    #[test]
    fn test_bishop_magics() {
        init();
        let a = Square::SqA1 as usize;
        let b = Square::SqH8 as usize;
        for i in a..=b {
            let sq = Square::new_from_n(i as i32);
            let mut empty_tested = false;
            let mut blocker: Bitboard = !0;
            while !empty_tested {
                empty_tested = blocker == 0;
                let block: Bitboard = blocker & !sq.bb();
                let magic: Bitboard = attacks_bb(PieceType::Bishop, sq, block);
                let manual: Bitboard = sliding_attack(&PieceType::Bishop, sq, block);
                assert_eq!(magic, manual);
                blocker >>= 8;
            }
        }
    }


    #[test]
    fn test_more_than_one() {
        assert!(!more_than_one(4));
        assert!(!more_than_one(2));
        assert!(!more_than_one(1));
        assert!(more_than_one(5));
        assert!(more_than_one(7));
        assert!(more_than_one(9));
    }
}
