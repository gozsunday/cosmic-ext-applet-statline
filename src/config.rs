// SPDX-License-Identifier: MPL-2.0

//! Persistent applet settings stored via cosmic-config (RON files).
//!
//! Minimal for now: a placeholder field keeps the config versioned while
//! sensor settings land in later stages.

use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

// ---------------------------------------------------------------------------
// Config struct
// ---------------------------------------------------------------------------

/// Versioned applet config (stored under the app id, v1).
#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    /// Placeholder value until real sensor settings are added.
    demo: String,
}
