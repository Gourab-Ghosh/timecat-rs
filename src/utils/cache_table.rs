use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CacheTableEntry<T> {
    hash: NonZeroU64, // To save space in cache table
    entry: T,
}

impl<T> CacheTableEntry<T> {
    #[inline]
    pub const fn new(hash: NonZeroU64, entry: T) -> Self {
        Self { hash, entry }
    }

    #[inline]
    pub fn get_hash(self) -> NonZeroU64 {
        self.hash
    }

    #[inline]
    pub fn get_entry(self) -> T {
        self.entry
    }

    #[inline]
    pub fn get_entry_mut(&mut self) -> &mut T {
        &mut self.entry
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheTableSize {
    Max(usize),
    Min(usize),
    Round(usize),
    Exact(usize),
}

impl CacheTableSize {
    pub const ZERO: Self = Self::Exact(0);

    pub const fn unwrap(self) -> usize {
        match self {
            Self::Max(size) => size,
            Self::Min(size) => size,
            Self::Round(size) => size,
            Self::Exact(size) => size,
        }
    }

    #[inline]
    pub const fn is_min(self) -> bool {
        matches!(self, Self::Min(_))
    }

    #[inline]
    pub const fn is_max(self) -> bool {
        matches!(self, Self::Max(_))
    }

    #[inline]
    pub const fn is_round(self) -> bool {
        matches!(self, Self::Round(_))
    }

    #[inline]
    pub const fn is_exact(self) -> bool {
        matches!(self, Self::Exact(_))
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    #[inline]
    pub const fn get_entry_size<T>() -> usize {
        size_of::<Option<CacheTableEntry<T>>>()
    }

    pub fn to_num_entries_and_entry_size<T>(self) -> (usize, usize) {
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

    #[inline]
    pub fn to_num_entries<T>(self) -> usize {
        self.to_num_entries_and_entry_size::<T>().0
    }

    #[inline]
    pub fn to_memory_size_in_mb<T>(self) -> usize {
        let (size, entry_size) = self.to_num_entries_and_entry_size::<T>();
        size * entry_size / 2_usize.pow(20)
    }
}

impl Default for CacheTableSize {
    fn default() -> Self {
        CACHE_TABLE_SIZE
    }
}

impl fmt::Display for CacheTableSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} MB", self.unwrap())
    }
}

#[cfg(feature = "extras")]
macro_rules! update_variables {
    ($self: ident, $e_copy: ident, $hash: ident, $entry: ident) => {
        if let Some(e) = $e_copy {
            if e.get_hash() == $hash {
                if e.get_entry() != $entry {
                    $self.num_overwrites.fetch_add(1, MEMORY_ORDERING);
                }
            } else {
                $self.num_collisions.fetch_add(1, MEMORY_ORDERING);
            }
        } else {
            $self.num_cells_filled.fetch_add(1, MEMORY_ORDERING);
        }
    };
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default)]
pub struct CacheTable<T> {
    table: RwLock<Box<[Option<CacheTableEntry<T>>]>>,
    size: RwLock<CacheTableSize>,
    mask: AtomicUsize,
    is_safe_to_do_bitwise_and: AtomicBool,
    num_cells_filled: AtomicUsize,
    #[cfg(feature = "extras")]
    num_overwrites: AtomicUsize,
    #[cfg(feature = "extras")]
    num_collisions: AtomicUsize,
    #[cfg(feature = "extras")]
    zero_hit: AtomicUsize,
}

impl<T: Copy + PartialEq> CacheTable<T> {
    #[inline]
    fn generate_table(size: CacheTableSize) -> Box<[Option<CacheTableEntry<T>>]> {
        vec![None; size.to_num_entries::<T>()].into_boxed_slice()
    }

    #[inline]
    const fn generate_mask(table_len: usize) -> (usize, bool) {
        // Is power of two and greater than 1
        let is_safe_to_do_bitwise_and = table_len.count_ones() == 1 && table_len > 1;
        let mask = if is_safe_to_do_bitwise_and {
            table_len - 1
        } else {
            table_len
        };
        (mask, is_safe_to_do_bitwise_and)
    }

    #[inline]
    fn reset_mask(&self) {
        let table_len = self.table.read().unwrap().len();
        let (mask, is_safe_to_do_bitwise_and) = Self::generate_mask(table_len);
        self.mask.store(mask, MEMORY_ORDERING);
        self.is_safe_to_do_bitwise_and
            .store(is_safe_to_do_bitwise_and, MEMORY_ORDERING);
    }

    pub fn new(size: CacheTableSize) -> Self {
        let cache_table = Self {
            table: RwLock::new(Self::generate_table(size)),
            size: RwLock::new(size),
            mask: Default::default(),
            is_safe_to_do_bitwise_and: Default::default(),
            num_cells_filled: AtomicUsize::new(0),
            #[cfg(feature = "extras")]
            num_overwrites: AtomicUsize::new(0),
            #[cfg(feature = "extras")]
            num_collisions: AtomicUsize::new(0),
            #[cfg(feature = "extras")]
            zero_hit: AtomicUsize::new(0),
        };
        cache_table.reset_mask();
        cache_table
    }

    #[inline]
    fn get_index(&self, hash: u64) -> usize {
        if self.is_safe_to_do_bitwise_and.load(MEMORY_ORDERING) {
            hash as usize & self.mask.load(MEMORY_ORDERING)
        } else {
            hash as usize % self.mask.load(MEMORY_ORDERING)
        }
    }

    #[inline]
    pub fn get(&self, hash: u64) -> Option<T> {
        let hash = NonZeroU64::new(hash).unwrap_or(DEFAULT_HASH);
        let entry = {
            let table = self.table.read().unwrap();
            *get_item_unchecked!(table, self.get_index(hash.get()))
        }?;
        (entry.hash == hash).then_some(entry.entry)
    }

    #[inline]
    pub fn add(&self, hash: u64, entry: T) {
        let hash = NonZeroU64::new(hash).unwrap_or(DEFAULT_HASH);
        let mut table = self.table.write().unwrap();
        let e = get_item_unchecked_mut!(table, self.get_index(hash.get()));
        #[cfg(not(feature = "extras"))]
        if e.is_none() {
            self.num_cells_filled.fetch_add(1, MEMORY_ORDERING);
        }
        #[cfg(feature = "extras")]
        let e_copy = *e;
        *e = Some(CacheTableEntry { hash, entry });
        drop(table);
        #[cfg(feature = "extras")]
        update_variables!(self, e_copy, hash, entry);
    }

    #[inline]
    pub fn replace_if<F: Fn(T) -> bool>(&self, hash: u64, entry: T, replace: F) {
        let hash = NonZeroU64::new(hash).unwrap_or(DEFAULT_HASH);
        let mut table = self.table.write().unwrap();
        let e = get_item_unchecked_mut!(table, self.get_index(hash.get()));
        let to_replace = if let Some(entry) = e {
            replace(entry.entry)
        } else {
            true
        };
        if to_replace {
            #[cfg(not(feature = "extras"))]
            if e.is_none() {
                self.num_cells_filled.fetch_add(1, MEMORY_ORDERING);
            }
            #[cfg(feature = "extras")]
            let e_copy = *e;
            *e = Some(CacheTableEntry { hash, entry });
            drop(table);
            #[cfg(feature = "extras")]
            update_variables!(self, e_copy, hash, entry);
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.table.write().unwrap().fill(None);
        self.num_cells_filled.store(0, MEMORY_ORDERING);
        self.reset_variables()
    }

    #[inline]
    pub const fn get_table(&self) -> &RwLock<Box<[Option<CacheTableEntry<T>>]>> {
        &self.table
    }

    #[inline]
    pub fn get_num_cells_filled(&self) -> usize {
        self.num_cells_filled.load(MEMORY_ORDERING)
    }

    #[inline]
    #[cfg(feature = "extras")]
    pub fn get_num_overwrites(&self) -> usize {
        self.num_overwrites.load(MEMORY_ORDERING)
    }

    #[inline]
    #[cfg(feature = "extras")]
    pub fn get_num_collisions(&self) -> usize {
        self.num_collisions.load(MEMORY_ORDERING)
    }

    #[inline]
    #[cfg(feature = "extras")]
    pub fn get_zero_hit(&self) -> usize {
        self.zero_hit.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn reset_num_cells_filled(&self) {
        self.num_cells_filled.store(0, MEMORY_ORDERING);
    }

    #[inline]
    #[cfg(feature = "extras")]
    pub fn reset_num_overwrites(&self) {
        self.num_overwrites.store(0, MEMORY_ORDERING);
    }

    #[inline]
    #[cfg(feature = "extras")]
    pub fn reset_num_collisions(&self) {
        self.num_collisions.store(0, MEMORY_ORDERING);
    }

    #[inline]
    #[cfg(feature = "extras")]
    pub fn reset_zero_hit(&self) {
        self.zero_hit.store(0, MEMORY_ORDERING);
    }

    /// Variable needed to be reset per search
    pub fn reset_variables(&self) {
        #[cfg(feature = "extras")]
        {
            self.reset_num_overwrites();
            self.reset_num_collisions();
            self.reset_zero_hit();
        }
    }

    #[inline]
    pub fn get_hash_full(&self) -> f64 {
        (self.num_cells_filled.load(MEMORY_ORDERING) as f64 / self.len() as f64) * 100.0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.table.read().unwrap().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get_num_cells_filled() == 0
    }

    #[inline]
    pub fn get_size(&self) -> CacheTableSize {
        *self.size.read().unwrap()
    }

    pub fn set_size(&self, size: CacheTableSize) {
        *self.size.write().unwrap() = size;
        let old_table = std::mem::replace(
            &mut *self.table.write().unwrap(),
            Self::generate_table(size),
        );
        self.reset_mask();
        self.reset_variables();
        old_table.into_iter().flatten().for_each(|entry| {
            self.add(entry.hash.get(), entry.entry);
        });
    }
}

impl<T: Copy + PartialEq> Clone for CacheTable<T> {
    fn clone(&self) -> Self {
        Self {
            table: RwLock::new(self.table.read().unwrap().clone()),
            size: RwLock::new(self.get_size()),
            mask: AtomicUsize::new(self.mask.load(MEMORY_ORDERING)),
            is_safe_to_do_bitwise_and: AtomicBool::new(
                self.is_safe_to_do_bitwise_and.load(MEMORY_ORDERING),
            ),
            num_cells_filled: AtomicUsize::new(self.num_cells_filled.load(MEMORY_ORDERING)),
            #[cfg(feature = "extras")]
            num_overwrites: AtomicUsize::new(self.num_overwrites.load(MEMORY_ORDERING)),
            #[cfg(feature = "extras")]
            num_collisions: AtomicUsize::new(self.num_collisions.load(MEMORY_ORDERING)),
            #[cfg(feature = "extras")]
            zero_hit: AtomicUsize::new(self.zero_hit.load(MEMORY_ORDERING)),
        }
    }
}
