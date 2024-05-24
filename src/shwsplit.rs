use crate::{Cow, Shard};

#[derive(Clone, Copy, Debug)]
pub struct SyntaxError;

pub struct ShellwordSplitter<'a> {
    input: &'a str,
}

impl<'a> ShellwordSplitter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    fn skip_whitespace(&mut self) {
        let mut it = self.input.char_indices();
        self.input = loop {
            break match it.next() {
                None => "",
                Some((pos, x)) if !x.is_whitespace() => &self.input[pos..],
                _ => continue,
            };
        };
    }
}

fn ch_is_quote(ch: char) -> bool {
    matches!(ch, '"' | '\'')
}

impl<'a> Iterator for ShellwordSplitter<'a> {
    type Item = Result<Cow<'a, str>, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let mut it = self.input.char_indices();
        let mut quotec = None;
        let mut ret = Shard::<'a>::new(self.input);
        while let Some((cpos, cx)) = it.next() {
            if cx == '\\' {
                // escape works the same, no matter if inside or outside of quotes
                let x = match it.next() {
                    Some(i) => i.1,
                    None => {
                        self.input = "";
                        return Some(Err(SyntaxError));
                    }
                };
                ret.push_owned(match x {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    _ if quotec.is_some() && x.is_whitespace() => continue,
                    _ => x,
                });
                continue;
            }
            if quotec.is_none() {
                if ch_is_quote(cx) {
                    // start of quotation
                    quotec = Some(cx);
                    // allow the algo to reuse simple, quoted args
                    ret.skip(1);
                    continue;
                } else if cx.is_whitespace() {
                    // argument separator, this will never happen on the first iteration
                    self.input = &self.input[cpos..];
                    return ret.finish_cvg().map(Ok);
                }
            } else if Some(cx) == quotec {
                // end of quotation
                quotec = None;
                match it.next() {
                    Some((npos, nx)) if nx.is_whitespace() => {
                        // simple case: the ending quote is followed by an separator
                        // we can thus skip the whitespace and return our item
                        self.input = &self.input[npos..];
                        return ret.finish_cvg().map(Ok);
                    }
                    Some((_, nx)) if ch_is_quote(nx) => {
                        // medium case: the ending quote if directly followed by another quote
                        // thus, remain in quote mode
                        quotec = Some(nx);
                    }
                    Some((_, nx)) => {
                        // complex case: the ending quote is followed by more data which
                        // belongs to the same argument
                        ret.push_owned(nx);
                    }
                    None => {
                        // simple case: the ending quote is followed by EOF
                        self.input = "";
                        return ret.finish_cvg().map(Ok);
                    }
                }
                continue;
            }
            ret.push(cx);
        }
        self.input = "";
        if quotec.is_some() {
            return Some(Err(SyntaxError));
        }
        ret.finish_cvg().map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec::Vec};
    use proptest::prelude::*;

    /// split_shellwords tests were taken from
    /// https://docs.rs/shellwords/1.1.0/src/shellwords/lib.rs.html
    /// License: MIT
    fn split(x: &str) -> Result<Vec<String>, super::SyntaxError> {
        super::ShellwordSplitter::new(x)
            .map(|i| i.map(super::Cow::into_owned))
            .collect()
    }

    #[test]
    fn nothing_special() {
        assert_eq!(split("a b c d").unwrap(), ["a", "b", "c", "d"]);
    }

    #[test]
    fn quoted_strings() {
        assert_eq!(split("a \"b b\" a").unwrap(), ["a", "b b", "a"]);
    }

    #[test]
    fn escaped_double_quotes() {
        assert_eq!(split("a \"\\\"b\\\" c\" d").unwrap(), ["a", "\"b\" c", "d"]);
    }

    #[test]
    fn escaped_single_quotes() {
        assert_eq!(split("a \"'b' c\" d").unwrap(), ["a", "'b' c", "d"]);
    }

    #[test]
    fn escaped_spaces() {
        assert_eq!(split("a b\\ c d").unwrap(), ["a", "b c", "d"]);
    }

    #[test]
    fn start_with_qspaces() {
        assert_eq!(split("\"  \" b c").unwrap(), ["  ", "b", "c"]);
    }

    #[test]
    fn bad_double_quotes() {
        split("a \"b c d e").unwrap_err();
    }

    #[test]
    fn bad_single_quotes() {
        split("a 'b c d e").unwrap_err();
    }

    #[test]
    fn bad_quotes() {
        split("one '\"\"\"").unwrap_err();
    }

    #[test]
    fn trailing_whitespace() {
        assert_eq!(split("a b c d ").unwrap(), ["a", "b", "c", "d"]);
    }

    proptest! {
        #[test]
        fn doesnt_crash(s in "\\PC*") {
            let _: Vec<_> = super::ShellwordSplitter::new(&s).collect();
        }
    }
}
