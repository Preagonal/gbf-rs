#![deny(missing_docs)]
use std::{cmp::Ordering, collections::HashMap, fmt::Display, ops::AddAssign};

use serde::{Deserialize, Serialize};

/// Represents the SSA version of a variable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub struct SsaVersion(usize);

/// Context for the SSA transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsaContext {
    /// The current versions of variables.
    pub current_versions: HashMap<String, SsaVersion>,
    /// The next version to assign.
    pub next_version: SsaVersion,
}

impl SsaContext {
    /// Creates a new `SsaContext`.
    pub fn new() -> Self {
        Self {
            current_versions: HashMap::new(),
            next_version: 0.into(),
        }
    }

    /// Creates a new SSA version for the given variable.
    pub fn new_ssa_version_for(&mut self, location: &str) -> SsaVersion {
        let version = self.next_version;
        self.next_version += 1;

        self.current_versions.insert(location.to_string(), version);
        version
    }

    /// Returns the current version of the given variable.
    pub fn current_version_of(&self, location: &str) -> Option<SsaVersion> {
        self.current_versions.get(location).copied()
    }

    /// Returns the current version of the given variable, or creates a new one if it doesn't exist.
    pub fn current_version_of_or_new(&mut self, location: &str) -> SsaVersion {
        self.current_versions
            .get(location)
            .copied()
            .unwrap_or_else(|| self.new_ssa_version_for(location))
    }
}

// == Other implementations ==
impl AddAssign for SsaVersion {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<usize> for SsaVersion {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl From<SsaVersion> for usize {
    fn from(value: SsaVersion) -> Self {
        value.0
    }
}

impl From<usize> for SsaVersion {
    fn from(value: usize) -> Self {
        SsaVersion(value)
    }
}

impl PartialOrd for SsaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for SsaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for SsaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SsaContext {
    fn default() -> Self {
        Self::new()
    }
}
