//! Kafka inbound adapters implementing [`crate::application::ports::MessagingInboundHandler`].
//! Add sibling modules for richer handling, then `pub use` here.

mod logging;

pub use logging::LoggingMessagingInboundHandler;
