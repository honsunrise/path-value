use parser::PathParser;
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::error::*;

mod parser {
    use pest::Parser;

    use crate::error::*;

    use super::{Path, PathNode};

    #[derive(Parser)]
    #[grammar = "path/path.pest"]
    pub struct PathParser;

    impl PathParser {
        pub fn parse_to_path(s: &str) -> Result<Path> {
            let mut result: Vec<PathNode> = Vec::new();
            let path = PathParser::parse(Rule::path, s)
                .map_err(|e| Error::path_parse(e, s))?
                .next()
                .unwrap();
            for sub_path in path.into_inner() {
                if let Some(ident) = sub_path.into_inner().next() {
                    match ident.as_rule() {
                        Rule::path_ident => {
                            result.push(PathNode::Identifier(ident.as_str().to_string()))
                        }
                        Rule::path_index_ident => {
                            let mut path_index_ident_inner = ident.into_inner();
                            result.push(PathNode::Identifier(
                                path_index_ident_inner.next().unwrap().as_str().to_string(),
                            ));
                            result.push(PathNode::Index(
                                path_index_ident_inner
                                    .next()
                                    .unwrap()
                                    .as_str()
                                    .parse()
                                    .unwrap(),
                            ));
                        }
                        _ => unreachable!(),
                    };
                }
            }

            Ok(Path(result))
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum PathNode {
    Identifier(String),
    Index(isize),
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Path(Vec<PathNode>);

impl Default for Path {
    fn default() -> Self {
        Path(Vec::new())
    }
}

impl Deref for Path {
    type Target = Vec<PathNode>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for Path {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        PathParser::parse_to_path(s)
    }
}

impl TryFrom<String> for Path {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        PathParser::parse_to_path(&value)
    }
}

impl<'a> TryFrom<&'a str> for Path {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self> {
        PathParser::parse_to_path(value)
    }
}

#[allow(unused_imports)]
mod tests {
    use super::{Path, PathNode};

    #[test]
    fn test_empty() {
        let parsed = "".parse::<Path>();
        assert!(matches!(parsed, Err(err)));
    }

    #[test]
    fn test_root() {
        let except_path = Path(vec![]);
        let parsed = "/".parse::<Path>();
        assert!(matches!(parsed, Ok(path) if path == except_path ));
    }

    #[test]
    fn test_1_level() {
        let except_path = Path(vec![PathNode::Identifier("a".to_string())]);
        let parsed = "/a".parse::<Path>();
        assert!(matches!(parsed, Ok(path) if path == except_path ));
    }

    #[test]
    fn test_n_level() {
        let except_path = Path(vec![
            PathNode::Identifier("a".to_string()),
            PathNode::Identifier("b".to_string()),
            PathNode::Identifier("c".to_string()),
        ]);
        let parsed = "/a/b/c".parse::<Path>();
        assert!(matches!(parsed, Ok(path) if path == except_path ));
    }

    #[test]
    fn test_1_level_index() {
        let except_path = Path(vec![
            PathNode::Identifier("a".to_string()),
            PathNode::Index(0),
        ]);
        let parsed = "/a[0]".parse::<Path>();
        assert!(matches!(parsed, Ok(path) if path == except_path ));
    }

    #[test]
    fn test_n_level_index() {
        let except_path = Path(vec![
            PathNode::Identifier("a".to_string()),
            PathNode::Index(0),
            PathNode::Identifier("b".to_string()),
            PathNode::Identifier("c".to_string()),
            PathNode::Index(1),
        ]);
        let parsed = "/a[0]/b/c[1]".parse::<Path>();
        assert!(matches!(parsed, Ok(path) if path == except_path ));
    }
}
