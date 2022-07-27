use mongodb::bson;
use pretty_assertions::assert_eq;
use raidprotect_model::mongodb::guild::{Captcha, Guild, Moderation};
use serde_test::{assert_tokens, Token};
use twilight_model::id::Id;

#[test]
fn test_guild_default() {
    let guild = Guild::new(Id::new(1));

    assert_tokens(
        &guild,
        &[
            Token::Map { len: None },
            Token::Str("_id"),
            Token::I64(1),
            Token::Str("logs_chan"),
            Token::None,
            Token::Str("moderation_enforce_reason"),
            Token::Bool(false),
            Token::Str("moderation_anonymize"),
            Token::Bool(true),
            Token::Str("captcha_enabled"),
            Token::Bool(false),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_guild_full() {
    let guild = Guild {
        id: Id::new(1),
        logs_chan: Some(Id::new(2)),
        moderation: Moderation {
            roles: vec![Id::new(3), Id::new(4)],
            enforce_reason: true,
            anonymize: false,
        },
        captcha: Captcha {
            enabled: true,
            channel: Some(Id::new(5)),
            message: Some(Id::new(6)),
            role: Some(Id::new(7)),
            verified_roles: vec![Id::new(8), Id::new(9)],
            logs: Some(Id::new(10)),
        },
    };

    assert_tokens(
        &guild,
        &[
            Token::Map { len: None },
            Token::Str("_id"),
            Token::I64(1),
            Token::Str("logs_chan"),
            Token::Some,
            Token::I64(2),
            Token::Str("moderation_roles"),
            Token::Seq { len: Some(2) },
            Token::I64(3),
            Token::I64(4),
            Token::SeqEnd,
            Token::Str("moderation_enforce_reason"),
            Token::Bool(true),
            Token::Str("moderation_anonymize"),
            Token::Bool(false),
            Token::Str("captcha_enabled"),
            Token::Bool(true),
            Token::Str("captcha_channel"),
            Token::Some,
            Token::I64(5),
            Token::Str("captcha_message"),
            Token::Some,
            Token::I64(6),
            Token::Str("captcha_role"),
            Token::Some,
            Token::I64(7),
            Token::Str("captcha_verified_roles"),
            Token::Seq { len: Some(2) },
            Token::I64(8),
            Token::I64(9),
            Token::SeqEnd,
            Token::Str("captcha_logs"),
            Token::Some,
            Token::I64(10),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_guild_bson() {
    let guild = Guild {
        id: Id::new(1),
        logs_chan: Some(Id::new(2)),
        moderation: Moderation {
            roles: vec![Id::new(3), Id::new(4)],
            enforce_reason: true,
            anonymize: false,
        },
        captcha: Captcha {
            enabled: true,
            channel: Some(Id::new(5)),
            message: Some(Id::new(6)),
            role: Some(Id::new(7)),
            verified_roles: vec![Id::new(8), Id::new(9)],
            logs: Some(Id::new(10)),
        },
    };

    let expected = bson::doc! {
        "_id": 1_i64,
        "logs_chan": 2_i64,
        "moderation_roles": [3_i64, 4_i64],
        "moderation_enforce_reason": true,
        "moderation_anonymize": false,
        "captcha_enabled": true,
        "captcha_channel": 5_i64,
        "captcha_message": 6_i64,
        "captcha_role": 7_i64,
        "captcha_verified_roles": [8_i64, 9_i64],
        "captcha_logs": 10_i64,
    };

    assert_eq!(bson::to_document(&guild).unwrap(), expected);
    assert_eq!(bson::from_document::<Guild>(expected).unwrap(), guild);
}
