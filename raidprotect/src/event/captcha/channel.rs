//! Handle channel-related events.

use tracing::error;
use twilight_model::channel::Channel;

use crate::cluster::ClusterState;

/// Handle `ChannelUpdate` event.
pub async fn channel_update(channel: &Channel, state: &ClusterState) {
    if let Err(error) = channel_update_inner(channel, state).await {
        error!(
            ?error,
            ?channel,
            "error while processing `ChannelUpdate` event (captcha)"
        )
    }
}

async fn channel_update_inner(
    _channel: &Channel,
    _state: &ClusterState,
) -> Result<(), anyhow::Error> {
    Ok(())
}
