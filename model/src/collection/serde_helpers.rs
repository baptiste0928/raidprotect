//! Serde helpers used in the `collection` module.
//!
//! These modules are intended to be used with [`serde_as`] and customize the
//! serialization and deserialization behavior of the fields they are applied on.
//!
//! [`serde_as`]: serde_with::serde_as

use std::num::NonZeroI64;

use serde::{ser::Error, Deserialize, Deserializer, Serializer};
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
#[derive(Debug, Clone, Copy)]
pub struct IdAsI64;

impl<'de, T> DeserializeAs<'de, Id<T>> for IdAsI64 {
    fn deserialize_as<D>(deserializer: D) -> Result<Id<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = NonZeroI64::deserialize(deserializer)?.get();

        Ok(Id::new(id as u64))
    }
}

impl<T: Copy> SerializeAs<Id<T>> for IdAsI64 {
    fn serialize_as<S>(source: &Id<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match i64::try_from(source.get()) {
            Ok(val) => serializer.serialize_i64(val),
            Err(_) => Err(Error::custom(format!("cannot convert {} to i64", source))),
        }
    }
}
