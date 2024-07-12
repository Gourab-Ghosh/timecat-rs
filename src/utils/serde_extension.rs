use super::*;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct SerdeWrapper<T>(T);

impl<T> SerdeWrapper<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn from_boxed_value(value: Box<T>) -> Box<Self> {
        let raw: *mut T = Box::into_raw(value);
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

impl<T> From<T> for SerdeWrapper<T> {
    fn from(value: T) -> Self {
        SerdeWrapper(value)
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
        Ok(SerdeWrapper(T::read_options(reader, options, ())?))
    }
}

#[cfg(feature = "serde")]
mod serde_implementations {
    use super::*;
    use serde::de::{self, Visitor};
    use serde::ser::SerializeTuple;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;

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
            struct ArrayVisitor<T, const N: usize> {
                marker: PhantomData<T>,
            }

            impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
            where
                T: Deserialize<'de>,
            {
                type Value = SerdeWrapper<[T; N]>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(&format!("an array of {} elements", N))
                }

                fn visit_seq<V>(self, mut seq: V) -> std::result::Result<Self::Value, V::Error>
                where
                    V: de::SeqAccess<'de>,
                {
                    let mut array: [MaybeUninit<T>; N] =
                        unsafe { MaybeUninit::uninit().assume_init() };
                    for i in 0..N {
                        array[i] = MaybeUninit::new(
                            seq.next_element()?
                                .ok_or_else(|| de::Error::invalid_length(i, &self))?,
                        );
                    }
                    Ok(SerdeWrapper(unsafe {
                        std::ptr::read(&array as *const _ as *const [T; N])
                    }))
                }
            }

            deserializer.deserialize_tuple(
                N,
                ArrayVisitor {
                    marker: PhantomData,
                },
            )
        }
    }
}
