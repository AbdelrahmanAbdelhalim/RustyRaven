use super::bitboard::pop_lsb;
use crate::all_pieces;
use crate::board::bitboard as bb;
use crate::board::bitboard::get_pawn_attacks_bb;
use crate::board::position as pos;
use crate::pieces_by_color_and_pt;
use crate::pieces_of_types;
use crate::types::*;

const MAX_MOVES: usize = 256;

pub struct ExtMove {
    base: Move,
    value: i32,
}

struct MoveList {
    move_list: Vec<ExtMove>,
}

impl ExtMove {
    pub fn new_from_move(m: Move) -> Self {
        Self { base: m, value: 0 }
    }
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

const fn bind_color(n: i32) -> Color {
    match n {
        WHITE => Color::White,
        BLACK => Color::Black,
        _ => unreachable!(),
    }
}

const fn bind_gentype(n: i32) -> GenType {
    match n {
        CAPTURES => GenType::Captures,
        QUIETS => GenType::Quiets,
        QUIET_CHECKS => GenType::QuietChecks,
        EVASIONS => GenType::Evasions,
        NON_EVASIONS => GenType::NonEvasions,
        LEGAL => GenType::Legal,
        _ => unreachable!(),
    }
}

pub fn make_promotions<const T: i32, const D: i32, const Enemy: bool>(
    move_list: &mut MoveList,
    to: Square,
) -> usize {
    let d = D;
    let gen_type = bind_gentype(T);
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

pub fn generate_pawn_moves<const T: i32, const C: i32>(
    pos: &pos::Position,
    move_list: &mut MoveList,
    target: Bitboard,
) {
    let us = bind_color(C);
    let them: Color = !us;
    let gen_type = bind_gentype(T);
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
                if C == Color::White as i32 {
                    make_promotions::<T, NORTH_EAST, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                } else {
                    make_promotions::<T, SOUTH_WEST, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                }
                b1 &= b1 - 1; //remove lsb
            }

            while b2 > 0 {
                if C == Color::White as i32 {
                    make_promotions::<T, NORTH_WEST, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                } else {
                    make_promotions::<T, SOUTH_EAST, true>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                }
                b2 &= b2 - 1; //remove lsb
            }

            while b3 > 0 {
                if C == Color::White as i32 {
                    make_promotions::<T, NORTH, false>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                } else {
                    make_promotions::<T, SOUTH, false>(
                        move_list,
                        Square::new_from_n(b1.trailing_zeros() as i32),
                    );
                }
                b3 &= b3 - 1; //remove lsb
            }
        }

        if gen_type == GenType::Captures
            || gen_type == GenType::Evasions
            || gen_type == GenType::NonEvasions
        {
            let mut b1 = bb::shift(pawns_not_on_7th, up_left) & enemies;
            let mut b2 = bb::shift(pawns_not_on_7th, up_right) & enemies;

            while b1 != 0 {
                let to = Square::new_from_n(b1.trailing_zeros() as i32);
                move_list.push_move(Move::new_from_to_sq(to - up_right, to));
                b1 &= b1 - 1; //pop lsb
            }

            while b2 != 0 {
                let to = Square::new_from_n(b2.trailing_zeros() as i32);
                move_list.push_move(Move::new_from_to_sq(to - up_right, to));
                b2 &= b2 - 1; //pop lsb
            }

            if pos.ep_square() != Square::SqNone {
                if gen_type == GenType::Evasions && target & (pos.ep_square() + up) != 0 {
                    return;
                }

                b1 = pawns_not_on_7th & bb::get_pawn_attacks_bb(us, pos.ep_square());
                //TODO: Push Enpassant Move
                todo!();
            }
        }
    }
}

pub fn generate_moves<const C: i32, const T: i32>() {
    let gen_type: i32 = T;
    let us = bind_color(C);
}

pub fn generate_all<const C: i32, const T: i32>() {
    let gen_type: i32 = T;
    let us = bind_color(C);
}
