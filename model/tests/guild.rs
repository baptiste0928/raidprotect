use mongodb::bson;
use pretty_assertions::assert_eq;
use raidprotect_model::mongodb::{Config, Guild};
use serde_test::{assert_tokens, Token};
use twilight_model::id::Id;

#[test]
fn test_guild_default() {
    let guild = Guild::new(Id::new(1));

    assert_tokens(
        &guild,
        &[
            Token::Struct {
                name: "Guild",
                len: 2,
            },
            Token::Str("_id"),
            Token::I64(1),
            Token::Str("config"),
            Token::Struct {
                name: "Config",
                len: 2,
            },
            Token::Str("enforce_reason"),
            Token::Bool(false),
            Token::Str("anonymize_moderator"),
            Token::Bool(true),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_guild_full() {
    let guild = Guild {
        id: Id::new(1),
        config: Config {
            logs_chan: Some(Id::new(2)),
            moderator_roles: vec![Id::new(3), Id::new(4)],
            enforce_reason: true,
            anonymize_moderator: false,
        },
    };

    assert_tokens(
        &guild,
        &[
            Token::Struct {
                name: "Guild",
                len: 2,
            },
            Token::Str("_id"),
            Token::I64(1),
            Token::Str("config"),
            Token::Struct {
                name: "Config",
                len: 4,
            },
            Token::Str("logs_chan"),
            Token::Some,
            Token::I64(2),
            Token::Str("moderator_roles"),
            Token::Seq { len: Some(2) },
            Token::I64(3),
            Token::I64(4),
            Token::SeqEnd,
            Token::Str("enforce_reason"),
            Token::Bool(true),
            Token::Str("anonymize_moderator"),
            Token::Bool(false),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_guild_bson() {
    let guild = Guild {
        id: Id::new(1),
        config: Config {
            logs_chan: Some(Id::new(2)),
            moderator_roles: vec![Id::new(3), Id::new(4)],
            enforce_reason: true,
            anonymize_moderator: false,
        },
    };

    let expected = bson::doc! {
        "_id": 1_i64,
        "config": {
            "logs_chan": 2_i64,
            "moderator_roles": [3_i64, 4_i64],
            "enforce_reason": true,
            "anonymize_moderator": false,
        }
    };

    assert_eq!(bson::to_document(&guild).unwrap(), expected);
    assert_eq!(bson::from_document::<Guild>(expected).unwrap(), guild);
}
