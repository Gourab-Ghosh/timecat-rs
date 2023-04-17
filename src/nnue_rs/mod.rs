//!# Rust NNUE inference library
//!`nnue-rs` is an [NNUE](https://www.chessprogramming.org/NNUE) inference library written in Rust.

pub mod ops;
pub mod layers;
pub mod stockfish;

macro_rules! simple_enum {
    ($(
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),*
        }
    )*) => {$(
        $(#[$attr])*
        $vis enum $name {
            $($variant),*
        }
        
        impl $name {
            pub const NUM: usize = [$(Self::$variant),*].len();
            pub const ALL: [Self; Self::NUM] = [$(Self::$variant),*];
            #[inline]
            pub fn from_index(index: usize) -> Self {
                $(#[allow(non_upper_case_globals)]
                const $variant: usize = $name::$variant as usize;)*
                #[allow(non_upper_case_globals)]
                match index {
                    $($variant => Self::$variant,)*
                    _ => panic!("Index {} is out of range.", index)
                }
            }
        }
    )*};
}

simple_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
    pub enum Piece {
        Pawn,
        Knight,
        Bishop,
        Rook,
        Queen,
        King
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Color {
        White,
        Black
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8
    }
}

impl std::ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Square {
    pub fn flip(self) -> Self {
        //Flip upper 3 bits, which represent the rank
        Self::from_index(self as usize ^ 0b111_000)
    }

    pub fn rotate(self) -> Self {
        //Flip both rank and file bits
        Self::from_index(self as usize ^ 0b111_111)
    }
}
