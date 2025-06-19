
#[inline(always)]
pub const fn assign_result<'a, T, R, F>(out: &'a mut R, f: F) -> impl FnOnce(T) -> () + 'a
where
    F: 'a + FnOnce(T) -> R
{
    move |arg| {
        *out = f(arg);
    }
}

#[inline(always)]
pub const fn assign_left_result<'a, T, L, R, F>(out: &'a mut L, f: F) -> impl FnOnce(T) -> R + 'a
where
    F: 'a + FnOnce(T) -> (L, R)
{
    move |arg| {
        let (left, right) = f(arg);
        *out = left;
        right
    }
}

// /// This function only works with ascii strings.
// pub fn match_rank<S0: AsRef<[u8]>, S1: AsRef<[u8]>>(left: S0, right: S1) -> u32 {
//     fn inner(left: &[u8], right: &[u8]) -> u32 {

//     }
//     inner(left.as_ref(), right.as_ref())
// }