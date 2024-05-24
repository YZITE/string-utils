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

    /// # Panics
    /// This panics if the string does not start with a character in XID
    #[cfg(feature = "consume-ident")]
    pub fn consume_ident(&mut self) -> alloc::string::String {
        use alloc::string::ToString;
        use unicode_normalization::UnicodeNormalization;
        let s = self
            .consume_select(unicode_ident::is_xid_continue)
            .nfkc()
            .to_string();
        assert!(!s.is_empty());
        s
    }

    #[cfg(feature = "consume-ident")]
    pub fn try_consume_ident(&mut self) -> Option<alloc::string::String> {
        if self.inp.chars().next().map(unicode_ident::is_xid_start) == Some(true) {
            Some(self.consume_ident())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "consume-ident")]
    fn test_consume_ident() {
        use alloc::string::ToString;
        let mut slb = StrLexerBase {
            inp: "hello ",
            offset: 0,
        };
        assert_eq!(slb.try_consume_ident(), Some("hello".to_string()));
        assert_eq!(slb.offset, 5);

        let mut slb = StrLexerBase {
            inp: "ö ",
            offset: 0,
        };
        assert_eq!(slb.try_consume_ident(), Some("ö".to_string()));
        assert_eq!(slb.offset, 2);

        let mut slb = StrLexerBase {
            inp: ".ö ",
            offset: 0,
        };
        assert_eq!(slb.try_consume_ident(), None);
        assert_eq!(slb.offset, 0);
    }
}
