#![doc(include = "README.md")]

extern crate juniper;

use super::content::*;
use serde::{Deserialize, Serialize};

graphql_derive_newtype! {
    /// Wraps a number that represents a score
    #[derive(Serialize, Deserialize, Copy, Clone)]
    pub struct Score(pub f64);
}

graphql_derive_newtype! {
    /// Wraps a string that identifies an award
    #[derive(Serialize, Deserialize, Clone)]
    pub struct AwardName(pub String);
}

/// Describes the possible values of a score.
#[derive(Serialize, Deserialize, Copy, Clone, juniper::GraphQLObject)]
pub struct ScoreRange {
    /// Number of significant decimal places.
    // FIXME: should be u8, but it is not trivially supported by juniper
    pub precision: i32,
    /// Maximum score.
    pub max: Score,
    /// Whether partial scores are allowed. If `false`, score must be either zero or `max`.
    pub allow_partial: bool,
}

/// An award that has a numerical score
#[derive(Serialize, Deserialize, Copy, Clone, juniper::GraphQLObject)]
pub struct ScoreAwardContent {
    pub range: ScoreRange,
}

graphql_derive_object_from_unit! {
    /// An award that has only two possible states (success or fail)
    #[derive(Serialize, Deserialize, Copy, Clone)]
    pub struct BadgeAwardContent;
}

graphql_derive_union_from_enum! {
    /// Describes the nature of an award
    #[derive(Serialize, Deserialize, Copy, Clone)]
    pub enum AwardContent {
        Score(ScoreAwardContent),
        Badge(BadgeAwardContent),
    }
}

/// Describes an item to which a score can be assigned
#[derive(Serialize, Deserialize, Clone, juniper::GraphQLObject)]
pub struct Award {
    /// Name of this award, used to identify it.
    /// Should never be shown to (non-admin) users.
    pub name: AwardName,
    /// Name of this award, as shown to users.
    pub title: Text,
    /// Content of this award.
    pub content: AwardContent,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
