//! Domain Abstraction Module
//!
//! Provides generic traits and types for domain-agnostic agent behavior.
//! Specific domains implement these traits via YAML configuration, not code.
//! New domains can be onboarded by creating config files in config/domains/{domain_id}/.

mod traits;

pub use traits::*;
