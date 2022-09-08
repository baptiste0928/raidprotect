//! "Post in chat" button interaction component.
//!
//! This module implement the "Post in chat" button, that allow users to post
//! in the channel an ephemeral response.

use anyhow::anyhow;
use raidprotect_model::cache::model::interaction::PostInChatButton;
use twilight_model::{
    application::{
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::Interaction,
    },
    channel::{message::MessageFlags, ReactionType},
    http::interaction::{InteractionResponseData, InteractionResponseType},
    id::{
        marker::{InteractionMarker, UserMarker},
        Id,
    },
};

use crate::{
    cluster::ClusterState,
    interaction::{
        embed,
        response::InteractionResponse,
        util::{CustomId, GuildConfigExt, GuildInteractionContext},
    },
    translations::Lang,
};

/// Adds a button to post in a channel an ephemeral response.
pub struct PostInChat;

impl PostInChat {
    /// Create a new [`PostInChat`] component.
    pub async fn create(
        mut response: InteractionResponseData,
        interaction_id: Id<InteractionMarker>,
        author_id: Id<UserMarker>,
        state: &ClusterState,
        lang: Lang,
    ) -> Result<InteractionResponse, anyhow::Error> {
        // Store button state in redis
        let component = PostInChatButton {
            response: response.clone(),
            interaction_id,
            author_id,
        };

        state.cache.set(&component).await?;

        // Add ephemeral flag to the response
        response.flags = response
            .flags
            .map(|flags| flags | MessageFlags::EPHEMERAL)
            .or(Some(MessageFlags::EPHEMERAL));

        // Add post in chat button.
        let custom_id = CustomId::new("post-in-chat", interaction_id.to_string());
        let button = Component::Button(Button {
            custom_id: Some(custom_id.to_string()),
            disabled: false,
            emoji: Some(ReactionType::Unicode {
                name: "💬".to_string(),
            }),
            label: Some(lang.post_in_chat_button().to_string()),
            style: ButtonStyle::Primary,
            url: None,
        });

        if let Some(components) = response.components.as_mut() {
            if let Some(Component::ActionRow(action_row)) = components.first_mut() {
                action_row.components.insert(0, button);
            }
        } else {
            response.components = Some(vec![Component::ActionRow(ActionRow {
                components: vec![button],
            })])
        }

        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }

    /// Handle the button click.
    pub async fn handle(
        interaction: Interaction,
        custom_id: CustomId,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let ctx = GuildInteractionContext::new(interaction)?;

        // Fetch component from redis
        let component_id = custom_id
            .id
            .ok_or_else(|| anyhow!("missing component id in custom_id"))?;
        let mut component = match state.cache.get::<PostInChatButton>(&component_id).await? {
            Some(component) => component,
            None => return Ok(embed::error::expired_interaction(ctx.lang)),
        };

        // Remove ephemeral flag
        if let Some(flags) = component.response.flags.as_mut() {
            flags.set(MessageFlags::EPHEMERAL, false);
        }

        let config = ctx.config(state).await?;
        component.response.content = Some(config.lang().post_in_chat_author(component.author_id));

        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(component.response),
        })
    }
}
