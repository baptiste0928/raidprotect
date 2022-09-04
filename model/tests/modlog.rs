use mongodb::bson::{self, oid::ObjectId, DateTime};
use pretty_assertions::assert_eq;
use raidprotect_model::database::model::{Modlog, ModlogType, ModlogUser};
use serde_test::{assert_tokens, Configure, Token};
use time::OffsetDateTime;
use twilight_model::{id::Id, util::ImageHash};

#[test]
fn test_modlog_full() {
    let modlog = Modlog {
        id: Some(ObjectId::parse_str("62aca55a551e9a0102351bda").unwrap()),
        kind: ModlogType::Kick,
        guild_id: Id::new(1),
        user: ModlogUser {
            id: Id::new(2),
            name: "username".to_string(),
            discriminator: 1234,
            avatar: Some(ImageHash::parse("1acefe340fafb4ecefae407f3abdb323".as_bytes()).unwrap()),
        },
        moderator: ModlogUser {
            id: Id::new(3),
            name: "moderator".to_string(),
            discriminator: 4567,
            avatar: Some(
                ImageHash::parse("a_b2a6536641da91a0b59bd66557c56c36".as_bytes()).unwrap(),
            ),
        },
        date: OffsetDateTime::from_unix_timestamp(1_628_594_197_123).unwrap(),
        reason: Some("reason".to_string()),
        notes: Some("notes".to_string()),
    };

    assert_tokens(
        &modlog.compact(),
        &[
            Token::Struct {
                name: "Modlog",
                len: 8,
            },
            // id
            Token::Str("_id"),
            Token::Some,
            Token::Struct {
                name: "$oid",
                len: 1,
            },
            Token::Str("$oid"),
            Token::Str("62aca55a551e9a0102351bda"),
            Token::StructEnd,
            // kind
            Token::Str("kind"),
            Token::Enum { name: "ModlogType" },
            Token::Str("kick"),
            Token::Unit,
            // guild_id
            Token::Str("guild_id"),
            Token::I64(1),
            // user
            Token::Str("user"),
            Token::Struct {
                name: "ModlogUser",
                len: 4,
            },
            // user: id
            Token::Str("id"),
            Token::I64(2),
            // user: name
            Token::Str("name"),
            Token::String("username"),
            // user: discriminator
            Token::Str("discriminator"),
            Token::U16(1234),
            // user: avatar
            Token::Str("avatar"),
            Token::Some,
            Token::Str("1acefe340fafb4ecefae407f3abdb323"),
            Token::StructEnd,
            // moderator
            Token::Str("moderator"),
            Token::Struct {
                name: "ModlogUser",
                len: 4,
            },
            // moderator: id
            Token::Str("id"),
            Token::I64(3),
            // moderator: name
            Token::Str("name"),
            Token::String("moderator"),
            // moderator: discriminator
            Token::Str("discriminator"),
            Token::U16(4567),
            // moderator: avatar
            Token::Str("avatar"),
            Token::Some,
            Token::Str("a_b2a6536641da91a0b59bd66557c56c36"),
            Token::StructEnd,
            // date
            Token::Str("date"),
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
            // reason
            Token::Str("reason"),
            Token::Some,
            Token::String("reason"),
            // notes
            Token::Str("notes"),
            Token::Some,
            Token::String("notes"),
            Token::StructEnd,
        ],
    )
}

#[test]
fn test_modlog_bson() {
    let modlog = Modlog {
        id: Some(ObjectId::parse_str("62aca55a551e9a0102351bda").unwrap()),
        kind: ModlogType::Kick,
        guild_id: Id::new(1),
        user: ModlogUser {
            id: Id::new(2),
            name: "username".to_string(),
            discriminator: 1234,
            avatar: Some(ImageHash::parse("1acefe340fafb4ecefae407f3abdb323".as_bytes()).unwrap()),
        },
        moderator: ModlogUser {
            id: Id::new(3),
            name: "moderator".to_string(),
            discriminator: 4567,
            avatar: Some(
                ImageHash::parse("a_b2a6536641da91a0b59bd66557c56c36".as_bytes()).unwrap(),
            ),
        },
        date: OffsetDateTime::from_unix_timestamp(1_628_594_197_123).unwrap(),
        reason: Some("reason".to_string()),
        notes: Some("notes".to_string()),
    };

    let expected = bson::doc! {
        "_id": ObjectId::parse_str("62aca55a551e9a0102351bda").unwrap(),
        "kind": "kick",
        "guild_id": 1_i64,
        "user": {
            "id": 2_i64,
            "name": "username",
            "discriminator": 1234_i32,
            "avatar": "1acefe340fafb4ecefae407f3abdb323",
        },
        "moderator": {
            "id": 3_i64,
            "name": "moderator",
            "discriminator": 4567_i32,
            "avatar": "a_b2a6536641da91a0b59bd66557c56c36",
        },
        "date": DateTime::from_millis(1_628_594_197_123),
        "reason": "reason",
        "notes": "notes",
    };

    assert_eq!(bson::to_document(&modlog).unwrap(), expected);
    assert_eq!(bson::from_document::<Modlog>(expected).unwrap(), modlog);
}
