use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CacheTableEntry<T: Copy + Clone + PartialEq> {
    hash: u64,
    entry: T,
}

impl<T: Copy + Clone + PartialEq> CacheTableEntry<T> {
    #[inline]
    pub const fn new(hash: u64, entry: T) -> CacheTableEntry<T> {
        CacheTableEntry { hash, entry }
    }

    #[inline]
    pub const fn get_hash(self) -> u64 {
        self.hash
    }

    #[inline]
    pub const fn get_entry(self) -> T {
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

    #[inline]
    pub fn to_cache_table_size<T: Copy + Clone + PartialEq>(self) -> usize {
        self.to_cache_table_and_entry_size::<T>().0
    }

    #[inline]
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

#[cfg(any(feature = "debug", not(feature = "binary")))]
macro_rules! update_overwrites_and_collisions {
    ($self: ident, $e_hash: ident, $e_entry: ident, $hash: ident, $entry: ident) => {
        if $e_hash == 0 {
            $self.num_cells_filled.fetch_add(1, MEMORY_ORDERING);
        } else {
            if $e_hash == $hash {
                if $e_entry != $entry {
                    $self.num_overwrites.fetch_add(1, MEMORY_ORDERING);
                }
            } else {
                $self.num_collisions.fetch_add(1, MEMORY_ORDERING);
            }
        }
    };
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CacheTable<T: Copy + Clone + PartialEq> {
    table: RwLock<Box<[CacheTableEntry<T>]>>,
    size: RwLock<CacheTableSize>,
    default: T,
    mask: AtomicUsize,
    is_safe_to_do_bitwise_and: AtomicBool,
    num_overwrites: AtomicUsize,
    num_collisions: AtomicUsize,
    num_cells_filled: AtomicUsize,
    zero_hit: AtomicUsize,
}

impl<T: Copy + Clone + PartialEq> CacheTable<T> {
    #[inline]
    fn generate_table(size: CacheTableSize, default: T) -> Box<[CacheTableEntry<T>]> {
        vec![
            CacheTableEntry {
                hash: 0,
                entry: default
            };
            size.to_cache_table_size::<T>()
        ]
        .into_boxed_slice()
    }

    #[inline]
    const fn is_safe_to_do_bitwise_and(size: usize) -> bool {
        size.count_ones() == 1 && size > 1
    }

    #[inline]
    const fn get_mask(table: &[CacheTableEntry<T>]) -> usize {
        if Self::is_safe_to_do_bitwise_and(table.len()) {
            table.len() - 1
        } else {
            table.len()
        }
    }

    #[inline]
    fn reset_mask(&self, table: &[CacheTableEntry<T>]) {
        self.mask.store(Self::get_mask(table), MEMORY_ORDERING);
        self.is_safe_to_do_bitwise_and.store(
            Self::is_safe_to_do_bitwise_and(table.len()),
            MEMORY_ORDERING,
        );
    }

    pub fn new(size: CacheTableSize, default: T) -> CacheTable<T> {
        let cache_table = CacheTable {
            table: RwLock::new(Self::generate_table(size, default)),
            size: RwLock::new(size),
            default,
            mask: Default::default(),
            is_safe_to_do_bitwise_and: Default::default(),
            num_overwrites: AtomicUsize::new(0),
            num_collisions: AtomicUsize::new(0),
            num_cells_filled: AtomicUsize::new(0),
            zero_hit: AtomicUsize::new(0),
        };
        cache_table.reset_mask(&cache_table.table.read().unwrap());
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
        let entry = *get_item_unchecked!(self.table.read().unwrap(), self.get_index(hash));
        if entry.hash == hash {
            Some(entry.entry)
        } else {
            None
        }
    }

    #[inline]
    pub fn add(&self, mut hash: u64, entry: T) {
        hash = hash.max(1);
        let mut table = self.table.write().unwrap();
        let e = get_item_unchecked_mut!(table, self.get_index(hash));
        #[cfg(any(feature = "debug", not(feature = "binary")))]
        let e_hash = e.get_hash();
        #[cfg(any(feature = "debug", not(feature = "binary")))]
        let e_entry = e.get_entry();
        *e = CacheTableEntry { hash, entry };
        drop(table);
        #[cfg(any(feature = "debug", not(feature = "binary")))]
        update_overwrites_and_collisions!(self, e_hash, e_entry, hash, entry);
    }

    #[inline]
    pub fn replace_if<F: Fn(T) -> bool>(&self, mut hash: u64, entry: T, replace: F) {
        hash = hash.max(1);
        let mut table = self.table.write().unwrap();
        let e = get_item_unchecked_mut!(table, self.get_index(hash));
        if replace(e.entry) {
            #[cfg(any(feature = "debug", not(feature = "binary")))]
            let e_hash = e.get_hash();
            #[cfg(any(feature = "debug", not(feature = "binary")))]
            let e_entry = e.get_entry();
            *e = CacheTableEntry { hash, entry };
            drop(table);
            #[cfg(any(feature = "debug", not(feature = "binary")))]
            update_overwrites_and_collisions!(self, e_hash, e_entry, hash, entry);
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.table
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|e| e.hash = 0);
        self.num_cells_filled.store(0, MEMORY_ORDERING);
        self.reset_variables()
    }

    #[inline]
    pub const fn get_table(&self) -> &RwLock<Box<[CacheTableEntry<T>]>> {
        &self.table
    }

    #[inline]
    pub fn get_num_overwrites(&self) -> usize {
        self.num_overwrites.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_num_collisions(&self) -> usize {
        self.num_collisions.load(MEMORY_ORDERING)
    }
    #[inline]
    pub fn get_num_cells_filled(&self) -> usize {
        self.num_cells_filled.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_zero_hit(&self) -> usize {
        self.zero_hit.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn reset_num_overwrites(&self) {
        self.num_overwrites.store(0, MEMORY_ORDERING);
    }

    #[inline]
    pub fn reset_num_collisions(&self) {
        self.num_collisions.store(0, MEMORY_ORDERING);
    }

    #[inline]
    pub fn reset_num_cells_filled(&self) {
        self.num_cells_filled.store(0, MEMORY_ORDERING);
    }

    #[inline]
    pub fn reset_zero_hit(&self) {
        self.zero_hit.store(0, MEMORY_ORDERING);
    }

    /// Variable needed to be reset per search
    pub fn reset_variables(&self) {
        self.reset_num_overwrites();
        self.reset_num_collisions();
        self.reset_zero_hit();
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
        let current_table_copy = self.table.read().unwrap().clone();
        *self.table.write().unwrap() = Self::generate_table(size, self.default);
        self.reset_mask(&current_table_copy);
        self.reset_variables();
        for &CacheTableEntry { hash, entry } in current_table_copy.iter() {
            if hash != 0 {
                self.add(hash, entry);
            }
        }
    }
}

impl<T: Copy + Clone + PartialEq> Clone for CacheTable<T> {
    fn clone(&self) -> Self {
        CacheTable {
            table: RwLock::new(self.table.read().unwrap().clone()),
            size: RwLock::new(self.get_size()),
            default: self.default,
            mask: AtomicUsize::new(self.mask.load(MEMORY_ORDERING)),
            is_safe_to_do_bitwise_and: AtomicBool::new(
                self.is_safe_to_do_bitwise_and.load(MEMORY_ORDERING),
            ),
            num_overwrites: AtomicUsize::new(self.num_overwrites.load(MEMORY_ORDERING)),
            num_collisions: AtomicUsize::new(self.num_collisions.load(MEMORY_ORDERING)),
            num_cells_filled: AtomicUsize::new(self.num_cells_filled.load(MEMORY_ORDERING)),
            zero_hit: AtomicUsize::new(self.zero_hit.load(MEMORY_ORDERING)),
        }
    }
}
