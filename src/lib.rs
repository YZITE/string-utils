#![no_std]

extern crate alloc;
pub use alloc::borrow::Cow;

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

/// Counts the number of bytes that got accepted by `f`.
pub fn count_str_bytes<F>(inp: &str, mut f: F) -> usize
where
    F: FnMut(char) -> bool,
{
    inp.chars()
        .take_while(move |&i| f(i))
        .map(|i| i.len_utf8())
        .sum()
}

pub trait SplitAtWhile {
    type Item;

    /// Splits a slice at the first point after which `f` returns false.
    /// Usually used to segment input according to character categories.
    ///
    /// e.g. 1. part while `f(x) == true`, then 2. part
    fn split_at_while<F>(&self, f: F) -> (&Self, &Self)
    where
        F: FnMut(&Self::Item) -> bool;
}

impl<T> SplitAtWhile for [T] {
    type Item = T;

    fn split_at_while<F>(&self, mut f: F) -> (&Self, &Self)
    where
        F: FnMut(&T) -> bool,
    {
        self.split_at(self.iter().take_while(move |&i| f(i)).count())
    }
}

impl SplitAtWhile for str {
    type Item = char;

    fn split_at_while<F>(&self, f: F) -> (&Self, &Self)
    where
        F: FnMut(&char) -> bool,
    {
        self.split_at(self.chars().take_while(f).map(|i| i.len_utf8()).sum())
    }
}

#[derive(Copy, Clone)]
pub struct StrLexerBase<'a> {
    pub inp: &'a str,
    pub offset: usize,
}

impl<'a> StrLexerBase<'a> {
    #[inline]
    pub fn consume(&mut self, l: usize) -> &'a str {
        let (a, b) = self.inp.split_at(l);
        self.inp = b;
        self.offset += l;
        a
    }

    pub fn consume_select<F>(&mut self, f: F) -> &'a str
    where
        F: FnMut(char) -> bool,
    {
        self.consume(count_str_bytes(self.inp, f))
    }
}
