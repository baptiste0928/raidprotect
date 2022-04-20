//! "Post in chat" button interaction component.
//!
//! This module implement the "Post in chat" button, that allow users to post
//! in the channel an ephemeral response.

use nanoid::nanoid;
use raidprotect_model::interaction::InteractionResponse;
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::{message::MessageFlags, ReactionType},
    http::interaction::InteractionResponseData,
};

use crate::interaction::response::IntoResponse;

pub struct PostInChat {
    /// The message to post.
    message: InteractionResponse,
    /// Button custom id.
    custom_id: String,
}

impl PostInChat {
    pub fn new(message: InteractionResponse) -> Self {
        let custom_id = nanoid!();

        Self { message, custom_id }
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
            label: Some("Envoyer dans le salon".into()),
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
