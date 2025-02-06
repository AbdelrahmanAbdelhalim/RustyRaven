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
    

}
