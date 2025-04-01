use crate::package::Version;
use anyhow::Result;
use pubgrub::range::Range as SRange;
use pubgrub::version::Version as _;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Open(bool),
    Comma,
    Close(bool),
    Version(String),
}

struct Tokenizer<I> {
    buffer: String,
    last: Option<Token>,
    iter: I,
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
    pub fn new(iter: I) -> Self {
        Self {
            buffer: String::with_capacity(12),
            last: None,
            iter,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Tokenizer<I> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if let Some(last) = self.last.take() {
            return Some(last);
        }
        loop {
            let token = if let Some(c) = self.iter.next() {
                Some(match c {
                    '[' => Token::Open(true),
                    '(' => Token::Open(false),
                    ']' => Token::Close(true),
                    ')' => Token::Close(false),
                    ',' => Token::Comma,
                    _ => {
                        self.buffer.push(c);
                        continue;
                    }
                })
            } else {
                None
            };
            if self.buffer.is_empty() {
                return token;
            } else {
                let version = self.buffer.clone();
                self.buffer.clear();
                self.last = token;
                return Some(Token::Version(version));
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Bound {
    version: String,
    inclusive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Range {
    Exact(String),
    Greater(Bound),
    Lower(Bound),
    Between(Bound, Bound),
}

struct Parser<I> {
    iter: I,
    first: bool,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(iter: I) -> Self {
        Self { iter, first: true }
    }

    pub fn parse_range(&mut self) -> Option<Range> {
        match self.iter.next()? {
            Token::Version(version) => Some(Range::Greater(Bound {
                version,
                inclusive: true,
            })),
            Token::Open(inclusive) => {
                let lower_bound = match self.iter.next()? {
                    Token::Version(version) => match self.iter.next()? {
                        Token::Comma => Some(Bound { version, inclusive }),
                        Token::Close(true) if inclusive => return Some(Range::Exact(version)),
                        _ => return None,
                    },
                    Token::Comma => None,
                    _ => return None,
                };
                let upper_bound = match self.iter.next()? {
                    Token::Close(_) => None,
                    Token::Version(version) => match self.iter.next()? {
                        Token::Close(inclusive) => Some(Bound { version, inclusive }),
                        _ => return None,
                    },
                    _ => return None,
                };
                let range = match (lower_bound, upper_bound) {
                    (None, Some(bound)) => Range::Lower(bound),
                    (Some(bound), None) => Range::Greater(bound),
                    (Some(lower), Some(upper)) => Range::Between(lower, upper),
                    _ => return None,
                };
                Some(range)
            }
            _ => None,
        }
    }
}

impl<I: Iterator<Item = Token>> Iterator for Parser<I> {
    type Item = Range;

    fn next(&mut self) -> Option<Range> {
        if !self.first {
            assert_eq!(self.iter.next()?, Token::Comma);
        }
        self.first = false;
        self.parse_range()
    }
}

pub fn range(range_str: &str) -> Result<SRange<Version>> {
    let parser = Parser::new(Tokenizer::new(range_str.chars()));
    let mut range = SRange::none();
    for partial_range in parser {
        let srange = match partial_range {
            Range::Exact(version) => SRange::exact(Version::from_str(&version)?),
            Range::Lower(Bound { version, inclusive }) => {
                let mut version = Version::from_str(&version)?;
                if inclusive {
                    version = version.bump();
                }
                SRange::strictly_lower_than(version)
            }
            Range::Greater(Bound { version, .. }) => {
                let version = Version::from_str(&version)?;
                SRange::higher_than(version)
            }
            Range::Between(lower_bound, upper_bound) => {
                let lower_version = Version::from_str(&lower_bound.version)?;
                let mut upper_version = Version::from_str(&upper_bound.version)?;
                if upper_bound.inclusive {
                    upper_version = upper_version.bump();
                }
                SRange::between(lower_version, upper_version)
            }
        };
        range = range.union(&srange);
    }
    Ok(range)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RANGES: &[&str] = &[
        "(,1.0]",
        "1.0",
        "[1.0]",
        "[1.2,1.3]",
        "[1.0,2.0)",
        "[1.5,)",
        "(,1.0],[1.2,)",
        "(,1.1),(1.1,)",
    ];

    impl Bound {
        pub fn inclusive(version: &str) -> Self {
            Self {
                version: version.to_string(),
                inclusive: true,
            }
        }

        pub fn exclusive(version: &str) -> Self {
            Self {
                version: version.to_string(),
                inclusive: false,
            }
        }
    }

    #[test]
    fn test_range_1() {
        let r = range("(,1.0]").unwrap();
        assert!(r.contains(&"1.0".parse().unwrap()));
        assert!(!r.contains(&"1.0.1".parse().unwrap()));
        assert!(r.contains(&"0.5".parse().unwrap()));
    }

    #[test]
    fn test_range_2() {
        let r = range("1.0").unwrap();
        assert!(r.contains(&"1.0".parse().unwrap()));
        assert!(r.contains(&"1.0.1".parse().unwrap()));
        assert!(!r.contains(&"0.5".parse().unwrap()));
    }

    #[test]
    fn test_range_3() {
        let r = range("[1.0]").unwrap();
        assert!(r.contains(&"1.0".parse().unwrap()));
        assert!(!r.contains(&"1.0.1".parse().unwrap()));
        assert!(!r.contains(&"0.5".parse().unwrap()));
    }

    #[test]
    fn test_range_4() {
        let r = range("[1.2,1.3]").unwrap();
        assert!(!r.contains(&"1.0".parse().unwrap()));
        assert!(r.contains(&"1.2".parse().unwrap()));
        assert!(r.contains(&"1.2.99".parse().unwrap()));
        assert!(!r.contains(&"1.4".parse().unwrap()));
    }

    #[test]
    fn test_range_5() {
        let r = range("[1.2,2.0)").unwrap();
        assert!(!r.contains(&"1.0".parse().unwrap()));
        assert!(r.contains(&"1.2".parse().unwrap()));
        assert!(r.contains(&"1.99".parse().unwrap()));
        assert!(!r.contains(&"2.0".parse().unwrap()));
    }

    #[test]
    fn test_range_6() {
        let r = range("[1.5,)").unwrap();
        assert!(!r.contains(&"1.4".parse().unwrap()));
        assert!(r.contains(&"1.5".parse().unwrap()));
        assert!(r.contains(&"1.99".parse().unwrap()));
        assert!(r.contains(&"2.0".parse().unwrap()));
    }

    #[test]
    fn test_range_7() {
        let r = range("(,1.0],[1.2,)").unwrap();
        assert!(r.contains(&"0.99".parse().unwrap()));
        assert!(r.contains(&"1.0".parse().unwrap()));
        assert!(!r.contains(&"1.0.1".parse().unwrap()));
        assert!(!r.contains(&"1.1.99".parse().unwrap()));
        assert!(r.contains(&"1.2".parse().unwrap()));
        assert!(r.contains(&"2.0".parse().unwrap()));
    }

    #[test]
    #[ignore]
    fn test_range_8() {
        let r = range("(,1.1),(1.1,)").unwrap();
        assert!(r.contains(&"1.0".parse().unwrap()));
        assert!(!r.contains(&"1.1".parse().unwrap()));
        assert!(r.contains(&"1.1.1".parse().unwrap()));
    }

    #[test]
    fn parse() {
        let ranges: &[&[Range]] = &[
            &[Range::Lower(Bound::inclusive("1.0"))],
            &[Range::Greater(Bound::inclusive("1.0"))],
            &[Range::Exact("1.0".to_string())],
            &[Range::Between(
                Bound::inclusive("1.2"),
                Bound::inclusive("1.3"),
            )],
            &[Range::Between(
                Bound::inclusive("1.0"),
                Bound::exclusive("2.0"),
            )],
            &[Range::Greater(Bound::inclusive("1.5"))],
            &[
                Range::Lower(Bound::inclusive("1.0")),
                Range::Greater(Bound::inclusive("1.2")),
            ],
            &[
                Range::Lower(Bound::exclusive("1.1")),
                Range::Greater(Bound::exclusive("1.1")),
            ],
        ];
        for (range, ranges) in RANGES.iter().zip(ranges) {
            let ranges2 = Parser::new(Tokenizer::new(range.chars())).collect::<Vec<_>>();
            assert_eq!(ranges2.len(), ranges.len());
            for (a, b) in ranges2.iter().zip(ranges.iter()) {
                assert_eq!(a, b);
            }
        }
    }

    #[test]
    fn tokenize() {
        let tokens: &[&[Token]] = &[
            &[
                Token::Open(false),
                Token::Comma,
                Token::Version("1.0".to_string()),
                Token::Close(true),
            ],
            &[Token::Version("1.0".to_string())],
            &[
                Token::Open(true),
                Token::Version("1.0".to_string()),
                Token::Close(true),
            ],
            &[
                Token::Open(true),
                Token::Version("1.2".to_string()),
                Token::Comma,
                Token::Version("1.3".to_string()),
                Token::Close(true),
            ],
            &[
                Token::Open(true),
                Token::Version("1.0".to_string()),
                Token::Comma,
                Token::Version("2.0".to_string()),
                Token::Close(false),
            ],
            &[
                Token::Open(true),
                Token::Version("1.5".to_string()),
                Token::Comma,
                Token::Close(false),
            ],
            &[
                Token::Open(false),
                Token::Comma,
                Token::Version("1.0".to_string()),
                Token::Close(true),
                Token::Comma,
                Token::Open(true),
                Token::Version("1.2".to_string()),
                Token::Comma,
                Token::Close(false),
            ],
            &[
                Token::Open(false),
                Token::Comma,
                Token::Version("1.1".to_string()),
                Token::Close(false),
                Token::Comma,
                Token::Open(false),
                Token::Version("1.1".to_string()),
                Token::Comma,
                Token::Close(false),
            ],
        ];
        for (range, tokens) in RANGES.iter().zip(tokens) {
            let tokens2 = Tokenizer::new(range.chars()).collect::<Vec<_>>();
            assert_eq!(tokens2.len(), tokens.len());
            for (a, b) in tokens2.iter().zip(tokens.iter()) {
                assert_eq!(a, b);
            }
        }
    }
}
