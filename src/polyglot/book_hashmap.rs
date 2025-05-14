use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PolyglotBookEntry {
    move_: Move,
    weight: u16,
    learn: u32,
}

impl PolyglotBookEntry {
    fn get_weighted_move(&self) -> WeightedMove {
        WeightedMove::new(self.move_, self.weight as MoveWeight)
    }

    fn write_to_file(&self, file: &mut fs::File) -> Result<()> {
        file.write_all(&move_to_polyglot_move_int(self.move_)?.to_be_bytes())?;
        file.write_all(&self.weight.to_be_bytes())?;
        file.write_all(&self.learn.to_be_bytes())?;
        Ok(())
    }
}

impl TryFrom<[u8; 8]> for PolyglotBookEntry {
    type Error = TimecatError;

    fn try_from(value: [u8; 8]) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            move_: polyglot_move_int_to_move(u16::from_be_bytes(
                value[0..2]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            ))?,
            weight: u16::from_be_bytes(
                value[2..4]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            ),
            learn: u32::from_be_bytes(
                value[4..8]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            ),
        })
    }
}

impl TryFrom<&[u8]> for PolyglotBookEntry {
    type Error = TimecatError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        <[u8; 8]>::try_from(value)?.try_into()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct PolyglotBookHashMap {
    entries_map: IdentityHashMap<u64, Vec<PolyglotBookEntry>>,
}

impl PolyglotBookHashMap {
    #[inline]
    pub fn sort_book(&mut self) {
        self.entries_map
            .values_mut()
            .for_each(|entries| entries.sort_unstable_by_key(|key| Reverse(key.weight)));
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries_map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries_map.is_empty()
    }

    pub fn get_all_weighted_moves_with_hashes(&self) -> IdentityHashMap<u64, Vec<WeightedMove>> {
        self.entries_map
            .iter()
            .map(|(&hash, entries)| {
                (
                    hash,
                    entries
                        .iter()
                        .map(|entry| entry.get_weighted_move())
                        .collect_vec(),
                )
            })
            .collect()
    }

    pub fn get_all_weighted_moves(&self, board: &Board) -> Vec<WeightedMove> {
        self.entries_map
            .get(&board.get_hash())
            .map_or(const { Vec::new() }, |entries| {
                let mut weighted_moves = Vec::with_capacity(entries.len());
                entries
                    .iter()
                    .map(|entry| entry.get_weighted_move())
                    .for_each(|weighted_move| weighted_moves.push(weighted_move));
                weighted_moves
            })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let mut data = self
            .entries_map
            .iter()
            .flat_map(|(hash, entries)| entries.iter().map(move |entry| (hash, entry)))
            .collect_vec();
        data.sort_unstable_by_key(|&(&hash, entry)| (hash, Reverse(entry.weight)));
        let mut file = fs::File::create(file_path)?;
        for (hash, entry) in data {
            file.write_all(&hash.to_be_bytes())?;
            entry.write_to_file(&mut file)?;
        }
        Ok(())
    }
}

impl PolyglotBook for PolyglotBookHashMap {
    #[inline]
    fn read_from_path(book_path: &str) -> Result<Self> {
        Self::from_str(book_path)
    }

    #[inline]
    fn get_best_weighted_move(&self, board: &Board) -> Option<WeightedMove> {
        Some(
            self.entries_map
                .get(&board.get_hash())?
                .first()?
                .get_weighted_move(),
        )
    }
}

impl TryFrom<&[u8]> for PolyglotBookHashMap {
    type Error = TimecatError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let mut entries_map = IdentityHashMap::default();
        let mut offset = 0;
        while offset < value.len() {
            let hash = u64::from_be_bytes(
                value[offset..offset + 8]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            );
            entries_map
                .entry(hash)
                .or_insert_with(Vec::new)
                .push(value[offset + 8..offset + 16].try_into()?);
            offset += 16;
        }
        Ok(Self { entries_map })
    }
}

impl TryFrom<Vec<u8>> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: [u8; N]) -> std::result::Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl TryFrom<&mut fs::File> for PolyglotBookHashMap {
    type Error = TimecatError;

    fn try_from(value: &mut fs::File) -> std::result::Result<Self, Self::Error> {
        let mut entries_map = IdentityHashMap::default();
        let mut buffer = [0; 16];
        while value.read_exact(&mut buffer).is_ok() {
            let hash = u64::from_be_bytes(
                buffer[0..8]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            );
            let move_int = u16::from_be_bytes(
                buffer[8..10]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            );
            let weight = u16::from_be_bytes(
                buffer[10..12]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            );
            let learn = u32::from_be_bytes(
                buffer[12..16]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            );
            let entry = PolyglotBookEntry {
                move_: polyglot_move_int_to_move(move_int)?,
                weight,
                learn,
            };
            entries_map.entry(hash).or_insert_with(Vec::new).push(entry);
        }
        Ok(Self { entries_map })
    }
}

impl TryFrom<fs::File> for PolyglotBookHashMap {
    type Error = TimecatError;

    fn try_from(mut value: fs::File) -> std::result::Result<Self, Self::Error> {
        (&mut value).try_into()
    }
}

impl FromStr for PolyglotBookHashMap {
    type Err = TimecatError;

    #[inline]
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        fs::File::open(s)?.try_into()
    }
}

impl TryFrom<String> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
