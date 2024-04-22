#![doc = include_str!("../README.md")]

// TODO add support for wasm using relays.

// Rexports
pub use bytes;
pub use simple_dns as dns;

// Modules

mod cache;
mod client;
mod error;
mod keys;
mod signed_packet;

// Exports
pub use crate::client::PkarrClient;
pub use crate::error::Error;
pub use crate::keys::{Keypair, PublicKey};
pub use crate::signed_packet::SignedPacket;

/// Default minimum TTL: 30 seconds
pub const DEFAULT_MINIMUM_TTL: u32 = 30;
/// Default maximum TTL: 24 hours
pub const DEFAULT_MAXIMUM_TTL: u32 = 24 * 60 * 60;

// Alias Result to be the crate Result.
pub type Result<T, E = Error> = core::result::Result<T, E>;
