use super::*;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct SerdeWrapper<T>(T);

impl<T> SerdeWrapper<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn from_boxed_value(value: Box<T>) -> Box<Self> {
        let raw = Box::into_raw(value);
        let wrapped_raw = raw as *mut Self;
        unsafe { Box::from_raw(wrapped_raw) }
    }
}

impl<T> Deref for SerdeWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SerdeWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "binread")]
impl<T: BinRead<Args = ()>> BinRead for SerdeWrapper<T> {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(Self(T::read_options(reader, options, ())?))
    }
}

#[cfg(feature = "serde")]
mod serde_implementations {
    use super::*;

    impl<T: Serialize, const N: usize> Serialize for SerdeWrapper<[T; N]> {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for SerdeWrapper<[T; N]> {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self(<[T; N]>::deserialize(deserializer)?))
        }
    }

    struct ArrayVisitor<T, const N: usize> {
        marker: PhantomData<T>,
    }

    impl<'de, T, const N: usize> serde::de::Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str(&format!("an array of {} elements", N))
        }

        fn visit_seq<V>(self, mut seq: V) -> std::result::Result<Self::Value, V::Error>
        where
            V: serde::de::SeqAccess<'de>,
        {
            let mut array: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
            for (i, entry) in array.iter_mut().enumerate() {
                *entry = MaybeUninit::new(
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?,
                );
            }
            Ok(unsafe { std::ptr::read(&array as *const _ as *const [T; N]) })
        }
    }

    impl<T: Serialize, const N: usize> SerdeSerialize for [T; N] {
        fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
            self.as_slice().serialize(serializer)
        }
    }

    impl<'de, T: Deserialize<'de>, const N: usize> SerdeDeserialize<'de> for [T; N] {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Self, D::Error> {
            deserializer.deserialize_tuple(
                N,
                ArrayVisitor {
                    marker: PhantomData,
                },
            )
        }
    }

    impl<T: Serialize, const N: usize> SerdeSerialize for Box<[T; N]> {
        fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
            self.as_ref().serialize(serializer)
        }
    }

    impl<'de, T: Deserialize<'de>, const N: usize> SerdeDeserialize<'de> for Box<[T; N]> {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Self, D::Error> {
            <[T; N]>::deserialize(deserializer).map(Self::new)
        }
    }
}

#[cfg(feature = "serde")]
pub mod serde_handler {
    use super::*;

    pub fn serialize<S: Serializer>(
        obj: &impl SerdeSerialize,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        obj.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: SerdeDeserialize<'de>>(
        deserializer: D,
    ) -> std::result::Result<T, D::Error> {
        T::deserialize(deserializer)
    }
}
