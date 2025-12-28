//! Core game systems and types for EVE Rebellion
//!
//! This module contains the fundamental building blocks:
//! - Game states and transitions
//! - Shared resources (score, currency)
//! - Custom events
//! - Game constants

pub mod game_state;
pub mod resources;
pub mod events;
pub mod constants;
pub mod campaign;

pub use game_state::*;
pub use resources::*;
pub use events::*;
pub use constants::*;
pub use campaign::*;
