#![no_std]

extern crate alloc;
pub use alloc::borrow::Cow;

mod advanpeekiter;
pub use advanpeekiter::{AdvanPeekIter, TakeWhile as APITakeWhile};

mod shard;
pub use shard::Shard;

mod shwsplit;
pub use shwsplit::{ShellwordSplitter, SyntaxError as SimpleSyntaxError};

fn get_offset_of<T>(whole_buffer: &T, part: &T) -> usize
where
    T: AsRef<[u8]> + ?Sized,
{
    // NOTE: originally I wanted to use offset_from() here once it's stable,
    // but according to https://github.com/rust-lang/rust/issues/41079#issuecomment-657163887
    // this would be UB in cases where the code below isn't.
    part.as_ref().as_ptr() as usize - whole_buffer.as_ref().as_ptr() as usize
}

/// Assuming that `post_part` is a true (in regards to memory allocations)
/// subslice of `whole_buffer_start`, returns everything which comes before `post_part`.
pub fn slice_between<'a>(whole_buffer_start: &'a [u8], post_part: &'a [u8]) -> &'a [u8] {
    debug_assert!(post_part.len() < whole_buffer_start.len());
    &whole_buffer_start[..get_offset_of(whole_buffer_start, post_part)]
}

/// Splits a slice at the first point after which `f` returns false.
/// Usually used to segment input according to character categories.
///
/// e.g. 1. part while `f(x) == true`, then 2. part
pub fn split_at_while(x: &[u8], f: impl FnMut(&u8) -> bool) -> (&[u8], &[u8]) {
    x.split_at(x.iter().copied().take_while(f).count())
}
