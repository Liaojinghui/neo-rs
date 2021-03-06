use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};

use std::fmt;
use std::marker::PhantomData;

use crate::{FromHex, ToHex};

pub fn serialize_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToHex,
{
    let s = data.encode_hex_upper::<String>();
    serializer.serialize_str(&s)
}

pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToHex,
{
    let s = data.encode_hex::<String>();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromHex,
    <T as FromHex>::Error: fmt::Display,
{
    struct HexStrVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for HexStrVisitor<T>
    where
        T: FromHex,
        <T as FromHex>::Error: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a hex encoded string")
        }

        fn visit_str<E>(self, data: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            FromHex::from_hex(data).map_err(|e| Error::custom(e))
        }

        fn visit_borrowed_str<E>(self, data: &'de str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            FromHex::from_hex(data).map_err(|e| Error::custom(e))
        }
    }

    deserializer.deserialize_str(HexStrVisitor(PhantomData))
}
