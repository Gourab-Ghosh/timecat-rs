use super::*;
pub use Color::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn to_my_backrank(self) -> Rank {
        *get_item_unchecked!(const [Rank::First, Rank::Eighth], self.to_index())
    }

    #[inline]
    pub fn to_their_backrank(self) -> Rank {
        *get_item_unchecked!(const [Rank::Eighth, Rank::First], self.to_index())
    }

    #[inline]
    pub fn to_second_rank(self) -> Rank {
        *get_item_unchecked!(const [Rank::Second, Rank::Seventh], self.to_index())
    }

    #[inline]
    pub fn to_third_rank(self) -> Rank {
        *get_item_unchecked!(const [Rank::Third, Rank::Sixth], self.to_index())
    }

    #[inline]
    pub fn to_fourth_rank(self) -> Rank {
        *get_item_unchecked!(const [Rank::Fourth, Rank::Fifth], self.to_index())
    }

    #[inline]
    pub fn to_seventh_rank(self) -> Rank {
        *get_item_unchecked!(const [Rank::Seventh, Rank::Second], self.to_index())
    }
}

impl Not for Color {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        *get_item_unchecked!(const [Black, White], self.to_index())
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            get_item_unchecked!(const ["White", "Black"], self.to_index())
        )
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
        if let Ok(s) = ob.extract::<&str>() {
            if s.eq_ignore_ascii_case("white") {
                return Ok(Self::White);
            }
            if s.eq_ignore_ascii_case("black") {
                return Ok(Self::Black);
            }
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
