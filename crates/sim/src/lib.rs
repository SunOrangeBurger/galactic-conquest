//! Galactic Conquest engine core — Phase 0.
//!
//! Scope per design doc Section 0: tick loop support, state storage,
//! fog-of-war masking, action validation, galaxy generation, deterministic
//! combat. Nothing here should reach toward Section 6-10 (deferred systems).

pub mod fixed;
pub mod galaxy;
pub mod hashing;
pub mod integer;
pub mod state;
pub mod validation;
