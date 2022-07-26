use mongodb::bson;
use pretty_assertions::assert_eq;
use raidprotect_model::mongodb::guild::{Captcha, Config, Guild};
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
            Token::Map { len: None },
            Token::Str("enforce_reason"),
            Token::Bool(false),
            Token::Str("anonymize_moderator"),
            Token::Bool(true),
            Token::Str("captcha_enabled"),
            Token::Bool(false),
            Token::MapEnd,
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
            captcha: Captcha {
                enabled: true,
                verification_channel: Some(Id::new(5)),
                verification_message: Some(Id::new(6)),
                unverified_role: Some(Id::new(7)),
                verified_roles: vec![Id::new(8), Id::new(9)],
                logs_channel: Some(Id::new(10)),
            },
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
            Token::Map { len: None },
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
            Token::Str("captcha_enabled"),
            Token::Bool(true),
            Token::Str("captcha_verification_channel"),
            Token::Some,
            Token::I64(5),
            Token::Str("captcha_verification_message"),
            Token::Some,
            Token::I64(6),
            Token::Str("captcha_unverified_role"),
            Token::Some,
            Token::I64(7),
            Token::Str("captcha_verified_roles"),
            Token::Seq { len: Some(2) },
            Token::I64(8),
            Token::I64(9),
            Token::SeqEnd,
            Token::Str("captcha_logs_channel"),
            Token::Some,
            Token::I64(10),
            Token::MapEnd,
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
            captcha: Captcha {
                enabled: true,
                verification_channel: Some(Id::new(5)),
                verification_message: Some(Id::new(6)),
                unverified_role: Some(Id::new(7)),
                verified_roles: vec![Id::new(8), Id::new(9)],
                logs_channel: Some(Id::new(10)),
            },
        },
    };

    let expected = bson::doc! {
        "_id": 1_i64,
        "config": {
            "logs_chan": 2_i64,
            "moderator_roles": [3_i64, 4_i64],
            "enforce_reason": true,
            "anonymize_moderator": false,
            "captcha_enabled": true,
            "captcha_verification_channel": 5_i64,
            "captcha_verification_message": 6_i64,
            "captcha_unverified_role": 7_i64,
            "captcha_verified_roles": [8_i64, 9_i64],
            "captcha_logs_channel": 10_i64,
        }
    };

    assert_eq!(bson::to_document(&guild).unwrap(), expected);
    assert_eq!(bson::from_document::<Guild>(expected).unwrap(), guild);
}
