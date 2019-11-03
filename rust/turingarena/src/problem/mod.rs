#![doc(include = "README.md")]

use serde::{Deserialize, Serialize};
use super::juniper_ext::*;

pub mod driver;
pub mod material;

/// Wraps a string representing the name of a problem.
/// Used only to identify a problem. Should never be shown to (non-admin) users.
///
/// Represents the default name used to identify a problem within a contest,
/// in a way similar to names of Cargo dependencies.
/// It could be overidden (as with the `package` option in `Cargo.toml`),
/// in the rare case two problems with the same name are desired in the same contest
/// (or two versions of the same problem).
#[derive(Debug, Serialize, Deserialize, Clone, GraphQLNewtype)]
pub struct ProblemName(pub String);
