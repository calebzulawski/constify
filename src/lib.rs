#![no_std]
//! `const`-ify runtime values.
//!
//! The [`constify`] macro checks runtime values against a list of constants, and evaluates an
//! expression using those constants.
//!
//! The [`try_constify`] macro does the same, but permits errors when the input is out-of-range.

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_constify {
    // Pull off the first const variable
    {
        @$mode:ident
        [{ const $const_var:ident: $const_ty:ty = $match_val:expr => $($const_expr:expr),+; } $($rest:tt)*]
        $block:block
    } => {
        $crate::__impl_constify! {
            @$mode
            const $const_var: $const_ty = $match_val => $($const_expr),*;
            [$($rest)*]
            $block
        }
    };

    // Implement the "normal" tree, requires all variants to be covered
    {
        @normal
        const $const_var:ident: $const_ty:ty = $match_val:expr => $($const_expr:expr),+;
        $rest:tt
        $block:block
    } => {
        match $match_val {
            $(
            $const_expr => {
                const $const_var: $const_ty = $const_expr;
                $crate::__impl_constify! { @normal $rest $block }
            }
            )*

        }
    };

    // Terminate the "normal" tree
    {
        @normal [] $block:block
    } => {
        $block
    };

    // Implement the "erroring" tree, which returns an error on missing variants
    {
        @error
        const $const_var:ident: $const_ty:ty = $match_val:expr => $($const_expr:expr),+;
        $rest:tt
        $block:block
    } => {
        match $match_val {
            $(
            $const_expr => {
                const $const_var: $const_ty = $const_expr;
                $crate::__impl_constify! { @error $rest $block }
            }
            )*
            #[allow(unreachable_patterns)]
            _ => ::core::result::Result::Err(concat!("unexpected value for `", stringify!($const_var), "`"))
        }
    };

    // Terminate the "erroring" tree
    {
        @error [] $block:block
    } => {
        Ok($block)
    };
}

/// Convert runtime values to `const`s.
///
/// This macro compares runtime expressions to the provided constants and evaluates the given
/// expression with the matching constants.
/// All cases must be covered; to match only a subset of values use [`try_constify`].
///
/// The expressions are evaluated in the order they are provided.
///
/// An example:
///
/// ```
/// fn sum_impl<const A: bool, const B: bool>(a: u32, b: u32) -> u32 {
///     let mut sum = 0;
///     if A {
///         sum += a;
///     }
///     if B {
///         sum += b;
///     }
///     sum
/// }
///
/// fn sum(a: u32, b: u32, add_a: bool, add_b: bool) -> u32 {
///     constify::constify! (
///         const A: bool = add_a => true, false;
///         const B: bool = add_b => true, false;
///
///         sum_impl::<A, B>(a, b)
///     )
/// }
///
/// assert_eq!(sum(3, 4, false, false), 0);
/// assert_eq!(sum(3, 4, true, false), 3);
/// assert_eq!(sum(3, 4, false, true), 4);
/// assert_eq!(sum(3, 4, true, true), 7);
/// ```
#[macro_export]
macro_rules! constify {
    {
        $(
        const $const_var:ident: $const_ty:ty = $match_val:expr => $($const_expr:expr),+;
        )*

        $return:expr
    } => {
        $crate::__impl_constify! {
            @normal
            [$({const $const_var: $const_ty = $match_val => $($const_expr),*;})*]

            { $return }
        }
    }
}

/// Fallibly convert runtime values to `const`s.
///
/// This macro is identical to [`constify`], except it returns a `Result<_, &'static str>`.
/// If all of the runtime expressions evaluate to one of the provided constants, `Ok(_)` is
/// returned.
/// If any of the runtime values don't match any of the provided constants, `Err(msg)` is returned,
/// and `msg` contains a description of which constant failed the match.
///
/// An example:
/// ```should_panic
/// fn add_impl<const A: u32, const B: u32>() -> u32 {
///     A + B
/// }
///
/// fn add(a: u32, b: u32) -> u32 {
///     constify::try_constify! (
///         const A: u32 = a => 1, 2;
///         const B: u32 = b => 3, 4;
///
///         add_impl::<A, B>()
///     ).unwrap()
/// }
///
/// assert_eq!(add(1, 3), 4); // This is OK
/// assert_eq!(add(3, 3), 6); // This panics, since `a` is out of range!
/// ```
#[macro_export]
macro_rules! try_constify {
    {
        $(
        const $const_var:ident: $const_ty:ty = $match_val:expr => $($const_expr:expr),+;
        )*

        $return:expr
    } => {
        $crate::__impl_constify! {
            @error
            [$({const $const_var: $const_ty = $match_val => $($const_expr),*;})*]

            { $return }
        }
    }
}
