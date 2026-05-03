//! Kafka inbound adapters for [`crate::application::ports::UserEventInboundHandler`].

mod logging;

pub use logging::LoggingUserEventInboundHandler;
