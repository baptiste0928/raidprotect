//! Serde helpers used in the `collection` module.
//!
//! These modules are intended to be used with [`serde_as`] and customize the
//! serialization and deserialization behavior of the fields they are applied on.
//!
//! [`serde_as`]: serde_with::serde_as

use serde::{de, ser, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use twilight_model::id::Id;

/// Serialize twilight [`Id`] as [`i64`].
///
/// This type implement [`SerializeAs`] and [`DeserializeAs`] and should be
/// used with the [`serde_as`] macro.
///
/// Ids are serialized as [`i64`] because BSON (used by MongoDB) doesn't
/// support storing integers as [`u64`].
///
/// Because one bit is lost when using [`i64`], the maximum timestamp
/// that can be stored is Sep 06 2084 (which shouldn't be a problem). Any
/// id that does not fit in a [`i64`] will produce an error.
///
/// [`serde_as`]: serde_with::serde_as
#[derive(Debug)]
pub struct IdAsI64;

impl<'de, T> DeserializeAs<'de, Id<T>> for IdAsI64 {
    fn deserialize_as<D>(deserializer: D) -> Result<Id<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = i64::deserialize(deserializer)?;

        match id {
            1.. => Ok(Id::new(id as u64)),
            _ => Err(de::Error::custom(format!(
                "invalid value: integer `{id}`, expected nonzero positive i64"
            ))),
        }
    }
}

impl<T: Copy> SerializeAs<Id<T>> for IdAsI64 {
    fn serialize_as<S>(source: &Id<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match i64::try_from(source.get()) {
            Ok(val) => serializer.serialize_i64(val),
            Err(_) => Err(ser::Error::custom(format!(
                "cannot convert {} to i64",
                source
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IdAsI64;

    use serde::{Deserialize, Serialize};
    use serde_test::{
        assert_de_tokens, assert_de_tokens_error, assert_ser_tokens_error, assert_tokens, Token,
    };
    use serde_with::serde_as;
    use twilight_model::id::{marker::GenericMarker, Id};

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct IdWrapper(#[serde_as(as = "IdAsI64")] Id<GenericMarker>);

    #[test]
    fn test_id() {
        let id = IdWrapper(Id::new(1));

        assert_tokens(
            &id,
            &[Token::NewtypeStruct { name: "IdWrapper" }, Token::I64(1)],
        );
    }

    #[test]
    fn test_de_id_other() {
        let id = IdWrapper(Id::new(1));

        assert_de_tokens(
            &id,
            &[Token::NewtypeStruct { name: "IdWrapper" }, Token::U64(1)],
        )
    }

    #[test]
    fn test_de_id_zero() {
        assert_de_tokens_error::<IdWrapper>(
            &[Token::NewtypeStruct { name: "IdWrapper" }, Token::I64(0)],
            "invalid value: integer `0`, expected nonzero positive i64",
        );
    }

    #[test]
    fn test_de_id_negative() {
        assert_de_tokens_error::<IdWrapper>(
            &[Token::NewtypeStruct { name: "IdWrapper" }, Token::I64(-20)],
            "invalid value: integer `-20`, expected nonzero positive i64",
        );
    }

    #[test]
    fn test_de_id_overflow() {
        assert_de_tokens_error::<IdWrapper>(
            &[
                Token::NewtypeStruct { name: "IdWrapper" },
                Token::U64(u64::MAX),
            ],
            "invalid value: integer `18446744073709551615`, expected i64",
        );
    }

    #[test]
    fn test_ser_id_overflow() {
        let id = IdWrapper(Id::new(u64::MAX));

        assert_ser_tokens_error(
            &id,
            &[Token::NewtypeStruct { name: "IdWrapper" }],
            "cannot convert 18446744073709551615 to i64",
        )
    }
}
