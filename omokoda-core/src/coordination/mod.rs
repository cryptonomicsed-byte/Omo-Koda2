//! Talking to a Vantage guild: joining, claiming, delivering, proving.
//!
//! Vantage is the coordination layer — guilds are rooms, workspaces are rooms
//! with a repository attached, and work moves through them as claim and
//! artifact messages carrying a typed reference. This module is the kernel's
//! side of that contract.
//!
//! Three things it deliberately does not do.
//!
//! **It does not hand Vantage a key.** Joining is a keypair handshake: the
//! instance issues a challenge, this kernel signs it with the Nostr identity
//! derived from its own root seed, and Vantage learns a public key. That is
//! the whole point of the boundary, so there is no code path here that sends
//! a secret anywhere.
//!
//! **It does not invent references.** [`WorkRef`] mirrors Vantage's grammar
//! exactly, so a claim this kernel posts is one the instance can resolve into
//! a row rather than one it drops on the floor.
//!
//! **It does not re-sign receipts.** A receipt is whatever
//! [`crate::receipt::Receipt`] produced, submitted verbatim. Reshaping one to
//! suit an HTTP body would mean Vantage verifying something other than what
//! this kernel signed.

pub mod client;
pub mod presence;
pub mod work_ref;

pub use client::{CoordinationClient, CoordinationError, MessageType, PostedMessage};
pub use presence::WorkState;
pub use work_ref::{WorkKind, WorkRef, WorkRefError};
