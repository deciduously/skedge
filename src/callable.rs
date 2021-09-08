//! The work functions that can be scheduled must implement the `Callable` trait.

use std::fmt;

/// A job is anything that implements this trait
pub(crate) trait Callable {
    /// Execute this callable
    fn call(&self) -> Option<bool>;
    /// Get the name of this callable
    fn name(&self) -> &str;
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Callable(name={})", self.name())
    }
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

/// A named callable function taking no parameters and returning nothing.
#[derive(Debug)]
pub struct OneToUnit<T>
where
    T: Clone,
{
    name: String,
    work: fn(T) -> (),
    arg: T,
}

impl<T> OneToUnit<T>
where
    T: Clone,
{
    pub fn new(name: &str, work: fn(T) -> (), arg: T) -> Self {
        Self {
            name: name.into(),
            work,
            arg,
        }
    }
}

impl<T> Callable for OneToUnit<T>
where
    T: Clone,
{
    fn call(&self) -> Option<bool> {
        (self.work)(self.arg.clone());
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A named callable function taking no parameters and returning nothing.
#[derive(Debug)]
pub struct TwoToUnit<T, U>
where
    T: Clone,
    U: Clone
{
    name: String,
    work: fn(T, U) -> (),
    arg_one: T,
    arg_two: U
}

impl<T, U> TwoToUnit<T, U>
where
    T: Clone,
    U: Clone
{
    pub fn new(name: &str, work: fn(T, U) -> (), arg_one: T, arg_two: U) -> Self {
        Self {
            name: name.into(),
            work,
            arg_one,
            arg_two
        }
    }
}

impl<T, U> Callable for TwoToUnit<T, U>
where
    T: Clone,
    U: Clone
{
    fn call(&self) -> Option<bool> {
        (self.work)(self.arg_one.clone(), self.arg_two.clone());
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}
