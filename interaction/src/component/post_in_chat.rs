//! "Post in chat" button interaction component.
//!
//! This module implement the "Post in chat" button, that allow users to post
//! in the channel an ephemeral response.

use nanoid::nanoid;
use raidprotect_cache::{
    model::component::{PendingComponent, PostInChatButton},
    redis::RedisClientError,
};
use raidprotect_state::ClusterState;
use raidprotect_translations::Lang;
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::{message::MessageFlags, ReactionType},
    http::interaction::{
        InteractionResponse as HttpInteractionResponse, InteractionResponseData,
        InteractionResponseType,
    },
    id::{marker::UserMarker, Id},
};

use crate::response::IntoResponse;

pub struct PostInChat {
    /// The message to post.
    response: InteractionResponseData,
    /// Button custom id.
    custom_id: String,
}

impl PostInChat {
    pub async fn new(
        response: InteractionResponseData,
        author_id: Id<UserMarker>,
        state: &ClusterState,
    ) -> Result<Self, RedisClientError> {
        let custom_id = nanoid!();
        let component = PendingComponent::PostInChatButton(PostInChatButton {
            id: custom_id.clone(),
            response: response.clone(),
            author_id,
        });

        state.redis().set(&component).await?;

        Ok(Self {
            response,
            custom_id,
        })
    }

    pub fn handle(mut component: PostInChatButton) -> HttpInteractionResponse {
        // Remove ephemeral flag
        if let Some(flags) = component.response.flags.as_mut() {
            flags.set(MessageFlags::EPHEMERAL, false);
        }

        component.response.content = Some(Lang::Fr.post_in_chat_author(component.author_id));

        HttpInteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(component.response),
        }
    }
}

impl IntoResponse for PostInChat {
    fn into_response(mut self) -> HttpInteractionResponse {
        // Add ephemeral flag.
        self.response.flags = self
            .response
            .flags
            .map(|flags| flags | MessageFlags::EPHEMERAL)
            .or(Some(MessageFlags::EPHEMERAL));

        // Add post in chat button.
        let button = Component::Button(Button {
            custom_id: Some(self.custom_id),
            disabled: false,
            emoji: Some(ReactionType::Unicode {
                name: "💬".to_string(),
            }),
            label: Some(Lang::Fr.post_in_chat_button().to_string()),
            style: ButtonStyle::Primary,
            url: None,
        });

        if let Some(components) = self.response.components.as_mut() {
            if let Some(Component::ActionRow(action_row)) = components.first_mut() {
                action_row.components.insert(0, button);
            }
        } else {
            self.response.components = Some(vec![Component::ActionRow(ActionRow {
                components: vec![button],
            })])
        }

        HttpInteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(self.response),
        }
    }
}
