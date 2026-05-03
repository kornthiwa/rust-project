//! Kafka-side adapters for [`crate::application::ports::AuthEventInboundHandler`].
//! Add more as sibling modules (e.g. `projection.rs`): `mod projection;` then `pub use` here;
//! parent `messaging` re-exports what [`crate::app::bootstrap`] wires.

mod logging;

pub use logging::LoggingAuthEventInboundHandler;
