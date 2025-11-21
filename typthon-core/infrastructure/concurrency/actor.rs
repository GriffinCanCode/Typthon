//! Actor-based concurrency for type checking and analysis
//!
//! Provides lightweight actors for coordinating concurrent work with message passing.
//! Uses bounded channels for backpressure and structured supervision.

use std::sync::Arc;
use flume::{Sender, Receiver, bounded, unbounded};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt;

/// Unique actor identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId(u64);

impl ActorId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Actor message trait - all messages must implement this
pub trait Message: Send + 'static {
    type Response: Send + 'static;
}

/// Actor behavior trait
#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    type Message: Message;

    /// Handle incoming message
    async fn handle(&mut self, msg: Self::Message) -> <Self::Message as Message>::Response;

    /// Called when actor starts
    async fn started(&mut self) {}

    /// Called when actor stops
    async fn stopped(&mut self) {}
}

/// Actor address for sending messages
pub struct ActorAddr<A: Actor> {
    id: ActorId,
    sender: Sender<ActorEnvelope<A>>,
}

impl<A: Actor> Clone for ActorAddr<A> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            sender: self.sender.clone(),
        }
    }
}

impl<A: Actor> ActorAddr<A> {
    /// Send message and wait for response
    pub async fn send(&self, msg: A::Message) -> Result<<A::Message as Message>::Response, ActorError>
    where
        A::Message: Message,
    {
        let (tx, rx) = bounded(1);
        let envelope = ActorEnvelope {
            message: msg,
            response: tx,
        };

        self.sender.send_async(envelope).await
            .map_err(|_| ActorError::Disconnected)?;

        rx.recv_async().await
            .map_err(|_| ActorError::Disconnected)
    }

    /// Send message without waiting for response (fire and forget)
    pub fn try_send(&self, msg: A::Message) -> Result<(), ActorError>
    where
        A::Message: Message,
    {
        let (tx, _rx) = bounded(1);
        let envelope = ActorEnvelope {
            message: msg,
            response: tx,
        };

        self.sender.try_send(envelope)
            .map_err(|_| ActorError::MailboxFull)
    }

    pub fn id(&self) -> ActorId {
        self.id
    }
}

/// Internal envelope for messages with response channels
struct ActorEnvelope<A: Actor> {
    message: A::Message,
    response: Sender<<A::Message as Message>::Response>,
}

/// Actor context managing lifecycle and mailbox
pub struct ActorContext<A: Actor> {
    id: ActorId,
    actor: A,
    receiver: Receiver<ActorEnvelope<A>>,
}

impl<A: Actor> ActorContext<A> {
    async fn run(mut self) {
        self.actor.started().await;

        while let Ok(envelope) = self.receiver.recv_async().await {
            let response = self.actor.handle(envelope.message).await;
            let _ = envelope.response.send(response);
        }

        self.actor.stopped().await;
    }
}

/// Actor system managing all actors
pub struct ActorSystem {
    next_id: Arc<RwLock<u64>>,
    runtime: tokio::runtime::Handle,
}

impl ActorSystem {
    /// Create new actor system with existing Tokio runtime
    pub fn new(runtime: tokio::runtime::Handle) -> Self {
        Self {
            next_id: Arc::new(RwLock::new(0)),
            runtime,
        }
    }

    /// Spawn actor with bounded mailbox
    pub fn spawn<A: Actor>(&self, actor: A, mailbox_size: usize) -> ActorAddr<A> {
        let id = ActorId::new({
            let mut next = self.next_id.write();
            let id = *next;
            *next += 1;
            id
        });

        let (tx, rx) = if mailbox_size > 0 {
            bounded(mailbox_size)
        } else {
            unbounded()
        };

        let context = ActorContext {
            id,
            actor,
            receiver: rx,
        };

        self.runtime.spawn(context.run());

        ActorAddr {
            id,
            sender: tx,
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new(tokio::runtime::Handle::current())
    }
}

/// Actor errors
#[derive(Debug, Clone)]
pub enum ActorError {
    Disconnected,
    MailboxFull,
    Timeout,
}

impl fmt::Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "actor disconnected"),
            Self::MailboxFull => write!(f, "actor mailbox full"),
            Self::Timeout => write!(f, "actor message timeout"),
        }
    }
}

impl std::error::Error for ActorError {}

/// Supervision strategy for actor failures
#[derive(Debug, Clone, Copy)]
pub enum SupervisionStrategy {
    /// Restart the actor
    Restart,
    /// Stop the actor
    Stop,
    /// Escalate to parent supervisor
    Escalate,
}

/// Supervisor managing child actors
pub struct Supervisor {
    strategy: SupervisionStrategy,
    children: Arc<RwLock<HashMap<ActorId, Box<dyn std::any::Any + Send>>>>,
}

impl Supervisor {
    pub fn new(strategy: SupervisionStrategy) -> Self {
        Self {
            strategy,
            children: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn supervise<A: Actor>(&self, _addr: &ActorAddr<A>) {
        // Track child actor for supervision
        // In a full implementation, this would monitor the actor and apply strategy on failure
    }

    pub fn strategy(&self) -> SupervisionStrategy {
        self.strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter {
        count: i32,
    }

    enum CounterMsg {
        Increment,
        Get,
    }

    impl Message for CounterMsg {
        type Response = i32;
    }

    #[async_trait::async_trait]
    impl Actor for Counter {
        type Message = CounterMsg;

        async fn handle(&mut self, msg: CounterMsg) -> i32 {
            match msg {
                CounterMsg::Increment => {
                    self.count += 1;
                    self.count
                }
                CounterMsg::Get => self.count,
            }
        }
    }

    #[tokio::test]
    async fn test_actor_system() {
        let rt = tokio::runtime::Handle::current();
        let system = ActorSystem::new(rt);

        let counter = Counter { count: 0 };
        let addr = system.spawn(counter, 10);

        let result = addr.send(CounterMsg::Increment).await.unwrap();
        assert_eq!(result, 1);

        let result = addr.send(CounterMsg::Get).await.unwrap();
        assert_eq!(result, 1);
    }
}

