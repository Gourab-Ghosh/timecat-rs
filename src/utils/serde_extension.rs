use super::*;

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

#[cfg(feature = "serde")]
mod implementations {
    use super::*;
    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    use serde::ser::SerializeTuple;
    use serde::de::{self, Visitor};
    use std::marker::PhantomData;
    
    impl<T: Serialize, const N: usize> Serialize for SerdeWrapper<[T; N]> {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
            self.0.serialize(serializer)
        }
    }
    
    impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for SerdeWrapper<[T; N]> {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
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
                    let array: SerdeWrapper<[T; N]> = std::array::from_fn(|_| {
                        seq.next_element::<T>()
                            .ok()
                            .flatten()
                            .expect("unexpected end of sequence")
                    }).into();
                    Ok(array)
                }
            }
        
            deserializer.deserialize_tuple(N, ArrayVisitor { marker: PhantomData })
        }
    }
}
