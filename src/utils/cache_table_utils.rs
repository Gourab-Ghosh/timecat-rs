use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheTableSize {
    Max(usize),
    Min(usize),
    Round(usize),
    Exact(usize),
}

impl CacheTableSize {
    pub fn unwrap(&self) -> usize {
        match self {
            Self::Max(size) => *size,
            Self::Min(size) => *size,
            Self::Round(size) => *size,
            Self::Exact(size) => *size,
        }
    }

    #[inline(always)]
    pub fn is_min(&self) -> bool {
        matches!(self, Self::Min(_))
    }

    #[inline(always)]
    pub fn is_max(&self) -> bool {
        matches!(self, Self::Max(_))
    }

    #[inline(always)]
    pub fn is_round(&self) -> bool {
        matches!(self, Self::Round(_))
    }
    #[inline(always)]
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact(_))
    }

    pub const fn get_entry_size<T: Copy + Clone + PartialEq>() -> usize {
        std::mem::size_of::<CacheTableEntry<T>>()
    }

    pub fn to_cache_table_and_entry_size<T: Copy + Clone + PartialEq>(self) -> (usize, usize) {
        let mut size = self.unwrap();
        let entry_size = Self::get_entry_size::<T>();
        size *= 2_usize.pow(20);
        size /= entry_size;
        if self.is_exact() {
            return (size, entry_size);
        }
        let pow_f64 = (size as f64).log2();
        let pow = match self {
            Self::Max(_) => pow_f64.floor(),
            Self::Min(_) => pow_f64.ceil(),
            Self::Round(_) => pow_f64.round(),
            Self::Exact(_) => unreachable!(),
        } as u32;
        size = 2_usize.pow(pow);
        (size, entry_size)
    }

    #[inline(always)]
    pub fn to_cache_table_size<T: Copy + Clone + PartialEq>(self) -> usize {
        self.to_cache_table_and_entry_size::<T>().0
    }

    pub fn to_cache_table_memory_size<T: Copy + Clone + PartialEq>(self) -> usize {
        let (size, entry_size) = self.to_cache_table_and_entry_size::<T>();
        size * entry_size / 2_usize.pow(20)
    }
}

impl fmt::Display for CacheTableSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} MB", self.unwrap())
    }
}
