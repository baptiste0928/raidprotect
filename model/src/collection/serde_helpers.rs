//! Serde helpers used in the `collection` module.
//!
//! These modules are intended to be used with `#[serde(with = "")]` and
//! customize the serialization and deserialization behavior of the fields
//! they are applied on.

pub mod id_as_i64 {
    //! Serialize twilight [`Id`] as [`i64`].
    //!
    //! Ids are serialized as [`i64`] because BSON (used by MongoDB) doesn't
    //! support storing integers as [`u64`].
    //!
    //! Because one bit is lost when using [`i64`], the maximum timestamp
    //! that can be stored is Sep 06 2084 (which shouldn't be a problem). Any
    //! id that does not fit in a [`i64`] will produce an error.

    use std::num::NonZeroI64;

    use serde::{ser, Deserialize, Deserializer, Serializer};
    use twilight_model::id::Id;

    /// Attempts to deserialize an [`Id`] from an [`i64`].
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Id<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = NonZeroI64::deserialize(deserializer)?.get();

        Ok(Id::new(id as u64))
    }

    /// Attempts to serialize an [`Id`] to an [`i64`].
    pub fn serialize<S, T>(val: &Id<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Copy,
    {
        match i64::try_from(val.get()) {
            Ok(val) => serializer.serialize_i64(val),
            Err(_) => Err(ser::Error::custom(format!("cannot convert {} to i64", val))),
        }
    }
}

pub mod option_id_as_i64 {
    //! Serialize twilight [`Option<Id>`] as [`Option<i64>`].
    //!
    //! This module is the same as [`id_as_i64`] but wraps type in [`Option`].

    use std::num::NonZeroI64;

    use serde::{Deserialize, Deserializer, Serializer};
    use twilight_model::id::Id;

    use super::id_as_i64;

    /// Attempts to deserialize an [`Id`] from an [`i64`].
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Id<T>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = Option::<NonZeroI64>::deserialize(deserializer)?;

        Ok(id.map(|id| Id::new(id.get() as u64)))
    }

    /// Attempts to serialize an [`Id`] to an [`i64`].
    pub fn serialize<S, T>(val: &Option<Id<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Copy,
    {
        match val {
            Some(val) => id_as_i64::serialize(val, serializer),
            None => serializer.serialize_none(),
        }
    }
}
