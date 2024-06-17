use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Magic<T> {
    architecture: T,
}

impl<T> Deref for Magic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.architecture
    }
}

impl<T: Debug> Debug for Magic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.architecture)
    }
}

impl<T: BinRead<Args = ()> + Copy + PartialEq + Send + Sync + 'static> BinRead for Magic<T> {
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
