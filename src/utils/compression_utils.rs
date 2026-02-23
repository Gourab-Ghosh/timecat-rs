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
    type CompressedItem = u8;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        self as Self::CompressedItem
    }
}

impl Compress for Move {
    type CompressedItem = u16;

    #[inline]
    fn compress(self) -> Self::CompressedItem {
        ((self.get_source() as Self::CompressedItem) << 6)
            ^ (self.get_dest() as Self::CompressedItem)
            ^ ((self.get_promotion().compress() as Self::CompressedItem) << 12)
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

impl Decompress<PieceType> for u8 {
    #[inline]
    fn decompress(self) -> Result<PieceType> {
        if self == 0 || self > NUM_PIECE_TYPES as Self {
            Err(TimecatError::DecompressionFailed {
                value: self.to_string().into(),
                type_name: std::any::type_name::<PieceType>().into(),
            })
        } else {
            Ok(unsafe { PieceType::from_int(self - 1) })
        }
    }
}

impl Decompress<Option<PieceType>> for u8 {
    #[inline]
    fn decompress(self) -> Result<Option<PieceType>> {
        if self == 0 {
            Ok(None)
        } else {
            Ok(Some(self.decompress()?))
        }
    }
}

impl Decompress<Square> for u8 {
    #[inline]
    fn decompress(self) -> Result<Square> {
        if self > NUM_SQUARES as Self {
            Err(TimecatError::DecompressionFailed {
                value: self.to_string().into(),
                type_name: std::any::type_name::<Square>().into(),
            })
        } else {
            Ok(unsafe { Square::from_int(self) })
        }
    }
}

impl Decompress<Move> for u16 {
    fn decompress(self) -> Result<Move> {
        Ok(unsafe {
            Move::new_unchecked(
                (((self >> 6) & 0x3F) as u8).decompress()?,
                ((self & 0x3F) as u8).decompress()?,
                ((self >> 12) as u8).decompress()?,
            )
        })
    }
}

impl Decompress<Option<Move>> for u16 {
    fn decompress(self) -> Result<Option<Move>> {
        if self == Self::MAX {
            Ok(None)
        } else {
            Ok(Some(self.decompress()?))
        }
    }
}

impl Decompress<ValidOrNullMove> for u16 {
    #[inline]
    fn decompress(self) -> Result<ValidOrNullMove> {
        Ok(<Self as Decompress<Option<Move>>>::decompress(self)?.into())
    }
}

// impl<T> Decompress<T> for CompressedObject where CompressedObject: Decompress<Option<T>> {
//     fn decompress(self) -> T {
//         self.decompress().unwrap_or_else(|| panic!("Failed to decompress"))
//     }
// }
