use crate::all_pieces;
use crate::board::bitboard as bb;
use crate::board::bitboard::get_pawn_attacks_bb;
use crate::board::position as pos;
use crate::pieces_by_color_and_pt;
use crate::pieces_of_types;
use crate::types::*;

use super::bitboard::pop_lsb;

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
    pub fn new_from_move(m: Move) -> Self {
        Self { base: m, value: 0 }
    }
}

struct MoveList {
    move_list: Vec<ExtMove>,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            move_list: Vec::with_capacity(MAX_MOVES),
        }
    }

    pub fn push_move(&mut self, mv: Move) {
        self.move_list.push(ExtMove::new_from_move(mv));
    }

    pub fn push_move_ext_move(&mut self, mv: ExtMove) {
        self.move_list.push(mv);
    }
}

pub fn make_promotions<T: GenTypeInfo, D: DirectionType, const Enemy: bool>(
    move_list: &mut MoveList,
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

        while b1 != 0 {
            let to = b1.trailing_zeros();
            let to = Square::new_from_n(to as i32);
            b1 = b1 & b1 - 1; // Pop LSB
            move_list.push_move(Move::new_from_to_sq(to - up, to));
        }

        while b2 != 0 {
            let to = b2.trailing_zeros();
            let to = Square::new_from_n(to as i32);
            b2 = b2 & b2 - 1;
            move_list.push_move(Move::new_from_to_sq(to - up - up, to));
        }

        if pawns_not_on_7th != 0 {
            let mut b1 = bb::shift(pawns_not_on_7th, up_right) & enemies;
            let mut b2 = bb::shift(pawns_not_on_7th, up_left) & enemies;
            let mut b3 = bb::shift(pawns_not_on_7th, up) & empty_squares;

            if gen_type == GenType::Evasions {
                b3 &= target;
            }
            while b1 > 0 {
                if C::COLOR == Color::White {
                    make_promotions::<T, North_East, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                } else {
                    make_promotions::<T, South_West, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                }
                b1 &= b1 - 1; //remove lsb
            }

            while b2 > 0 {
                if C::COLOR == Color::White {
                    make_promotions::<T, North_West, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                } else {
                    make_promotions::<T, South_East, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                }
                b2 &= b2 - 1; //remove lsb
            }

            while b3 > 0 {
                if C::COLOR == Color::White {
                    make_promotions::<T, North, false>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                } else {
                    make_promotions::<T, South, false>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                }
                b3 &= b3 - 1; //remove lsb
            }
        }

        if T::GEN_TYPE == GenType::Captures
            || T::GEN_TYPE == GenType::Evasions
            || T::GEN_TYPE == GenType::NonEvasions
        {
            let mut b1 = bb::shift(pawns_not_on_7th, up_left) & enemies;
            let mut b2 = bb::shift(pawns_not_on_7th, up_right) & enemies;

            while b1 != 0 {
                let to = Square::new_from_n(b1.trailing_zeros() as i32);
                move_list.push_move(Move::new_from_to_sq(to - up_right, to));
                b1 &= b1 - 1; //pop lsb
            }
        }
    }
}
pub fn generate<C: ColorInfo, T: GenTypeInfo>(
    pos: &pos::Position,
    move_list: &mut Vec<ExtMove>,
    target: Bitboard,
) {
}
