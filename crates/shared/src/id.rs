//! Typed string identifiers.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A string identifier tagged with the kind of thing it names, so an `Id<A>` can't be passed where
/// an `Id<B>` is expected.
///
/// ```compile_fail,E0308
/// use shared::id::Id;
///
/// enum Apple {}
/// enum Pear {}
///
/// let apple: Id<Apple> = Id::new("a");
/// let pear: Id<Pear> = apple;
/// ```
pub struct Id<T> {
    value: String,
    kind: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), kind: PhantomData }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

// Written by hand so the impls don't require `T` itself to implement them.
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
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
        write!(f, "Id<{kind}>({:?})", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::Id;

    enum Apple {}

    #[test]
    fn ids_with_the_same_value_are_equal() {
        assert_eq!(Id::<Apple>::new("a"), Id::<Apple>::new("a"));
        assert_ne!(Id::<Apple>::new("a"), Id::<Apple>::new("b"));
    }

    #[test]
    fn as_str_returns_the_value() {
        assert_eq!(Id::<Apple>::new("a").as_str(), "a");
    }

    #[test]
    fn debug_names_the_kind() {
        assert_eq!(format!("{:?}", Id::<Apple>::new("a")), r#"Id<Apple>("a")"#);
    }
}
