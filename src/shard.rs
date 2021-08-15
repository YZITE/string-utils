use crate::Cow;
use alloc::string::{String, ToString};

pub enum Shard<'a> {
    Borrowed { whole: &'a str, slen: usize },
    Owned(String),
}

impl<'a> Shard<'a> {
    #[inline]
    pub fn new(whole: &'a str) -> Self {
        Self::Borrowed { whole, slen: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Borrowed { ref slen, .. } => *slen,
            Self::Owned(ref owned) => owned.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn to_mut(&mut self) -> &mut String {
        if let Self::Borrowed { whole, slen } = *self {
            *self = Self::Owned(whole[..slen].to_string());
        }
        match *self {
            Self::Borrowed { .. } => unreachable!(),
            Self::Owned(ref mut x) => x,
        }
    }

    #[inline]
    pub fn skip(&mut self, len: usize) {
        if let Self::Borrowed { whole, slen: 0 } = self {
            *whole = &whole[len..];
        }
    }

    // promotes self to owned
    #[inline]
    pub fn push_owned(&mut self, ch: char) {
        self.to_mut().push(ch);
    }

    pub fn push(&mut self, ch: char) {
        match self {
            Self::Borrowed { whole, slen } => {
                let slen = *slen;
                let new_len = slen + ch.len_utf8();
                *self = if !whole[slen..].starts_with(ch) {
                    // promote to owned
                    let mut owned = whole[..slen].to_string();
                    owned.push(ch);
                    Self::Owned(owned)
                } else {
                    // remain borrowed
                    Self::Borrowed {
                        whole,
                        slen: new_len,
                    }
                };
            }
            Self::Owned(ref mut x) => x.push(ch),
        }
    }

    pub fn finish(self) -> Cow<'a, str> {
        if self.is_empty() {
            Cow::Borrowed("")
        } else {
            match self {
                Self::Borrowed { whole, slen } => Cow::Borrowed(&whole[..slen]),
                Self::Owned(x) => Cow::Owned(x),
            }
        }
    }

    pub(crate) fn finish_cvg(self) -> Option<Cow<'a, str>> {
        if self.is_empty() {
            None
        } else {
            Some(self.finish())
        }
    }
}
