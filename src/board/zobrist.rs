use crate::types::*;
use std::sync::OnceLock;
use crate::misc::*;

pub static PSQ: OnceLock<[[Key; SQNB]; PNB]> = OnceLock::new();
pub static ENPASSANT: OnceLock<[Key; FNB]> = OnceLock::new(); 
pub static CASTLING: OnceLock<[Key; CRNB]> = OnceLock::new();
pub static SIDE: OnceLock<Key> = OnceLock::new();   
pub static NOPAWNS: OnceLock<Key> = OnceLock::new();   


pub fn init_zobrist() {
    let mut psq : [[Key; SQNB]; PNB]= [[0; SQNB]; PNB];
    let mut enpassant: [Key; FNB] = [0; FNB];
    let mut castling: [Key; CRNB] = [0; CRNB];
    let mut side = 0;
    let mut nopawns = 0;

    let file_a = File::FileA as usize;
    let file_h = File::FileH as usize;
    let mut prng = Prng::new(1070372);

    for i in 0..PNB {
        for j in 0..SQNB {
            psq[i][j] = prng.rand::<Key>();
        }
    }

    for i in file_a..=file_h {
        enpassant[i] = prng.rand::<Key>();
    } 

    for i in 0..CRNB {
        castling[i] = prng.rand::<Key>();
    }

    side = prng.rand::<Key>();
    nopawns = prng.rand::<Key>();

    PSQ.get_or_init(|| psq);
    ENPASSANT.get_or_init(|| enpassant);
    CASTLING.get_or_init(|| castling);
    SIDE.get_or_init(|| side);
    NOPAWNS.get_or_init(|| nopawns);
}
