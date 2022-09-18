//! Captcha modal interaction handling.

use std::time::Duration;

use anyhow::{bail, Context};
use raidprotect_model::{
    cache::discord::{
        permission::{CachePermissions, RoleOrdering},
        CachedRole,
    },
    database::model::GuildConfig,
};
use tracing::{error, info, instrument};
use twilight_model::{
    application::interaction::Interaction,
    guild::Permissions,
    id::{
        marker::{RoleMarker, UserMarker},
        Id,
    },
};
use twilight_util::builder::embed::EmbedBuilder;

use super::verify::{get_captcha, kick_after};
use crate::{
    interaction::{
        embed,
        response::InteractionResponse,
        util::{
            parse_modal_data, parse_modal_field_required, GuildConfigExt, GuildInteractionContext,
        },
    },
    shard::BotState,
};

/// Captcha verification modal.
///
/// This modal is used to ask the user to solve the captcha.
pub struct CaptchaModal;

impl CaptchaModal {
    #[instrument(skip(state))]
    pub async fn handle(
        mut interaction: Interaction,
        state: &BotState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let data = parse_modal_data(&mut interaction)?;
        let ctx = GuildInteractionContext::new(interaction)?;
        let config = ctx.config(state).await?;

        // Get the pending captcha from the cache.
        let captcha = match get_captcha(&ctx, state).await? {
            Some(captcha) => captcha,
            None => {
                return Ok(embed::captcha::captcha_not_found(ctx.lang));
            }
        };

        // Check if the entered code is correct.
        let code = parse_modal_field_required(&data, "captcha-input")?;

        if !validate_code(code, &captcha.code) {
            let state_clone = state.clone();
            tokio::spawn(async move {
                kick_after(&state_clone, ctx.guild_id, ctx.author.id, config.lang()).await
            });

            return Ok(embed::captcha::captcha_invalid_code(ctx.lang));
        }

        // Delete the captcha from the cache and update the user roles.
        state.cache.delete(&captcha).await?;

        let state_clone = state.clone();
        tokio::spawn(async move {
            // Wait for the user to read the message.
            tokio::time::sleep(Duration::from_secs(2)).await;

            if let Err(error) = update_roles(ctx.author.id, &config, &state_clone).await {
                error!(error = ?error, "failed to user roles");
            }
        });

        // Send a success message.
        let embed = EmbedBuilder::new()
            .title(ctx.lang.captcha_success_title())
            .color(embed::COLOR_SUCCESS)
            .description(ctx.lang.captcha_success_description())
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

/// Update the user roles.
async fn update_roles(
    user_id: Id<UserMarker>,
    config: &GuildConfig,
    state: &BotState,
) -> Result<(), anyhow::Error> {
    // Get the current member roles.
    let member = state
        .http
        .guild_member(config.id, user_id)
        .exec()
        .await?
        .model()
        .await?;
    let mut roles = member.roles;

    // Ensure the bot has required permissions.
    let permissions = state
        .cache
        .permissions(config.id)
        .await?
        .current_member()
        .await?;

    if !permissions.guild().contains(Permissions::MANAGE_ROLES) {
        bail!("missing permission to manage roles");
    }

    // Remove the captcha role.
    let role = config
        .captcha
        .role
        .context("missing captcha role in config")?;

    if !check_role_permission(&permissions, role, state).await {
        bail!("missing permission to manage captcha role");
    }

    if let Some(index) = roles.iter().position(|r| r == &role) {
        roles.remove(index);
    }

    // Add the verified roles.
    for role in &config.captcha.verified_roles {
        if !check_role_permission(&permissions, *role, state).await {
            info!("missing permission to manage verified role");
            continue;
        }

        if !roles.contains(role) {
            roles.push(*role);
        }
    }

    // Update the member roles.
    state
        .http
        .update_guild_member(config.id, user_id)
        .roles(&roles)
        .exec()
        .await?;

    Ok(())
}

/// Ensure the bot has the required permission to update a role.
async fn check_role_permission(
    permissions: &CachePermissions<'_>,
    role_id: Id<RoleMarker>,
    state: &BotState,
) -> bool {
    let role = match state.cache.get::<CachedRole>(&role_id).await {
        Ok(Some(role)) => role,
        _ => return false,
    };

    if RoleOrdering::from(&role) >= permissions.highest_role() {
        return false;
    }

    true
}

/// Validate the captcha code, accepting at most one error.
fn validate_code(a: &str, b: &str) -> bool {
    let mut errors: u8 = 0;

    for (a, b) in a.chars().zip(b.chars()) {
        if a != b {
            errors += 1;
        }
    }

    errors <= 1
}

#[cfg(test)]
mod tests {
    use super::validate_code;

    #[test]
    fn test_validate_code() {
        assert!(validate_code("abc", "abc")); // no errors
        assert!(validate_code("abc", "abd")); // one error
        assert!(!validate_code("abc", "ade")); // two errors (fail)
    }
}
