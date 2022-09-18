//! Kick command.
//!
//! The command allows to kick a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is kicked, the action is logged in the database and a message is
//! sent in the guild's logs channel. The kicked user receives a pm with the
//! reason of the kick.

use raidprotect_model::{cache::model::interaction::PendingSanction, database::model::ModlogType};
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};
use twilight_model::{
    channel::message::component::{ActionRow, Component, TextInput, TextInputStyle},
    guild::Permissions,
    id::{marker::InteractionMarker, Id},
    user::User,
};

use crate::{
    desc_localizations, impl_guild_command_handle,
    interaction::{
        embed,
        response::InteractionResponse,
        util::{CustomId, GuildInteractionContext},
    },
    shard::BotState,
    translations::Lang,
    util::TextProcessExt,
};

/// Kick command model.
///
/// See the [`module`][self] documentation for more information.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "kick",
    desc = "Kicks a user from the server",
    desc_localizations = "kick_description",
    default_permissions = "KickCommand::default_permissions",
    dm_permission = false
)]
pub struct KickCommand {
    /// Member to kick.
    #[command(rename = "member")]
    pub user: ResolvedUser,
    /// Reason for kick.
    pub reason: Option<String>,
}

impl_guild_command_handle!(KickCommand);
desc_localizations!(kick_description);

impl KickCommand {
    fn default_permissions() -> Permissions {
        Permissions::KICK_MEMBERS
    }

    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &BotState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let user = self.user.resolved;
        let member = match self.user.member {
            Some(member) => member,
            None => return Ok(embed::kick::not_member(user.name, ctx.lang)),
        };

        // Fetch the author and the bot permissions.
        let permissions = state.cache.permissions(ctx.guild_id).await?;
        let author_permissions = permissions.member(ctx.author.id, &member.roles).await?;
        let member_permissions = permissions.member(user.id, &member.roles).await?;
        let bot_permissions = permissions.current_member().await?;

        // Check if the author and the bot have required permissions.
        if member_permissions.is_owner() {
            return Ok(embed::kick::member_owner(ctx.lang));
        }

        if !bot_permissions.guild().contains(Permissions::KICK_MEMBERS) {
            return Ok(embed::kick::bot_missing_permission(ctx.lang));
        }

        // Check if the role hierarchy allow the author and the bot to perform
        // the kick.
        let member_highest_role = member_permissions.highest_role();

        if member_highest_role >= author_permissions.highest_role() {
            return Ok(embed::kick::user_hierarchy(ctx.lang));
        }

        if member_highest_role >= bot_permissions.highest_role() {
            return Ok(embed::kick::bot_hierarchy(ctx.lang));
        }

        // Send reason modal.
        let enforce_reason = state
            .database
            .get_guild_or_create(ctx.guild_id)
            .await?
            .moderation
            .enforce_reason;

        match self.reason {
            Some(_reason) => Ok(InteractionResponse::EphemeralDeferredMessage),
            None => {
                KickCommand::reason_modal(ctx.interaction.id, user, enforce_reason, state, ctx.lang)
                    .await
            }
        }
    }

    /// Modal that asks the user to enter a reason for the kick.
    ///
    /// This modal is only shown if the user has not specified a reason in the
    /// initial command.
    async fn reason_modal(
        interaction_id: Id<InteractionMarker>,
        user: User,
        enforce_reason: bool,
        state: &BotState,
        lang: Lang,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let username = user.name.max_len(15);
        let components = vec![
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "reason".to_owned(),
                    label: lang.modal_kick_reason_label().to_owned(),
                    max_length: Some(100),
                    min_length: None,
                    placeholder: Some(lang.modal_reason_placeholder().to_owned()),
                    required: Some(enforce_reason),
                    style: TextInputStyle::Short,
                    value: None,
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "notes".to_owned(),
                    label: lang.modal_notes_label().to_owned(),
                    max_length: Some(1000),
                    min_length: None,
                    placeholder: Some(lang.modal_notes_placeholder().to_owned()),
                    required: Some(false),
                    style: TextInputStyle::Paragraph,
                    value: None,
                })],
            }),
        ];

        // Add pending component in Redis
        let custom_id = CustomId::new("sanction", interaction_id.to_string());
        let pending = PendingSanction {
            interaction_id,
            kind: ModlogType::Kick,
            user,
        };

        state.cache.set(&pending).await?;

        Ok(InteractionResponse::Modal {
            custom_id: custom_id.to_string(),
            title: lang.modal_kick_title(username),
            components,
        })
    }
}
