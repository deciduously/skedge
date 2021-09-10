//! The work functions that can be scheduled must implement the `Callable` trait.

use std::fmt;

/// A job is anything that implements this trait
pub trait Callable {
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

/// A named callable function taking one parameter and returning nothing.
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

/// A named callable function taking two parameters and returning nothing.
#[derive(Debug)]
pub struct TwoToUnit<T, U>
where
    T: Clone,
    U: Clone,
{
    name: String,
    work: fn(T, U) -> (),
    arg_one: T,
    arg_two: U,
}

impl<T, U> TwoToUnit<T, U>
where
    T: Clone,
    U: Clone,
{
    pub fn new(name: &str, work: fn(T, U) -> (), arg_one: T, arg_two: U) -> Self {
        Self {
            name: name.into(),
            work,
            arg_one,
            arg_two,
        }
    }
}

impl<T, U> Callable for TwoToUnit<T, U>
where
    T: Clone,
    U: Clone,
{
    fn call(&self) -> Option<bool> {
        (self.work)(self.arg_one.clone(), self.arg_two.clone());
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A named callable function taking three parameters and returning nothing.
#[derive(Debug)]
pub struct ThreeToUnit<T, U, V>
where
    T: Clone,
    U: Clone,
    V: Clone,
{
    name: String,
    work: fn(T, U, V) -> (),
    arg_one: T,
    arg_two: U,
    arg_three: V,
}

impl<T, U, V> ThreeToUnit<T, U, V>
where
    T: Clone,
    U: Clone,
    V: Clone,
{
    pub fn new(name: &str, work: fn(T, U, V) -> (), arg_one: T, arg_two: U, arg_three: V) -> Self {
        Self {
            name: name.into(),
            work,
            arg_one,
            arg_two,
            arg_three,
        }
    }
}

impl<T, U, V> Callable for ThreeToUnit<T, U, V>
where
    T: Clone,
    U: Clone,
    V: Clone,
{
    fn call(&self) -> Option<bool> {
        (self.work)(
            self.arg_one.clone(),
            self.arg_two.clone(),
            self.arg_three.clone(),
        );
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A named callable function taking three parameters and returning nothing.
#[derive(Debug)]
pub struct FourToUnit<T, U, V, W>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
{
    name: String,
    work: fn(T, U, V, W) -> (),
    arg_one: T,
    arg_two: U,
    arg_three: V,
    arg_four: W,
}

impl<T, U, V, W> FourToUnit<T, U, V, W>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
{
    pub fn new(
        name: &str,
        work: fn(T, U, V, W) -> (),
        arg_one: T,
        arg_two: U,
        arg_three: V,
        arg_four: W,
    ) -> Self {
        Self {
            name: name.into(),
            work,
            arg_one,
            arg_two,
            arg_three,
            arg_four,
        }
    }
}

impl<T, U, V, W> Callable for FourToUnit<T, U, V, W>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
{
    fn call(&self) -> Option<bool> {
        (self.work)(
            self.arg_one.clone(),
            self.arg_two.clone(),
            self.arg_three.clone(),
            self.arg_four.clone(),
        );
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A named callable function taking three parameters and returning nothing.
#[derive(Debug)]
pub struct FiveToUnit<T, U, V, W, X>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
    X: Clone,
{
    name: String,
    work: fn(T, U, V, W, X) -> (),
    arg_one: T,
    arg_two: U,
    arg_three: V,
    arg_four: W,
    arg_five: X,
}

impl<T, U, V, W, X> FiveToUnit<T, U, V, W, X>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
    X: Clone,
{
    pub fn new(
        name: &str,
        work: fn(T, U, V, W, X) -> (),
        arg_one: T,
        arg_two: U,
        arg_three: V,
        arg_four: W,
        arg_five: X,
    ) -> Self {
        Self {
            name: name.into(),
            work,
            arg_one,
            arg_two,
            arg_three,
            arg_four,
            arg_five,
        }
    }
}

impl<T, U, V, W, X> Callable for FiveToUnit<T, U, V, W, X>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
    X: Clone,
{
    fn call(&self) -> Option<bool> {
        (self.work)(
            self.arg_one.clone(),
            self.arg_two.clone(),
            self.arg_three.clone(),
            self.arg_four.clone(),
            self.arg_five.clone(),
        );
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// A named callable function taking three parameters and returning nothing.
#[derive(Debug)]
pub struct SixToUnit<T, U, V, W, X, Y>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
    X: Clone,
    Y: Clone,
{
    name: String,
    work: fn(T, U, V, W, X, Y) -> (),
    arg_one: T,
    arg_two: U,
    arg_three: V,
    arg_four: W,
    arg_five: X,
    arg_six: Y,
}

impl<T, U, V, W, X, Y> SixToUnit<T, U, V, W, X, Y>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
    X: Clone,
    Y: Clone,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        work: fn(T, U, V, W, X, Y) -> (),
        arg_one: T,
        arg_two: U,
        arg_three: V,
        arg_four: W,
        arg_five: X,
        arg_six: Y,
    ) -> Self {
        Self {
            name: name.into(),
            work,
            arg_one,
            arg_two,
            arg_three,
            arg_four,
            arg_five,
            arg_six,
        }
    }
}

impl<T, U, V, W, X, Y> Callable for SixToUnit<T, U, V, W, X, Y>
where
    T: Clone,
    U: Clone,
    V: Clone,
    W: Clone,
    X: Clone,
    Y: Clone,
{
    fn call(&self) -> Option<bool> {
        (self.work)(
            self.arg_one.clone(),
            self.arg_two.clone(),
            self.arg_three.clone(),
            self.arg_four.clone(),
            self.arg_five.clone(),
            self.arg_six.clone(),
        );
        None
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "ffi")]
pub mod ffi {
    //! The CFFI feature requires different types, defined here

    use super::*;

    /// A named callable function taking no parameters and returning nothing.
    #[derive(Debug)]
    pub struct ExternUnitToUnit {
        name: String,
        work: extern "C" fn() -> (),
    }

    impl ExternUnitToUnit {
        pub fn new(name: &str, work: extern "C" fn() -> ()) -> Self {
            Self {
                name: name.into(),
                work,
            }
        }
    }

    impl Callable for ExternUnitToUnit {
        fn call(&self) -> Option<bool> {
            (self.work)();
            None
        }
        fn name(&self) -> &str {
            &self.name
        }
    }
    /*

    NOTE: This doesn't work - can't use generic interface across boundary, must be mangled

    /// A named callable function taking one parameter and returning nothing.
    #[derive(Debug)]
    pub struct ExternOneToUnit<T>
    where
        T: Clone,
    {
        name: String,
        work: extern "C" fn(T) -> (),
        arg: T,
    }

    impl<T> ExternOneToUnit<T>
    where
        T: Clone,
    {
        pub fn new(name: &str, work: extern "C" fn(T) -> (), arg: T) -> Self {
            Self {
                name: name.into(),
                work,
                arg,
            }
        }
    }

    impl<T> Callable for ExternOneToUnit<T>
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
    */
}
