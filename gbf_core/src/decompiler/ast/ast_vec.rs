#![deny(missing_docs)]

use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// Represents a vector of AST nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AstVec<T>(pub Vec<T>);

impl<T, U> From<AstVec<T>> for Vec<U>
where
    T: Into<U>,
{
    fn from(wrapper: AstVec<T>) -> Self {
        wrapper.0.into_iter().map(Into::into).collect()
    }
}

impl<T, U> From<Vec<U>> for AstVec<T>
where
    U: Into<T>,
{
    fn from(vec: Vec<U>) -> Self {
        AstVec(vec.into_iter().map(Into::into).collect())
    }
}

impl<T> FromIterator<T> for AstVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        AstVec(vec)
    }
}

impl<T> Deref for AstVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
