//! The work functions that can be scheduled must implement the `Callable` trait.

use std::fmt;

/// A job is anything that implements this trait
pub(crate) trait Callable: fmt::Debug {
    /// Execute this callable
    fn call(&self) -> Option<bool>;
    /// Get the name of this callable
    fn name(&self) -> &str;
}

impl PartialEq for dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        // Callable objects are equal if their names are equal
        // FIXME: this seems fishy
        self.name() == other.name()
    }
}

impl Eq for dyn Callable {}

/// A named callable function taking no parameters and returning nothing.
#[derive(Debug)]
pub struct UnitToUnit {
    name: String,
    work: fn() -> (),
}

impl UnitToUnit {
    pub fn new(name: &str, work: fn() -> ()) -> Self {
        Self {
            name: name.into(),
            work,
        }
    }
}

impl Callable for UnitToUnit {
    fn call(&self) -> Option<bool> {
        (self.work)();
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}
