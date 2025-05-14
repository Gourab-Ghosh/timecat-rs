use super::*;

#[derive(Clone)]
pub struct PolyglotBookMemoryMappedReader {
    file: Arc<fs::File>,
}

impl PolyglotBookMemoryMappedReader {
    pub fn from_file_path(file_path: &str) -> Result<Self> {
        Ok(Self::new(Arc::new(fs::File::open(file_path)?)))
    }

    pub const fn new(file: Arc<fs::File>) -> Self {
        Self { file }
    }

    pub fn get_file(&self) -> Arc<fs::File> {
        self.file.clone()
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
        // TODO: Check for error handling logic
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

impl FromStr for PolyglotBookMemoryMappedReader {
    type Err = TimecatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_file_path(s)
    }
}

impl PolyglotBook for PolyglotBookMemoryMappedReader {
    #[inline]
    fn read_from_path(book_path: &str) -> Result<Self> {
        Self::from_str(book_path)
    }

    fn get_best_weighted_move(&self, board: &Board) -> Option<WeightedMove> {
        let mut buffer = [0; 16];
        Self::read_bytes_at_offset(
            &mut std::io::BufReader::new(self.file.clone()),
            &mut buffer,
            self.find_first_matching_index(board.get_hash()).ok()?? << 4,
        )
        .ok()?;

        let move_int = u16::from_be_bytes(buffer[8..10].try_into().ok()?);
        let weight = u16::from_be_bytes(buffer[10..12].try_into().ok()?);
        let valid_or_null_move = polyglot_move_int_to_move(move_int).ok()?;

        Some(WeightedMove::new(valid_or_null_move, weight as MoveWeight))
    }
}

impl TryFrom<&PolyglotBookMemoryMappedReader> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: &PolyglotBookMemoryMappedReader) -> std::result::Result<Self, Self::Error> {
        value.get_file().try_clone().map_or_else(
            |_| {
                value
                    .get_file()
                    .bytes()
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .try_into()
            },
            |file| file.try_into(),
        )
    }
}

impl TryFrom<PolyglotBookMemoryMappedReader> for PolyglotBookHashMap {
    type Error = TimecatError;

    #[inline]
    fn try_from(value: PolyglotBookMemoryMappedReader) -> std::result::Result<Self, Self::Error> {
        (&value).try_into()
    }
}
