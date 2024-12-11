use super::*;

#[inline]
fn polyglot_move_int_to_move(move_int: u16) -> Result<Move> {
    Move::new(
        (move_int >> 6 & 0x3F).decompress(),
        (move_int & 0x3F).decompress(),
        *const { [None, Some(Knight), Some(Bishop), Some(Rook), Some(Queen)] }
            .get((move_int >> 12) as usize)
            .ok_or(TimecatError::BadPolyglotFile)?,
    )
}

#[inline]
fn move_to_polyglot_move_int(move_: Move) -> Result<u16> {
    let mut move_int = match move_.get_promotion() {
        None => 0,
        Some(Knight) => 1,
        Some(Bishop) => 2,
        Some(Rook) => 3,
        Some(Queen) => 4,
        _ => return Err(TimecatError::PolyglotTableParseError),
    };
    move_int = move_int << 6 ^ move_.get_source().compress();
    move_int = move_int << 6 ^ move_.get_dest().compress();
    Ok(move_int)
}

#[derive(Clone)]
pub struct PolyglotBookReader {
    file: Arc<fs::File>,
}

impl PolyglotBookReader {
    pub fn from_file_path(file_path: &str) -> Result<Self> {
        Ok(Self::new(Arc::new(fs::File::open(file_path)?)))
    }

    pub const fn new(file: Arc<fs::File>) -> Self {
        Self { file }
    }

    fn read_bytes_at_offset(
        reader: &mut BufReader<Arc<fs::File>>,
        buffer: &mut [u8],
        offset: u64,
    ) -> std::io::Result<()> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        reader.read_exact(buffer)
    }

    fn find_first_matching_index(&self, target_hash: u64) -> Result<Option<u64>> {
        let mut reader = std::io::BufReader::new(self.file.clone());
        let mut buffer = [0; 16];
        let mut start = 0;
        let mut end = (self.file.metadata()?.len() / 16)
            .checked_sub(1)
            .ok_or(TimecatError::BadPolyglotFile)?;
        let mut first_match_idx = None;
        while start <= end {
            let mid = start + (end - start) / 2;
            if Self::read_bytes_at_offset(&mut reader, &mut buffer, mid << 4).is_err() {
                break;
            }
            let hash = u64::from_be_bytes(
                buffer[0..8]
                    .try_into()
                    .map_err(|_| TimecatError::BadPolyglotFile)?,
            );
            match hash.cmp(&target_hash) {
                Ordering::Equal => {
                    first_match_idx = Some(mid);
                    end = mid.checked_sub(1).ok_or(TimecatError::BadPolyglotFile)?;
                }
                Ordering::Less => {
                    start = mid.checked_add(1).ok_or(TimecatError::BadPolyglotFile)?;
                }
                Ordering::Greater => {
                    end = mid.checked_sub(1).ok_or(TimecatError::BadPolyglotFile)?;
                }
            }
        }
        Ok(first_match_idx)
    }

    pub fn get_all_weighted_moves(&self, board: &Board) -> Result<Vec<WeightedMove>> {
        let target_hash = board.get_hash();
        let mut reader = std::io::BufReader::new(self.file.clone());
        let mut buffer = [0; 16];
        let mut moves = Vec::new();
        if let Some(first_match_idx) = self.find_first_matching_index(target_hash)? {
            // Gather all moves with matching hash
            let mut idx = first_match_idx;
            loop {
                let offset = idx << 4;
                let read_result = Self::read_bytes_at_offset(&mut reader, &mut buffer, offset);
                if read_result.is_err() {
                    break;
                }
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
                if hash == target_hash {
                    let valid_or_null_move = polyglot_move_int_to_move(move_int)?;
                    moves.push(WeightedMove::new(valid_or_null_move, weight as MoveWeight));
                    idx += 1;
                } else {
                    break;
                }
            }
        }
        Ok(moves)
    }

    #[inline]
    pub fn to_polyglot_hashmap(&self) -> Result<PolyglotBookHashMap> {
        self.try_into()
    }
}

impl FromStr for PolyglotBookReader {
    type Err = TimecatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_file_path(s)
    }
}

impl PolyglotBook for PolyglotBookReader {
    #[inline]
    fn read_from_path(book_path: &str) -> Result<Self> {
        Self::from_str(book_path)
    }

    fn get_best_weighted_move(&self, board: &Board) -> Option<WeightedMove> {
        self.find_first_matching_index(board.get_hash())
            .ok()?
            .map(|index| -> Result<_> {
                let mut buffer = [0; 16];
                Self::read_bytes_at_offset(
                    &mut std::io::BufReader::new(self.file.clone()),
                    &mut buffer,
                    index << 4,
                )?;
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
                let valid_or_null_move = polyglot_move_int_to_move(move_int)?;
                Ok(WeightedMove::new(valid_or_null_move, weight as MoveWeight))
            })
            .transpose()
            .ok()
            .flatten()
    }
}

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

    pub fn get_all_weighed_moves(&self, board: &Board) -> Vec<WeightedMove> {
        //TODO: optimize
        self.entries_map
            .get(&board.get_hash())
            .map(|entries| {
                entries
                    .iter()
                    .map(|entry| entry.get_weighted_move())
                    .collect_vec()
            })
            .unwrap_or_default()
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let mut data = self
            .entries_map
            .iter()
            .flat_map(|(hash, entries)| entries.iter().map(move |entry| (hash, entry)))
            .collect_vec();
        data.sort_unstable_by_key(|(&hash, entry)| (hash, Reverse(entry.weight)));
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
        //TODO: optimize
        self.entries_map
            .get(&board.get_hash())
            .and_then(|entries| entries.first().map(|entry| entry.get_weighted_move()))
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

impl TryFrom<&PolyglotBookReader> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: &PolyglotBookReader) -> std::result::Result<Self, Self::Error> {
        value.file.try_clone().map_or_else(
            |_| {
                value
                    .file
                    .clone()
                    .bytes()
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .try_into()
            },
            |file| file.try_into(),
        )
    }
}

impl TryFrom<PolyglotBookReader> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: PolyglotBookReader) -> std::result::Result<Self, Self::Error> {
        (&value).try_into()
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
