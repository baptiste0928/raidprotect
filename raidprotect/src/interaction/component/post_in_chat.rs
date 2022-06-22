//! "Post in chat" button interaction component.
//!
//! This module implement the "Post in chat" button, that allow users to post
//! in the channel an ephemeral response.

use nanoid::nanoid;
use raidprotect_model::cache::model::interaction::{PendingComponent, PostInChatButton};
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::{message::MessageFlags, ReactionType},
    http::interaction::{InteractionResponseData, InteractionResponseType},
    id::{marker::UserMarker, Id},
};

use crate::{
    cluster::ClusterState, interaction::response::InteractionResponse, translations::Lang,
};

/// Adds a button to post in a channel an ephemeral response.
pub struct PostInChat;

impl PostInChat {
    /// Create a new [`PostInChat`] component.
    pub async fn create(
        mut response: InteractionResponseData,
        author_id: Id<UserMarker>,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        // Store button state in redis
        let custom_id = nanoid!();
        let component = PendingComponent::PostInChat(PostInChatButton {
            id: custom_id.clone(),
            response: response.clone(),
            author_id,
        });

        state.redis().set(&component).await?;

        // Add ephemeral flag to the response
        response.flags = response
            .flags
            .map(|flags| flags | MessageFlags::EPHEMERAL)
            .or(Some(MessageFlags::EPHEMERAL));

        // Add post in chat button.
        let button = Component::Button(Button {
            custom_id: Some(custom_id),
            disabled: false,
            emoji: Some(ReactionType::Unicode {
                name: "💬".to_string(),
            }),
            label: Some(Lang::Fr.post_in_chat_button().to_string()),
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
    pub fn handle(mut component: PostInChatButton) -> InteractionResponse {
        // Remove ephemeral flag
        if let Some(flags) = component.response.flags.as_mut() {
            flags.set(MessageFlags::EPHEMERAL, false);
        }

        component.response.content = Some(Lang::Fr.post_in_chat_author(component.author_id));

        InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(component.response),
        }
    }
}
