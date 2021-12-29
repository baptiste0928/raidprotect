//! Shutdown handling.
//!
//! These types are used to manage graceful shutdown of running tasks.
//!
//! ## Shutdown strategy
//! All running tasks can be represented as a dependency tree. When a shutdown
//! is requested from a top-level task, this task will forward the shutdown
//! signal to all subtasks (for example, connections of a server) and wait until
//! they gracefully shutdown, with a timeout to avoid blocking shutdown.
//!
//! This strategy is applied recursively until all tasks gracefully shutdown.
//!
//! ## Usage
//! Each task that will have dependencies should initialize a [`Shutdown`] type.
//! This type will be used to sent shutdown signal to child tasks. The child tasks
//! are tracked with a [`ShutdownSubscriber`] type, that is used both to receive the
//! shutdown signal and know when sub-tasks have stopped.
//!
//! Internally, a [`broadcast`] channel is used to emit shutdown signal, and a [`mpsc`]
//! channel is used to know when all subtasks are dropped.

use std::{io, time::Duration};

use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};

/// Wait until a shutdown signal is received.
///
/// This method will complete when one of the following signal
/// is received: `SIGINT`, `SIGQUIT`, `SIGTERM`.
#[cfg(unix)]
pub async fn wait_shutdown() -> io::Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigquit = signal(SignalKind::quit())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => (),
        _ = sigquit.recv() => (),
        _ = sigterm.recv() => (),
    };

    Ok(())
}

/// Wait until a shutdown signal is received.
///
/// This method will complete when a `Ctrl-C` signal is received.
#[cfg(not(unix))]
pub async fn wait_shutdown() -> io::Result<()> {
    tokio::signal::ctrl_c().await
}

/// Shutdown manager.
///
/// This type should be initialized by each task that depend on
/// other subtasks. It allow to notify them when performing a shutdown
/// and also waiting until they gracefully stopped.
#[derive(Debug)]
pub struct Shutdown {
    /// This sender is used to emit shutdown signal. The sender is
    /// dropped on shutdown to send a signal to all remaining receivers.
    notify: broadcast::Sender<()>,
    /// This sender is used to track subtasks. When all senders are dropped,
    /// a signal is sent to the receiver.
    sender: mpsc::Sender<()>,
    receiver: mpsc::Receiver<()>,
}

impl Shutdown {
    /// Initialize a [`Shutdown`] manager.
    pub fn new() -> Self {
        let (notify, _) = broadcast::channel(1);
        let (sender, receiver) = mpsc::channel(1);

        Self {
            notify,
            sender,
            receiver,
        }
    }

    /// Create a new [`ShutdownSubscriber`].
    ///
    /// The returned notifier can be sent to subtasks to allow
    /// them to gracefully shutdown when requested. It is also used
    /// to track when all tasks are dropped.
    pub fn subscriber(&self) -> ShutdownSubscriber {
        ShutdownSubscriber {
            shutdown: false,
            notify: self.notify.subscribe(),
            _sender: self.sender.clone(),
        }
    }

    /// Emit a shutdown signal.
    ///
    /// When called, a shutdown signal is sent to all subtasks.
    /// The function returns when all subtasks have gracefully
    /// stopped or when the timeout is expired.
    #[must_use = "shutdown has no effect if no signal is emitted"]
    pub async fn shutdown(self, timeout: u64) -> bool {
        // Extract channels to allow dropping them.
        let Shutdown {
            notify,
            sender,
            mut receiver,
        } = self;

        drop(notify); // Notify shutdown to subscribers
        drop(sender); // Remaining senders are those held by subscribers

        // Wait until all tasks are finished, or timeout is elapsed
        tokio::select! {
            _ = receiver.recv() => true,
            _ = sleep(Duration::from_secs(timeout)) => false
        }
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

/// Shutdown subscriber.
///
/// This type is created from a [`Shutdown`] and is used to be notified
/// of shutdowns. It also tracks the completion of subtasks and notify the
/// shutdown notifier when all subscribers are dropped.
#[derive(Debug)]
pub struct ShutdownSubscriber {
    /// Whether a shutdown signal has been received.
    shutdown: bool,
    /// Receiver of shutdown requests.
    notify: broadcast::Receiver<()>,
    /// Sender used to notify when the task is dropped.
    _sender: mpsc::Sender<()>,
}

impl ShutdownSubscriber {
    /// Returns `true` if the shutdown signal has been received.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Wait until a shutdown signal is received.
    #[must_use = "subscribers should listen to shutdown signal"]
    pub async fn wait_shutdown(&mut self) {
        if self.shutdown {
            return;
        }

        let _ = self.notify.recv().await;
        self.shutdown = true
    }
}
