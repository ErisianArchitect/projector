use std::{cell::RefCell, collections::HashSet};


#[inline(always)]
pub const fn assign_result<'a, T, R, F>(out: &'a mut R, f: F) -> impl 'a + FnOnce(T)
where
    F: 'a + FnOnce(T) -> R
{
    move |arg| {
        *out = f(arg);
    }
}

#[inline(always)]
pub const fn assign_left_result<'a, T, L, R, F>(out: &'a mut L, f: F) -> impl 'a + FnOnce(T) -> R
where
    F: 'a + FnOnce(T) -> (L, R)
{
    move |arg| {
        let (left, right) = f(arg);
        *out = left;
        right
    }
}

#[inline]
pub fn pass<T>(value: T) -> T {
    value
}

#[inline]
pub fn eval<R, F: FnOnce() -> R>(f: F) -> R {
    f()
}

#[inline]
pub fn catch<T, E, F: FnOnce() -> Result<T, E>>(f: F) -> Result<T, E> {
    f()
}

pub fn has_duplicate<T: PartialEq<T>>(items: &[T]) -> bool {
    if items.len() <= 1 {
        return false;
    }
    for i in 0..(items.len() - 1) {
        for j in (i + 1)..items.len() {
            if items[i] == items[j] {
                return true;
            }
        }
    }
    false
}

// /// This function only works with ascii strings.
// pub fn match_rank<S0: AsRef<[u8]>, S1: AsRef<[u8]>>(left: S0, right: S1) -> u32 {
//     fn inner(left: &[u8], right: &[u8]) -> u32 {

//     }
//     inner(left.as_ref(), right.as_ref())
// }