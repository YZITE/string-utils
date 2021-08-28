use core::iter::Peekable;

pub struct AdvanPeekIter<I>
where
    I: Iterator,
{
    inner: Peekable<I>,
}

pub struct TakeWhile<'a, I, P>
where
    I: Iterator,
{
    iter: &'a mut AdvanPeekIter<I>,
    flag: bool,
    predicate: P,
}

impl<I> AdvanPeekIter<I>
where
    I: Iterator,
{
    #[inline]
    pub fn new(inner: I) -> Self {
        Self { inner: inner.peekable() }
    }

    /// like `take_while`, but doesn't consume the first non-matching line by utilizing Peekable
    #[inline]
    pub fn intell_take_while<P>(&mut self, predicate: P) -> TakeWhile<'_, I, P>
    where
        // aide type inference
        P: FnMut(&I::Item) -> bool,
    {
        TakeWhile {
            iter: self,
            flag: false,
            predicate,
        }
    }

    #[inline]
    pub fn count(self) -> usize {
        self.inner.count()
    }
}

impl<I> core::ops::Deref for AdvanPeekIter<I>
where
    I: Iterator,
{
    type Target = Peekable<I>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I> core::ops::DerefMut for AdvanPeekIter<I>
where
    I: Iterator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I, P> Iterator for TakeWhile<'_, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.flag {
            None
        } else if let Some(x) = self.iter.inner.next_if(&mut self.predicate) {
            Some(x)
        } else {
            self.flag = true;
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let upp = if self.flag {
            Some(0)
        } else {
            self.iter.inner.size_hint().1
        };
        // can't know a lower bound, due to the predicate
        (0, upp)
    }

    // can't implement try_fold, it's unstable bc of `Try`...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut it0 = AdvanPeekIter::new((0..10).into_iter());
        let a = it0.intell_take_while(|&i| i < 5).count();
        let b = it0.count();
        assert_eq!(a, 5);
        assert_eq!(b, 5);
    }
}
