//! The `inner!` macro makes descending into an enum variant
//! more ergonomic.
//!
//! The `some!` and `ok!` macros turn your enum into an `Option` and `Result`, respectively.
//!
//! # Helpful unwrap
//! The simplest case for `inner!` is almost like unwrap:
//!
//! ```
//! # use try_utils::*;
//! # fn main() {
//! let x = Some(1);
//! let y: Result<_, ()> = Ok(2);
//! assert_eq!(inner!(x), 1);
//! assert_eq!(inner!(y), 2);
//! # }
//! ```
//!
//! ...but if you instead use it on a `None` or `Err` value:
//!
//! ```ignore
//! let z = None;
//! let y = inner!(z);
//! ```
//!
//! ...it will panic, with an error message that points you to a more
//! helpful location than some line number inside libcore:
//!
//! ```ignore
//! thread "test" panicked at "Unexpected value found inside "z"", src/lib.rs:23
//! ```
//!
//! # Error handling
//! If panic isn't an option - and it usually isn't - just add an `else` clause:
//!
//! ```
//! # use try_utils::*;
//! # fn main() {
//! let x: Result<String, i32> = Err(7);
//! let y = inner!(x, else return);
//! // Since x is an Err, we'll never get here.
//! println!("The string length is: {}", y.len());
//! # }
//! ```
//!
//! You can use the else clause to compute a default value, or use flow control
//! (e g `break`, `continue`, or `return`).
//!
//! Want access to what's inside the `Err` value in your `else` clause?
//! No problem, just add a `|variable|` after `else`, like this:
//!
//! ```
//! # use try_utils::*;
//! # fn main() {
//! let x: Result<String, i32> = Err(7);
//! let y = inner!(x, else |e| {
//!     assert_eq!(e, 7);
//!     (e + 2).to_string()
//! });
//! assert_eq!(&y, "9");
//! # }
//! ```
//!
//! Note: This does not turn your else clause into a closure, so you can still use
//! (e g) `return` the same way as before.
//!
//! # It works with your enums too
//! It does not work only with `Option` and `Result`. Just add an `if` clause:
//!
//! ```
//! # use try_utils::*;
//! # fn main() {
//! enum Fruit {
//!     Apple(i32),
//!     Orange(i16),
//! }
//!
//! let z = Fruit::Apple(15);
//! let y = inner!(z, if Fruit::Apple, else {
//!     println!("I wanted an apple and I didn't get one!");
//!     0
//! });
//! assert_eq!(y, 15);
//! # }
//! ```
//!
//! You can skip the `else` clause to panic in case the enum is not
//! the expected variant.
//!
//! Note that in this case, the entire item (instead of the contents inside
//! `Err`) is passed on to the `else` clause:
//!
//! ```
//! # use try_utils::*;
//! # fn main() {
//! #[derive(Eq, PartialEq, Debug)]
//! enum Fruit {
//!     Apple(i32),
//!     Orange(i16),
//! }
//!
//! let z = Fruit::Orange(15);
//! inner!(z, if Fruit::Apple, else |e| {
//!     assert_eq!(e, Fruit::Orange(15));
//!     return;
//! });
//! # }
//! ```
//!
//! Another option is to implement this crate's `IntoResult` trait for
//! your enum. Then you don't have to write an `if` clause to tell what
//! enum variant you want to descend into, and you can choose more than
//! one enum variant to be `Ok`:
//!
//! ```ignore
//! enum Fruit {
//!     Apple(i32),
//!     Orange(i16),
//!     Rotten,
//! }
//!
//! impl IntoResult<i32, ()> for Fruit {
//!     fn into_result(self) -> Result<i32, ()> {
//!         match self {
//!             Fruit::Apple(i) => Ok(i),
//!             Fruit::Orange(i) => Ok(i as i32),
//!             Fruit::Rotten => Err(()),
//!         }
//!     }
//! }
//!
//! assert_eq!(9, inner!(Fruit::Apple(9)));
//! ```
//!
//! # License
//! Apache2.0/MIT

/// Converts a value into a Result.
/// You can implement this for your own types if you want
/// to use the `inner!` macro in more ergonomic ways.
pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E> IntoResult<T, E> for Result<T, E> {
    #[inline]
    fn into_result(self) -> Result<T, E> {
        self
    }
}

impl<T> IntoResult<T, ()> for Option<T> {
    #[inline]
    fn into_result(self) -> Result<T, ()> {
        self.ok_or(())
    }
}

/// The `try!` macro - see module level documentation for details.
#[macro_export]
macro_rules! inner {
    ($x:expr, if $i:path, else |$e:ident| $b:expr) => {{
        match $x {
            $i(q) => q,
            $e @ _ => $b,
        }
    }};

    ($x:expr, if $i:path, else $b:expr) => {{
        match $x {
            $i(q) => q,
            _ => $b,
        }
    }};

    ($x:expr, else |$e:ident| $b:expr) => {{
        use $crate::IntoResult;
        match $x.into_result() {
            Ok(q) => q,
            Err($e) => $b,
        }
    }};

    ($x:expr, else $b:expr) => {{
        use $crate::IntoResult;
        match $x.into_result() {
            Ok(q) => q,
            _ => $b,
        }
    }};

    ($x:expr, if $i:path) => {{
        match $x {
            $i(q) => q,
            _ => panic!("Unexpected value found inside '{}'", stringify!($x)),
        }
    }};

    ($x:expr) => {{
        use $crate::IntoResult;
        match $x.into_result() {
            Ok(q) => q,
            _ => panic!("Unexpected value found inside '{}'", stringify!($x)),
        }
    }};
}

/// Converts your enum to an Option.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(some!(Fruit::Apple(15), if Fruit::Apple), Some(15));
/// assert_eq!(some!(Fruit::Orange(5), if Fruit::Apple), None);
/// ```
#[macro_export]
macro_rules! some {
    ($x:expr, if $i:path, else |$e:ident| $b:expr) => {{
        match $x {
            $i(q) => Some(q),
            $e @ _ => $b,
        }
    }};

    ($x:expr, if $i:path, else $b:expr) => {{
        match $x {
            $i(q) => Some(q),
            _ => $b,
        }
    }};

    ($x:expr, if $i:path) => {{
        match $x {
            $i(q) => Some(q),
            _ => None,
        }
    }};
}

/// Converts your enum to an Result.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(ok!(Fruit::Apple(15), if Fruit::Apple), Ok(15));
/// assert_eq!(ok!(Fruit::Orange(5), if Fruit::Apple), Err(Fruit::Orange(5)));
///
/// assert_eq!(ok!(Fruit::Orange(5), if Fruit::Apple, or {75}), Err(75));
/// assert_eq!(ok!(Fruit::Orange(5), if Fruit::Apple, else {Err(75)}), Err(75));
/// ```
#[macro_export]
macro_rules! ok {
    ($x:expr, if $i:path, else |$e:ident| $b:expr) => {{
        match $x {
            $i(q) => Ok(q),
            $e @ _ => $b,
        }
    }};

    ($x:expr, if $i:path, else $b:expr) => {{
        match $x {
            $i(q) => Ok(q),
            _ => $b,
        }
    }};

    ($x:expr, if $i:path, or |$e:ident| $b:expr) => {{
        match $x {
            $i(q) => Ok(q),
            $e @ _ => Err($b),
        }
    }};

    ($x:expr, if $i:path, or $b:expr) => {{
        match $x {
            $i(q) => Ok(q),
            _ => Err($b),
        }
    }};

    ($x:expr, if $i:path) => {{
        match $x {
            $i(q) => Ok(q),
            n @ _ => Err(n),
        }
    }};
}

#[test]
fn simple_opt() {
    assert_eq!(inner!(Some(7)), 7);
}

#[test]
#[should_panic]
fn simple_opt_fail() {
    let z: Option<i32> = None;
    inner!(z);
}

#[test]
fn else_clause() {
    let x: Result<String, i32> = Err(7);
    let _ = inner!(x, else return);
    panic!();
}

#[test]
fn else_clause_2() {
    let x: Result<String, i32> = Err(7);
    let y = inner!(x, else |e| {
        assert_eq!(e, 7);
        (e + 2).to_string()
    });
    assert_eq!(&y, "9");
}

#[test]
fn apple() {
    enum Fruit {
        Apple(i32),
        _Orange(i16),
    }
    let z = Fruit::Apple(15);
    assert_eq!(15, inner!(z, if Fruit::Apple));
}

#[test]
fn if_else() {
    enum Fruit {
        Apple(i32),
        _Orange(i16),
    }
    let z = Fruit::Apple(15);
    assert_eq!(15, inner!(z, if Fruit::Apple, else panic!("Not an apple")));
}

#[test]
fn own_enum() {
    #[derive(Debug, PartialEq, Eq)]
    enum Fruit {
        Apple(i32),
        Orange(i16),
    }

    impl IntoResult<i32, i16> for Fruit {
        fn into_result(self) -> Result<i32, i16> {
            match self {
                Fruit::Apple(i) => Ok(i),
                Fruit::Orange(i) => Err(i),
            }
        }
    }
    let z = Fruit::Orange(15);
    assert_eq!(7, inner!(z, else |e| (e - 8) as i32));

    let z = Fruit::Apple(15);
    assert_eq!(
        9,
        inner!(z, if Fruit::Orange, else |e| {
            assert_eq!(e, Fruit::Apple(15));
            9
        })
    );
}

#[test]
fn some() {
    #[derive(Debug, PartialEq, Eq)]
    enum Fruit {
        Apple(i32),
        Orange(i16),
    }

    assert_eq!(some!(Fruit::Apple(15), if Fruit::Apple), Some(15));
    assert_eq!(some!(Fruit::Orange(15), if Fruit::Apple), None);
    assert_eq!(
        some!(Fruit::Orange(15), if Fruit::Apple, else |e| {
            assert_eq!(e, Fruit::Orange(15));
            Some(30)
        }),
        Some(30)
    );
}

#[test]
fn ok() {
    #[derive(Debug, PartialEq, Eq)]
    enum Fruit {
        Apple(i32),
        Orange(i16),
    }

    assert_eq!(ok!(Fruit::Apple(15), if Fruit::Apple), Ok(15));

    assert_eq!(
        ok!(Fruit::Orange(15), if Fruit::Apple),
        Err(Fruit::Orange(15))
    );
    assert_eq!(
        ok!(Fruit::Orange(15), if Fruit::Apple, else |e| {
            assert_eq!(e, Fruit::Orange(15));
            Err(3)
        }),
        Err(3)
    );

    assert_eq!(ok!(Fruit::Apple(15), if Fruit::Orange, or 67), Err(67));
    assert_eq!(ok!(Fruit::Apple(15), if Fruit::Apple, or 67), Ok(15));
}
