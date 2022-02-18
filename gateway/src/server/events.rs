use anyhow::Result;
use raidprotect_model::event::Event;
use raidprotect_transport::{model::EventBroadcastResponse, remoc::rch};
use raidprotect_util::shutdown::ShutdownSubscriber;
use tokio::sync::broadcast;
use tracing::{debug, error, instrument, trace, warn};

/// Event broadcast handler.
///
/// This type hold a [`rch::mpsc`] channel used to send events received by
/// the gateway to client services.
pub struct EventBroadcastHandler {
    /// Event sender stream.
    sender: rch::mpsc::Sender<Event>,
}

impl EventBroadcastHandler {
    /// Start an event broadcast handler.
    #[instrument(skip_all)]
    pub async fn start(
        mut events: broadcast::Receiver<Event>,
        callback: rch::oneshot::Sender<EventBroadcastResponse>,
        mut shutdown: ShutdownSubscriber,
    ) {
        let (sender, receiver) = rch::mpsc::channel(5);

        // Send the event stream to the client
        let res = EventBroadcastResponse { events: receiver };

        if let Err(err) = callback.send(res) {
            error!(error = ?err, "failed to send event stream channel");
            return;
        }

        // Start the handler
        let handler = EventBroadcastHandler { sender };

        tokio::select! {
            res = handler.handle_events(&mut events) => {
                if let Err(err) = res {
                    error!(error = %err, "event stream closed");
                }
            }
            _ = shutdown.wait_shutdown() => {
                debug!("shutting down");
            }
        };

        drop(handler.sender);
    }

    /// Handle incoming events
    async fn handle_events(&self, events: &mut broadcast::Receiver<Event>) -> Result<()> {
        loop {
            let event = events.recv().await?;
            let sender = self.sender.clone();

            tokio::spawn(async move {
                trace!(event = ?event, "sending event to client");

                if let Err(err) = sender.send(event).await {
                    warn!(error = %err, "error while sending event")
                }
            });
        }
    }
}
