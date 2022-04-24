//! "Post in chat" button interaction component.
//!
//! This module implement the "Post in chat" button, that allow users to post
//! in the channel an ephemeral response.

use nanoid::nanoid;
use raidprotect_model::{
    interaction::{
        component::{PendingComponent, PostInChatButton},
        InteractionResponse,
    },
    ClusterState,
};
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::{message::MessageFlags, ReactionType},
    http::interaction::InteractionResponseData,
    id::{marker::UserMarker, Id},
};

use crate::{response::IntoResponse, translations::Lang};

pub struct PostInChat {
    /// The message to post.
    message: InteractionResponse,
    /// Button custom id.
    custom_id: String,
}

impl PostInChat {
    pub async fn new(
        message: InteractionResponse,
        author_id: Id<UserMarker>,
        state: &ClusterState,
    ) -> Self {
        let custom_id = nanoid!();

        state
            .pending_components()
            .insert(
                custom_id.clone(),
                PendingComponent::PostInChatButton(PostInChatButton {
                    response: message.clone(),
                    author_id,
                }),
            )
            .await;

        Self { message, custom_id }
    }

    pub fn handle(component: PostInChatButton) -> InteractionResponseData {
        let mut response = component.response.into_response();

        // Remove ephemeral flag
        if let Some(flags) = response.flags.as_mut() {
            flags.set(MessageFlags::EPHEMERAL, false);
        }

        response.content = Some(Lang::Fr.post_in_chat_author(component.author_id));

        response
    }
}

impl IntoResponse for PostInChat {
    fn into_response(self) -> InteractionResponseData {
        let mut response = self.message.into_response();

        // Add ephemeral flag.
        response.flags = response
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

        if let Some(components) = response.components.as_mut() {
            if let Some(Component::ActionRow(action_row)) = components.first_mut() {
                action_row.components.insert(0, button);
            }
        } else {
            response.components = Some(vec![Component::ActionRow(ActionRow {
                components: vec![button],
            })])
        }

        response
    }
}
