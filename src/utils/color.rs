use super::*;
pub use Color::*;

macro_rules! generate_functions {
    ($func_name:ident, $black_rank:expr, $white_rank:expr) => {
        #[inline]
        pub fn $func_name(self) -> Rank {
            *get_item_unchecked!(const { [$black_rank, $white_rank] }, self.to_index())
        }

        paste!{
            #[inline]
            pub fn [<$func_name _bitboard>] (self) -> BitBoard {
                *get_item_unchecked!(const { [$black_rank.to_bitboard(), $white_rank.to_bitboard()] }, self.to_index())
            }
        }
    };
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Color {
    #[inline]
    pub const unsafe fn from_int(int: u8) -> Self {
        std::mem::transmute(int)
    }

    #[inline]
    pub const unsafe fn from_index(index: usize) -> Self {
        Self::from_int(index as u8)
    }

    #[inline]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn to_my_backrank(self) -> Rank {
        self.to_first_rank()
    }

    #[inline]
    pub fn to_their_backrank(self) -> Rank {
        self.to_eighth_rank()
    }

    #[inline]
    pub fn to_my_backrank_bitboard(self) -> BitBoard {
        self.to_first_rank_bitboard()
    }

    #[inline]
    pub fn to_their_backrank_bitboard(self) -> BitBoard {
        self.to_eighth_rank_bitboard()
    }

    generate_functions!(to_first_rank, Rank::Eighth, Rank::First);
    generate_functions!(to_second_rank, Rank::Seventh, Rank::Second);
    generate_functions!(to_third_rank, Rank::Sixth, Rank::Third);
    generate_functions!(to_fourth_rank, Rank::Fifth, Rank::Fourth);
    generate_functions!(to_fifth_rank, Rank::Fourth, Rank::Fifth);
    generate_functions!(to_sixth_rank, Rank::Third, Rank::Sixth);
    generate_functions!(to_seventh_rank, Rank::Second, Rank::Seventh);
    generate_functions!(to_eighth_rank, Rank::First, Rank::Eighth);
}

impl Not for Color {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        *get_item_unchecked!(const { [White, Black] }, self.to_index())
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            get_item_unchecked!(const { ["Black", "White"] }, self.to_index())
        )
    }
}

impl FromStr for Color {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "b" | "black" => Ok(Black),
            "w" | "white" => Ok(White),
            _ => Err(TimecatError::InvalidColorString {
                s: s.to_string().into(),
            }),
        }
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for Color {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(boolean) = ob.extract::<bool>() {
            return if boolean {
                Ok(Self::White)
            } else {
                Ok(Self::Black)
            };
        }
        if let Ok(s) = ob.extract::<&str>()
            && let Ok(color) = s.parse()
        {
            return Ok(color);
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string().into(),
            to: std::any::type_name::<Self>().into(),
        }
        .into())
    }
}
