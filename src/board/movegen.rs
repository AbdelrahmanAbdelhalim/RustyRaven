use crate::all_pieces;
use crate::board::bitboard as bb;
use crate::board::bitboard::get_pawn_attacks_bb;
use crate::board::position as pos;
use crate::pieces_by_color_and_pt;
use crate::pieces_of_types;
use crate::types::*;

const MAX_MOVES: usize = 256;

#[derive(Debug, PartialEq, Eq)]
pub enum GenType {
    Captures,
    Quiets,
    QuietChecks,
    Evasions,
    NonEvasions,
    Legal,
}

struct ExtMove {
    base: Move,
    value: i32,
}

//Using Marker Types and Marker Traits:
pub struct White;
pub struct Black;
pub trait ColorInfo {
    const COLOR: Color;
}

impl ColorInfo for White {
    const COLOR: Color = Color::White;
}

impl ColorInfo for Black {
    const COLOR: Color = Color::Black;
}

pub struct Captures;
pub struct Quiets;
pub struct QuietChecks;
pub struct Evasions;
pub struct NonEvasions;
pub struct Legal;

pub trait GenTypeInfo {
    const GEN_TYPE: GenType;
}

impl GenTypeInfo for Captures {
    const GEN_TYPE: GenType = GenType::Captures;
}
impl GenTypeInfo for Quiets {
    const GEN_TYPE: GenType = GenType::Quiets;
}
impl GenTypeInfo for QuietChecks {
    const GEN_TYPE: GenType = GenType::QuietChecks;
}
impl GenTypeInfo for Evasions {
    const GEN_TYPE: GenType = GenType::Evasions;
}
impl GenTypeInfo for NonEvasions {
    const GEN_TYPE: GenType = GenType::NonEvasions;
}
impl GenTypeInfo for Legal {
    const GEN_TYPE: GenType = GenType::Legal;
}

pub struct North;
pub struct South;
pub struct East;
pub struct West;
pub struct North_East;
pub struct South_East;
pub struct South_West;
pub struct North_West;

pub trait DirectionType {
    const DIR: Direction;
}

impl DirectionType for North {
    const DIR: Direction = Direction::North;
}
impl DirectionType for South {
    const DIR: Direction = Direction::South;
}
impl DirectionType for East {
    const DIR: Direction = Direction::East;
}
impl DirectionType for West {
    const DIR: Direction = Direction::West;
}
impl DirectionType for North_East {
    const DIR: Direction = Direction::NorthEast;
}
impl DirectionType for North_West {
    const DIR: Direction = Direction::NorthWest;
}
impl DirectionType for South_East {
    const DIR: Direction = Direction::SouthEast;
}
impl DirectionType for South_West {
    const DIR: Direction = Direction::SouthWest;
}
impl ExtMove {
    fn set_from_move(&mut self, m: Move) {
        self.set_from_move(m);
    }
}

struct MoveList {
    moveList: [ExtMove; MAX_MOVES],
}

pub fn make_promotions<T: GenTypeInfo, D: DirectionType, const Enemy: bool>(
    move_list: &mut Vec<ExtMove>,
    to: Square,
) -> usize {
    let gen_type: GenType = T::GEN_TYPE;
    let d: Direction = D::DIR;

    let all = gen_type == GenType::Captures || gen_type == GenType::Evasions;
    let mut cur: usize = 0;
    if gen_type == GenType::Captures || all {
        cur += 1;
        todo!();
    }

    if gen_type == GenType::Evasions && Enemy || (gen_type == GenType::Quiets && !Enemy) || all {
        todo!();
    }
    cur
}
pub fn generate_pawn_moves<T: GenTypeInfo, C: ColorInfo>(
    pos: &pos::Position,
    move_list: &mut MoveList,
    target: Bitboard,
) {
    let us = C::COLOR;
    let gen_type = T::GEN_TYPE;
    let them = !us;
    let TRank7BB = if us == Color::White {
        bb::RANK7BB
    } else {
        bb::RANK2BB
    };
    let TRank3BB = if us == Color::White {
        bb::RANK3BB
    } else {
        bb::RANK6BB
    };
    let up: Direction = pawn_push(us);
    let up_right = if us == Color::White {
        Direction::NorthEast
    } else {
        Direction::SouthWest
    };
    let up_left = if us == Color::White {
        Direction::NorthWest
    } else {
        Direction::SouthEast
    };

    let empty_squares: Bitboard = !pos.all_pieces();
    let enemies = if gen_type == GenType::Evasions {
        pos.checkers()
    } else {
        pos.pieces_by_color(them)
    };
    let pawns_on_7th = pieces_by_color_and_pt!(pos, us, PieceType::Pawn) & TRank7BB;
    let pawns_not_on_7th = pieces_by_color_and_pt!(pos, us, PieceType::Pawn) & !TRank7BB;

    if gen_type != GenType::Captures {
        let mut b1 = bb::shift(pawns_on_7th, up) & empty_squares;
        let mut b2 = bb::shift(b1 & TRank3BB, up) & empty_squares;

        if gen_type == GenType::Evasions {
            b1 &= target;
            b2 &= target;
        }

        if gen_type == GenType::QuietChecks {
            let ksq = pos.square(them, PieceType::King);
            let dcCandidatePawns = pos.blockers_for_king(them) & !ksq.file_bb(); 
            // b1 &= pos.pawn_attacks_bb(them, ksq) | bb::shift(dcCandidatePawns, up);
        }
    }
}
pub fn generate<C: ColorInfo, T: GenTypeInfo>(
    pos: &pos::Position,
    move_list: &mut Vec<ExtMove>,
    target: Bitboard,
) {
}
