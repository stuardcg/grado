//! Typed string identifiers.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::text::NonBlankString;

/// A string identifier tagged with the kind of thing it names, so an `Id<A>` can't be passed where
/// an `Id<B>` is expected. It is never blank (ADR 0007).
///
/// Tag each kind with its own type, usually behind an alias:
///
/// ```
/// use shared::id::Id;
///
/// enum Apple {}
/// type AppleId = Id<Apple>;
///
/// let apple = AppleId::new("a").expect("not blank");
/// assert_eq!(apple.as_str(), "a");
/// assert_eq!(Some(apple), AppleId::new("a"));
/// assert_eq!(AppleId::new(" "), None);
/// ```
///
/// Ids of different kinds don't mix, even when they hold the same value:
///
/// ```compile_fail,E0308
/// use shared::id::Id;
///
/// enum Apple {}
/// enum Pear {}
///
/// let apple: Id<Apple> = Id::new("a").expect("not blank");
/// let pear: Id<Pear> = apple;
/// ```
pub struct Id<T> {
    value: NonBlankString,
    kind: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    /// Returns `None` when `value` is empty or only whitespace.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        NonBlankString::new(value).map(|value| Self { value, kind: PhantomData })
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

// Written by hand so the impls don't require `T` itself to implement them.
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self { value: self.value.clone(), kind: PhantomData }
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

// Names the kind, so ids of different kinds can be told apart in logs: `Id<Player>("p-1")`.
impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = std::any::type_name::<T>();
        let kind = path.rsplit("::").next().unwrap_or(path);
        write!(f, "Id<{kind}>({:?})", self.value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Id;

    enum Apple {}

    fn apple(value: &str) -> Id<Apple> {
        Id::new(value).expect("test ids aren't blank")
    }

    #[test]
    fn ids_with_the_same_value_are_equal() {
        assert_eq!(apple("a"), apple("a"));
        assert_ne!(apple("a"), apple("b"));
    }

    #[test]
    fn as_str_returns_the_value() {
        assert_eq!(apple("a").as_str(), "a");
    }

    #[test]
    fn debug_names_the_kind() {
        assert_eq!(format!("{:?}", apple("a")), r#"Id<Apple>("a")"#);
    }

    #[test]
    fn blank_ids_are_refused() {
        assert_eq!(Id::<Apple>::new(""), None);
        assert_eq!(Id::<Apple>::new("  "), None);
    }
}
