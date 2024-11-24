use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BinaryMagic<T> {
    architecture: T,
}

impl<T> Deref for BinaryMagic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.architecture
    }
}

impl<T: Debug> Debug for BinaryMagic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.architecture)
    }
}

impl<T: BinRead<Args = ()> + Copy + PartialEq + Send + Sync + 'static> BinRead for BinaryMagic<T> {
    type Args = (T,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        (magic,): Self::Args,
    ) -> BinResult<Self> {
        let architecture = BinRead::read_options(reader, options, ())?;
        if architecture == magic {
            Ok(Self { architecture })
        } else {
            Err(binread::Error::BadMagic {
                pos: reader.stream_position()?,
                found: Box::new(architecture),
            })
        }
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for BinaryMagic<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.architecture.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for BinaryMagic<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            architecture: T::deserialize(deserializer)?,
        })
    }
}
