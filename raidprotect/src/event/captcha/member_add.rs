//! Handle `MemberAdd` event.

use std::sync::Arc;

use raidprotect_model::cache::model::interaction::PendingCaptcha;
use time::{Duration, OffsetDateTime};
use tracing::{debug, error, instrument};
use twilight_http::request::AuditLogReason;
use twilight_model::guild::Member;

use crate::{cluster::ClusterState, feature::captcha, translations::Lang};

/// Handle `MemberAdd` event.
pub async fn member_add(member: &Member, state: Arc<ClusterState>) {
    if let Err(error) = member_add_inner(member, state).await {
        error!(error = ?error, member = ?member, "error while processing `MemberAdd` event");
    }
}

async fn member_add_inner(member: &Member, state: Arc<ClusterState>) -> Result<(), anyhow::Error> {
    // Ensure the member has joined recently to ignore members sent on bot
    // startup.
    let now = OffsetDateTime::now_utc();
    let joined_at = OffsetDateTime::from_unix_timestamp(member.joined_at.as_secs())?;

    if (now - joined_at) > Duration::seconds(5) {
        return Ok(());
    }

    // Get the guild configuration.
    let config = state.mongodb().get_guild_or_create(member.guild_id).await?;
    let lang = Lang::from(&*config.lang);

    if !config.captcha.enabled {
        return Ok(());
    }

    let role = match config.captcha.role {
        Some(role) => role,
        None => {
            debug!("captcha is enabled but no role is set");

            return Ok(());
        }
    };

    // Give the unverified role to the member.
    if let Err(error) = state
        .cache_http(member.guild_id)
        .add_guild_member_role(member.user.id, role)
        .await?
        .reason(lang.captcha_role_reason())?
        .exec()
        .await
    {
        error!(error = ?error, "error while adding unverified role to member");

        return Ok(());
    }

    // Store the captcha in redis.
    let pending_captcha = PendingCaptcha {
        guild_id: member.guild_id,
        member_id: member.user.id,
        code: String::new(), // Code generated on button click.
        regenerate_count: 0,
        expires_at: OffsetDateTime::now_utc() + captcha::DEFAULT_DURATION,
    };

    state.redis().set(&pending_captcha).await?;
    tokio::spawn(captcha_expire(state, pending_captcha, lang));

    Ok(())
}

/// Kick the user if the captcha has not been validated in time.
#[instrument(skip(state))]
async fn captcha_expire(state: Arc<ClusterState>, captcha: PendingCaptcha, lang: Lang) {
    // Sleep until the captcha expiration.
    let duration = captcha.expires_at - OffsetDateTime::now_utc();

    if let Ok(duration) = duration.try_into() {
        tokio::time::sleep(duration).await;
    }

    // If the captcha still present in the cache, kick the user.
    let key = (captcha.guild_id, captcha.member_id);
    let pending_captcha = state.redis().get::<PendingCaptcha>(&key).await;

    if pending_captcha
        .as_ref()
        .map(Option::is_some)
        .unwrap_or(false)
    {
        debug!("captcha not expired");
        return;
    }

    if let Err(error) = kick_user_expired(&state, captcha, lang).await {
        error!(error = ?error, "error while kicking user on captcha expiration");
    }
}

async fn kick_user_expired(
    state: &ClusterState,
    captcha: PendingCaptcha,
    lang: Lang,
) -> Result<(), anyhow::Error> {
    state
        .cache_http(captcha.guild_id)
        .remove_guild_member(captcha.member_id)
        .await?
        .reason(lang.captcha_expired_reason())?
        .exec()
        .await?;

    Ok(())
}
