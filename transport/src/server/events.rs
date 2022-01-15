use raidprotect_model::event::Event;
use raidprotect_util::shutdown::ShutdownSubscriber;
use remoc::rch;
use tokio::sync::broadcast;
use tracing::instrument;

use crate::model::{EventBroadcast, EventBroadcastResponse};

pub struct EventBroadcastHandler {
    /// Event receiver stream.
    events: broadcast::Receiver<Event>,
    /// Event sender stream.
    sender: rch::mpsc::Sender<EventBroadcast>,
}

impl EventBroadcastHandler {
    /// Start an event broadcast handler.
    #[instrument(skip_all)]
    pub async fn start(
        events: broadcast::Receiver<Event>,
        callback: rch::oneshot::Sender<EventBroadcastResponse>,
        mut shutdown: ShutdownSubscriber,
    ) {
        todo!()
    }
}
