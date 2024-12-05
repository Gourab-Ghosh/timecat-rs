use super::*;
use std::io::{Read, Seek};

#[inline]
pub fn get_move_from_polyglot_move_int(move_int: u16) -> Result<Move> {
    Move::new(
        (move_int & 0x3F).decompress(),
        (move_int >> 6 & 0x3F).decompress(),
        *const { [None, Some(Knight), Some(Bishop), Some(Rook), Some(Queen)] }
            .get((move_int >> 12) as usize)
            .ok_or("Invalid promotion piece")?,
    )
}

pub mod array_implementation {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct PolyglotBookEntry {
        hash: u64,
        move_: Move,
        weight: u16,
        learn: u32,
    }

    impl PolyglotBookEntry {
        fn get_unique_key(&self) -> (u64, u16) {
            (self.hash, u16::MAX - self.weight)
        }

        fn get_weighted_move(&self) -> WeightedMove {
            WeightedMove {
                move_: self.move_,
                weight: self.weight as MoveWeight,
            }
        }
    }

    impl PartialOrd for PolyglotBookEntry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some((self.get_unique_key()).cmp(&other.get_unique_key()))
        }
    }

    impl Ord for PolyglotBookEntry {
        fn cmp(&self, other: &Self) -> Ordering {
            self.get_unique_key().cmp(&other.get_unique_key())
        }
    }

    #[derive(Clone, Debug)]
    pub struct PolyglotBook {
        entries: Vec<PolyglotBookEntry>,
    }

    impl PolyglotBook {
        pub fn sort_book(&mut self) {
            self.entries.sort_unstable();
        }

        pub fn len(&self) -> usize {
            self.entries.len()
        }

        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }

        pub fn get_all_weighted_moves_with_hashes(&self) -> Vec<(u64, WeightedMove)> {
            self.entries
                .iter()
                .map(|entry| (entry.hash, entry.get_weighted_move()))
                .collect()
        }

        pub fn get_all_weighed_moves(&self, board: &Board) -> Vec<WeightedMove> {
            //TODO: optimize
            let hash = board.get_hash();
            let index = self
                .entries
                .binary_search_by(|entry| entry.hash.cmp(&hash))
                .ok();
            let mut moves = Vec::new();
            if let Some(index) = index {
                let mut initial_index = index;
                let mut final_index = index;
                while initial_index > 0 && self.entries[initial_index - 1].hash == hash {
                    initial_index -= 1;
                }
                while final_index < self.entries.len() && self.entries[final_index].hash == hash {
                    final_index += 1;
                }
                for entry in &self.entries[initial_index..final_index] {
                    moves.push(entry.get_weighted_move());
                }
            }
            moves
        }

        pub fn get_best_move(&self, board: &Board) -> Option<Move> {
            //TODO: optimize
            let hash = board.get_hash();
            let index = self
                .entries
                .binary_search_by(|entry| entry.hash.cmp(&hash))
                .ok();
            if let Some(mut index) = index {
                while index > 0 && self.entries[index - 1].hash == hash {
                    index -= 1;
                }
                return Some(self.entries[index].move_);
            }
            None
        }
    }

    #[derive(Debug)]
    pub struct PolyglotBookReader;

    impl PolyglotBookReader {
        fn entries_are_sorted(entries: &[PolyglotBookEntry]) -> bool {
            entries.windows(2).all(|w| w[0] <= w[1])
        }

        fn get_entries_from_bytes(bytes: &[u8]) -> Result<Vec<PolyglotBookEntry>> {
            let mut entries = Vec::new();
            let mut offset = 0;
            while offset < bytes.len() {
                let hash = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap());
                let move_int =
                    u16::from_be_bytes(bytes[offset + 8..offset + 10].try_into().unwrap());
                let weight =
                    u16::from_be_bytes(bytes[offset + 10..offset + 12].try_into().unwrap());
                let learn = u32::from_be_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
                entries.push(PolyglotBookEntry {
                    hash,
                    move_: get_move_from_polyglot_move_int(move_int)?,
                    weight,
                    learn,
                });
                offset += 16;
            }
            if !Self::entries_are_sorted(&entries) {
                entries.sort_unstable();
            }
            Ok(entries)
        }

        pub fn read_book_from_file(path: &str) -> Result<PolyglotBook> {
            let mut entries = Vec::new();
            let mut file = fs::File::open(path)?;
            let mut buffer = [0; 16];
            while file.read_exact(&mut buffer).is_ok() {
                let hash = u64::from_be_bytes(buffer[0..8].try_into()?);
                let move_int = u16::from_be_bytes(buffer[8..10].try_into()?);
                let weight = u16::from_be_bytes(buffer[10..12].try_into()?);
                let learn = u32::from_be_bytes(buffer[12..16].try_into()?);
                entries.push(PolyglotBookEntry {
                    hash,
                    move_: get_move_from_polyglot_move_int(move_int)?,
                    weight,
                    learn,
                });
            }
            if !Self::entries_are_sorted(&entries) {
                entries.sort_unstable();
            }
            Ok(PolyglotBook { entries })
        }

        pub fn read_book_from_bytes(bytes: &[u8]) -> Result<PolyglotBook> {
            Ok(PolyglotBook {
                entries: Self::get_entries_from_bytes(bytes)?,
            })
        }
    }
}

pub mod map_implementation {
    use super::*;

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
    }

    impl PartialOrd for PolyglotBookEntry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for PolyglotBookEntry {
        fn cmp(&self, other: &Self) -> Ordering {
            other.weight.cmp(&self.weight)
        }
    }

    #[derive(Clone, Debug)]
    pub struct PolyglotBook {
        // start_index: usize,
        entries_map: IdentityHashMap<u64, Vec<PolyglotBookEntry>>,
    }

    impl PolyglotBook {
        pub fn sort_book(&mut self) {
            self.entries_map
                .values_mut()
                .for_each(|entries| entries.sort_unstable());
        }

        pub fn len(&self) -> usize {
            self.entries_map.len()
        }

        pub fn is_empty(&self) -> bool {
            self.entries_map.is_empty()
        }

        pub fn get_all_weighted_moves_with_hashes(
            &self,
        ) -> IdentityHashMap<u64, Vec<WeightedMove>> {
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
            let hash = board.get_hash();
            self.entries_map
                .get(&hash)
                .map(|entries| {
                    entries
                        .iter()
                        .map(|entry| entry.get_weighted_move())
                        .collect_vec()
                })
                .unwrap_or_default()
        }

        pub fn get_best_move(&self, board: &Board) -> Option<Move> {
            //TODO: optimize
            let hash = board.get_hash();
            self.entries_map
                .get(&hash)
                .and_then(|entries| entries.first().map(|entry| entry.move_))
        }
    }

    #[derive(Debug)]
    pub struct PolyglotBookReader;

    impl PolyglotBookReader {
        fn entries_are_sorted(entries: &[PolyglotBookEntry]) -> bool {
            entries.windows(2).all(|w| w[0] <= w[1])
        }

        fn get_entries_from_bytes(
            bytes: &[u8],
        ) -> Result<IdentityHashMap<u64, Vec<PolyglotBookEntry>>> {
            let mut entries_map = IdentityHashMap::default();
            let mut offset = 0;
            while offset < bytes.len() {
                let hash = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap());
                let move_int =
                    u16::from_be_bytes(bytes[offset + 8..offset + 10].try_into().unwrap());
                let weight =
                    u16::from_be_bytes(bytes[offset + 10..offset + 12].try_into().unwrap());
                let learn = u32::from_be_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
                let entry = PolyglotBookEntry {
                    move_: get_move_from_polyglot_move_int(move_int)?,
                    weight,
                    learn,
                };
                entries_map.entry(hash).or_insert_with(Vec::new).push(entry);
                offset += 16;
            }
            for entries in entries_map.values_mut() {
                if !Self::entries_are_sorted(entries) {
                    entries.sort_unstable();
                }
            }
            Ok(entries_map)
        }

        pub fn read_book_from_file(path: &str) -> Result<PolyglotBook> {
            let mut entries_map = IdentityHashMap::default();
            let mut file = fs::File::open(path)?;
            let mut buffer = [0; 16];
            while file.read_exact(&mut buffer).is_ok() {
                let hash = u64::from_be_bytes(buffer[0..8].try_into()?);
                let move_int = u16::from_be_bytes(buffer[8..10].try_into()?);
                let weight = u16::from_be_bytes(buffer[10..12].try_into()?);
                let learn = u32::from_be_bytes(buffer[12..16].try_into()?);
                let entry = PolyglotBookEntry {
                    move_: get_move_from_polyglot_move_int(move_int)?,
                    weight,
                    learn,
                };
                entries_map.entry(hash).or_insert_with(Vec::new).push(entry);
            }
            for entries in entries_map.values_mut() {
                if !Self::entries_are_sorted(entries) {
                    entries.sort_unstable();
                }
            }
            Ok(PolyglotBook { entries_map })
        }

        pub fn read_book_from_bytes(bytes: &[u8]) -> Result<PolyglotBook> {
            Ok(PolyglotBook {
                entries_map: Self::get_entries_from_bytes(bytes)?,
            })
        }
    }
}

fn read_bytes_at_offset(file: &fs::File, buffer: &mut [u8], offset: u64) -> std::io::Result<()> {
    let mut reader = std::io::BufReader::new(file);
    reader.seek(std::io::SeekFrom::Start(offset))?;
    reader.read_exact(buffer)
}

pub fn find_first_matching_index(file: &fs::File, target_hash: u64) -> Result<Option<u64>> {
    let mut buffer = [0; 16];
    let mut start = 0;
    let mut end = file.metadata()?.len() / 16 - 1;
    let mut first_match_idx = None;
    while start <= end {
        let mid = start + (end - start) / 2;
        let offset = mid * 16;
        let read_result = read_bytes_at_offset(file, &mut buffer, offset);
        if read_result.is_err() {
            break;
        }
        let hash = u64::from_be_bytes(buffer[0..8].try_into()?);
        match hash.cmp(&target_hash) {
            Ordering::Equal => {
                first_match_idx = Some(mid);
                end = mid - 1;
            }
            Ordering::Less => start = mid + 1,
            Ordering::Greater => end = mid - 1,
        }
    }
    Ok(first_match_idx)
}

pub fn search_all_moves_from_file(path: &str, board: &Board) -> Result<Vec<WeightedMove>> {
    let target_hash = board.get_hash();
    let file = fs::File::open(path)?;
    let mut buffer = [0; 16];
    let mut moves = Vec::new();
    if let Some(first_match_idx) = find_first_matching_index(&file, target_hash)? {
        // Gather all moves with matching hash
        let mut idx = first_match_idx;
        loop {
            let offset = idx * 16;
            let read_result = read_bytes_at_offset(&file, &mut buffer, offset);
            if read_result.is_err() {
                break;
            }
            let hash = u64::from_be_bytes(buffer[0..8].try_into()?);
            let move_int = u16::from_be_bytes(buffer[8..10].try_into()?);
            let weight = u16::from_be_bytes(buffer[10..12].try_into()?);
            if hash == target_hash {
                let valid_or_null_move = get_move_from_polyglot_move_int(move_int)?;
                moves.push(WeightedMove::new(valid_or_null_move, weight as MoveWeight));
                idx += 1;
            } else {
                break;
            }
        }
        moves.sort_unstable_by_key(|wm| Reverse(wm.weight));
    }
    Ok(moves)
}

pub fn search_best_moves_from_file(path: &str, board: &Board) -> Result<Option<Move>> {
    Ok(search_all_moves_from_file(path, board)?
        .first()
        .map(|wm| wm.move_))
}

pub fn test_polyglot(book_path: &str) -> Result<()> {
    // let board = Board::from_fen(STARTING_POSITION_FEN)?;
    let board = Board::from_fen("rnbqkbnr/ppp2ppp/4p3/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3")?;
    let book = array_implementation::PolyglotBookReader::read_book_from_file(book_path)?;
    let moves1 = book.get_all_weighed_moves(&board);
    println_wasm!("{}", moves1.stringify());
    let moves2 = search_all_moves_from_file(book_path, &board)?;
    println_wasm!("{}", moves2.stringify());
    assert_eq!(moves1, moves2);
    Ok(())
}
