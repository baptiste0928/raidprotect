use mongodb::bson;
use pretty_assertions::assert_eq;
use raidprotect_model::database::model::{CaptchaConfig, GuildConfig, ModerationConfig};
use serde_test::{assert_tokens, Token};
use twilight_model::id::Id;

#[test]
fn test_guild_default() {
    let guild = GuildConfig::new(Id::new(1));

    assert_tokens(
        &guild,
        &[
            Token::Struct {
                name: "GuildConfig",
                len: 5,
            },
            Token::Str("_id"),
            Token::I64(1),
            Token::Str("logs_chan"),
            Token::None,
            Token::Str("lang"),
            Token::Str("fr"),
            Token::Str("moderation"),
            Token::Struct {
                name: "ModerationConfig",
                len: 2,
            },
            Token::Str("enforce_reason"),
            Token::Bool(false),
            Token::Str("anonymize"),
            Token::Bool(true),
            Token::StructEnd,
            Token::Str("captcha"),
            Token::Struct {
                name: "CaptchaConfig",
                len: 1,
            },
            Token::Str("enabled"),
            Token::Bool(false),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_guild_full() {
    let guild = GuildConfig {
        id: Id::new(1),
        logs_chan: Some(Id::new(2)),
        lang: "en".to_string(),
        moderation: ModerationConfig {
            roles: vec![Id::new(3), Id::new(4)],
            enforce_reason: true,
            anonymize: false,
        },
        captcha: CaptchaConfig {
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
            Token::Struct {
                name: "GuildConfig",
                len: 5,
            },
            Token::Str("_id"),
            Token::I64(1),
            Token::Str("logs_chan"),
            Token::Some,
            Token::I64(2),
            Token::Str("lang"),
            Token::Str("en"),
            // moderation
            Token::Str("moderation"),
            Token::Struct {
                name: "ModerationConfig",
                len: 3,
            },
            Token::Str("roles"),
            Token::Seq { len: Some(2) },
            Token::I64(3),
            Token::I64(4),
            Token::SeqEnd,
            Token::Str("enforce_reason"),
            Token::Bool(true),
            Token::Str("anonymize"),
            Token::Bool(false),
            Token::StructEnd,
            // captcha
            Token::Str("captcha"),
            Token::Struct {
                name: "CaptchaConfig",
                len: 6,
            },
            Token::Str("enabled"),
            Token::Bool(true),
            Token::Str("channel"),
            Token::Some,
            Token::I64(5),
            Token::Str("message"),
            Token::Some,
            Token::I64(6),
            Token::Str("role"),
            Token::Some,
            Token::I64(7),
            Token::Str("verified_roles"),
            Token::Seq { len: Some(2) },
            Token::I64(8),
            Token::I64(9),
            Token::SeqEnd,
            Token::Str("logs"),
            Token::Some,
            Token::I64(10),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_guild_bson() {
    let guild = GuildConfig {
        id: Id::new(1),
        logs_chan: Some(Id::new(2)),
        lang: "en".to_string(),
        moderation: ModerationConfig {
            roles: vec![Id::new(3), Id::new(4)],
            enforce_reason: true,
            anonymize: false,
        },
        captcha: CaptchaConfig {
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
        "lang": "en".to_string(),
        "moderation": {
            "roles": [3_i64, 4_i64],
            "enforce_reason": true,
            "anonymize": false,
        },
        "captcha": {
            "enabled": true,
            "channel": 5_i64,
            "message": 6_i64,
            "role": 7_i64,
            "verified_roles": [8_i64, 9_i64],
            "logs": 10_i64,
        },
    };

    assert_eq!(bson::to_document(&guild).unwrap(), expected);
    assert_eq!(bson::from_document::<GuildConfig>(expected).unwrap(), guild);
}
