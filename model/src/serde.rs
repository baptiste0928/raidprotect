//! Serde helpers.
//!
//! These modules are intended to be used with [`serde_as`] and customize the
//! serialization and deserialization behavior of the fields they are applied on.
//!
//! [`serde_as`]: serde_with::serde_as

use std::num::NonZeroU64;

use mongodb::bson;
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use time::OffsetDateTime;
use twilight_model::{id::Id, util::Timestamp};

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

impl<T> SerializeAs<Id<T>> for IdAsI64 {
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

/// Serialize twilight [`Id`] as [`u64`].
///
/// This type implement [`SerializeAs`] and [`DeserializeAs`] and should be
/// used with the [`serde_as`] macro.
///
/// [`serde_as`]: serde_with::serde_as
#[derive(Debug)]
pub struct IdAsU64;

impl<'de, T> DeserializeAs<'de, Id<T>> for IdAsU64 {
    fn deserialize_as<D>(deserializer: D) -> Result<Id<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = NonZeroU64::deserialize(deserializer)?;

        Ok(Id::from(id))
    }
}

impl<T> SerializeAs<Id<T>> for IdAsU64 {
    fn serialize_as<S>(source: &Id<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(source.get())
    }
}

/// Serialize twilight [`Timestamp`] as [`i64`].
///
/// The default implementation serializes timestamps as ISO 8601 datetime.
///
/// This type implement [`SerializeAs`] and [`DeserializeAs`] and should be
/// used with the [`serde_as`] macro.
///
/// [`serde_as`]: serde_with::serde_as
#[derive(Debug)]
pub struct TimestampAsI64;

impl<'de> DeserializeAs<'de, Timestamp> for TimestampAsI64 {
    fn deserialize_as<D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let micros = i64::deserialize(deserializer)?;

        Timestamp::from_micros(micros).map_err(de::Error::custom)
    }
}

impl SerializeAs<Timestamp> for TimestampAsI64 {
    fn serialize_as<S>(source: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(source.as_micros())
    }
}

/// Serialize [`OffsetDateTime`] as [`bson::DateTime`].
///
/// This allow usage of `time` types with MongoDB models.
///
/// This type implement [`SerializeAs`] and [`DeserializeAs`] and should be
/// used with the [`serde_as`] macro.
///
/// [`serde_as`]: serde_with::serde_as
#[derive(Debug)]
pub struct DateTimeAsBson;

impl<'de> DeserializeAs<'de, OffsetDateTime> for DateTimeAsBson {
    fn deserialize_as<D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let datetime = bson::DateTime::deserialize(deserializer)?;

        OffsetDateTime::from_unix_timestamp(datetime.timestamp_millis()).map_err(de::Error::custom)
    }
}

impl SerializeAs<OffsetDateTime> for DateTimeAsBson {
    fn serialize_as<S>(source: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let datetime = bson::DateTime::from_millis(source.unix_timestamp());

        datetime.serialize(serializer)
    }
}

/// Serialize [`OffsetDateTime`] as a UNIX timestamp ([`i64`]).
///
/// This type implement [`SerializeAs`] and [`DeserializeAs`] and should be
/// used with the [`serde_as`] macro.
///
/// [`serde_as`]: serde_with::serde_as
pub struct DateTimeAsI64;

impl<'de> DeserializeAs<'de, OffsetDateTime> for DateTimeAsI64 {
    fn deserialize_as<D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;

        OffsetDateTime::from_unix_timestamp(timestamp).map_err(de::Error::custom)
    }
}

impl SerializeAs<OffsetDateTime> for DateTimeAsI64 {
    fn serialize_as<S>(source: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(source.unix_timestamp())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_test::{assert_de_tokens_error, assert_ser_tokens_error, assert_tokens, Token};
    use serde_with::serde_as;
    use time::OffsetDateTime;
    use twilight_model::{
        id::{marker::GenericMarker, Id},
        util::Timestamp,
    };

    use super::{DateTimeAsBson, DateTimeAsI64, IdAsI64, IdAsU64, TimestampAsI64};

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct IdI64Wrapper(#[serde_as(as = "IdAsI64")] Id<GenericMarker>);

    #[test]
    fn test_id_i64() {
        let id = IdI64Wrapper(Id::new(1));

        assert_tokens(
            &id,
            &[
                Token::NewtypeStruct {
                    name: "IdI64Wrapper",
                },
                Token::I64(1),
            ],
        );
    }

    #[test]
    fn test_de_id_i64_zero() {
        assert_de_tokens_error::<IdI64Wrapper>(
            &[
                Token::NewtypeStruct {
                    name: "IdI64Wrapper",
                },
                Token::I64(0),
            ],
            "invalid value: integer `0`, expected nonzero positive i64",
        );
    }

    #[test]
    fn test_de_id_i64_negative() {
        assert_de_tokens_error::<IdI64Wrapper>(
            &[
                Token::NewtypeStruct {
                    name: "IdI64Wrapper",
                },
                Token::I64(-20),
            ],
            "invalid value: integer `-20`, expected nonzero positive i64",
        );
    }

    #[test]
    fn test_de_id_i64_overflow() {
        assert_de_tokens_error::<IdI64Wrapper>(
            &[
                Token::NewtypeStruct {
                    name: "IdI64Wrapper",
                },
                Token::U64(u64::MAX),
            ],
            "invalid value: integer `18446744073709551615`, expected i64",
        );
    }

    #[test]
    fn test_ser_id_i64_overflow() {
        let id = IdI64Wrapper(Id::new(u64::MAX));

        assert_ser_tokens_error(
            &id,
            &[Token::NewtypeStruct {
                name: "IdI64Wrapper",
            }],
            "cannot convert 18446744073709551615 to i64",
        )
    }

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct IdU64Wrapper(#[serde_as(as = "IdAsU64")] Id<GenericMarker>);

    #[test]
    fn test_id_u64() {
        let id = IdU64Wrapper(Id::new(1));

        assert_tokens(
            &id,
            &[
                Token::NewtypeStruct {
                    name: "IdU64Wrapper",
                },
                Token::U64(1),
            ],
        );
    }

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TimestampWrapper(#[serde_as(as = "TimestampAsI64")] Timestamp);

    #[test]
    fn test_timestamp() {
        let timestamp = TimestampWrapper(Timestamp::from_micros(1_628_594_197_123_456).unwrap());

        assert_tokens(
            &timestamp,
            &[
                Token::NewtypeStruct {
                    name: "TimestampWrapper",
                },
                Token::I64(1_628_594_197_123_456),
            ],
        );
    }

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct DateTimeWrapper(#[serde_as(as = "DateTimeAsBson")] OffsetDateTime);

    #[test]
    fn test_datetime() {
        let datetime =
            DateTimeWrapper(OffsetDateTime::from_unix_timestamp(1_628_594_197_123).unwrap());

        assert_tokens(
            &datetime,
            &[
                Token::NewtypeStruct {
                    name: "DateTimeWrapper",
                },
                Token::Struct {
                    name: "$date",
                    len: 1,
                },
                Token::Str("$date"),
                Token::Struct {
                    name: "Int64",
                    len: 1,
                },
                Token::Str("$numberLong"),
                Token::Str("1628594197123"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        )
    }

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct DateTimeAsI64Wrapper(#[serde_as(as = "DateTimeAsI64")] OffsetDateTime);

    #[test]
    fn test_datetime_as_i64() {
        let datetime =
            DateTimeAsI64Wrapper(OffsetDateTime::from_unix_timestamp(1_628_594_197_123).unwrap());

        assert_tokens(
            &datetime,
            &[
                Token::NewtypeStruct {
                    name: "DateTimeAsI64Wrapper",
                },
                Token::I64(1_628_594_197_123),
            ],
        )
    }
}
