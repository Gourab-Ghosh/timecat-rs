use super::*;

impl Compress for Option<PieceType> {
    type CompressedItem = u8;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        self.map_or(0, |piece| piece as Self::CompressedItem + 1)
    }
}

impl Compress for PieceType {
    type CompressedItem = u8;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        Some(self).compress()
    }
}

impl Compress for Square {
    type CompressedItem = u16;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        self.to_index() as Self::CompressedItem
    }
}

impl Compress for Move {
    type CompressedItem = u16;

    fn compress(self) -> Self::CompressedItem {
        let mut compressed_move = 0;
        compressed_move ^= self.get_source().compress() << 6;
        compressed_move ^= self.get_dest().compress();
        compressed_move ^= (self.get_promotion().compress() as Self::CompressedItem) << 12;
        compressed_move
    }
}

impl Compress for Option<Move> {
    type CompressedItem = u16;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        self.map_or(Self::CompressedItem::MAX, |m| m.compress())
    }
}

impl Compress for ValidOrNullMove {
    type CompressedItem = u16;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        (*self).compress()
    }
}

impl Decompress<Option<PieceType>> for u8 {
    #[inline]
    fn decompress(self) -> Option<PieceType> {
        if self == 0 {
            return None;
        }
        Some(*get_item_unchecked!(ALL_PIECE_TYPES, (self - 1) as usize))
    }
}

impl Decompress<Option<PieceType>> for u16 {
    #[inline]
    fn decompress(self) -> Option<PieceType> {
        (self as u8).decompress()
    }
}

impl Decompress<Square> for u16 {
    #[inline]
    fn decompress(self) -> Square {
        *get_item_unchecked!(ALL_SQUARES, self as usize)
    }
}

impl Decompress<Option<Move>> for u16 {
    fn decompress(self) -> Option<Move> {
        if self == u16::MAX {
            return None;
        }
        Some(Move::new_unchecked(
            (self >> 6 & 0x3F).decompress(),
            (self & 0x3F).decompress(),
            (self >> 12).decompress(),
        ))
    }
}

impl Decompress<ValidOrNullMove> for u16 {
    #[inline]
    fn decompress(self) -> ValidOrNullMove {
        <Self as Decompress<Option<Move>>>::decompress(self).into()
    }
}

// impl<T> Decompress<T> for CompressedObject where CompressedObject: Decompress<Option<T>> {
//     fn decompress(self) -> T {
//         self.decompress().unwrap_or_else(|| panic!("Failed to decompress"))
//     }
// }
