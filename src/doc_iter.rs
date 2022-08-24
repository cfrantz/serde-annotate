use crate::document::Document;

impl Document {
    /// Returns an iterator over all all document nodes including
    /// comments and fragments.
    ///
    /// When encountering a container node (mapping, sequence or fragment),
    /// the container node is yielded first, then all of its children.
    pub fn iter(&self) -> DocIter {
        let v = std::slice::from_ref(self);
        DocIter {
            stack: vec![v.iter()],
        }
    }

    /// Returns an iterator over all value nodes in the document.
    /// The iterator yields tuples of (object-path, value-node).
    pub fn iter_path(&self) -> DocPathIter {
        let v = std::slice::from_ref(self);
        DocPathIter {
            stack: vec![v.iter()],
            aggregate: Vec::new(),
            path: Vec::new(),
        }
    }

    /// Returns a mutable iterator over all value nodes in the document.
    /// The iterator yields tuples of (object-path, value-node).
    pub fn iter_path_mut(&mut self) -> DocPathIterMut {
        let v = std::slice::from_mut(self);
        DocPathIterMut {
            stack: vec![v.iter_mut()],
            aggregate: Vec::new(),
            path: Vec::new(),
        }
    }
}

impl<'a> IntoIterator for &'a Document {
    type Item = &'a Document;
    type IntoIter = DocIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct DocIter<'a> {
    stack: Vec<std::slice::Iter<'a, Document>>,
}

impl<'a> Iterator for DocIter<'a> {
    type Item = &'a Document;
    fn next(&mut self) -> Option<Self::Item> {
        let val = loop {
            let top = match self.stack.last_mut() {
                Some(top) => top,
                None => return None,
            };
            if let Some(val) = top.next() {
                break val;
            }
            self.stack.pop();
        };
        match val {
            Document::Mapping(v) => self.stack.push(v.iter()),
            Document::Sequence(v) => self.stack.push(v.iter()),
            Document::Compact(v) => self.stack.push(std::slice::from_ref(&**v).iter()),
            Document::Fragment(v) => self.stack.push(v.iter()),
            _ => {}
        };
        Some(val)
    }
}

pub struct DocPathIter<'a> {
    stack: Vec<std::slice::Iter<'a, Document>>,
    aggregate: Vec<bool>,
    path: Vec<DocPath<'a>>,
}

pub struct DocPathIterMut<'a> {
    stack: Vec<std::slice::IterMut<'a, Document>>,
    aggregate: Vec<bool>,
    path: Vec<DocPath<'a>>,
}

#[derive(Debug, Clone)]
pub enum DocPath<'a> {
    Name(&'a str),
    Index(usize),
}

impl std::fmt::Display for DocPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocPath::Name(n) => write!(f, "{}", n),
            DocPath::Index(i) => write!(f, "{}", i),
        }
    }
}

impl<'a> Iterator for DocPathIter<'a> {
    type Item = (Vec<DocPath<'a>>, &'a Document);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.stack.last_mut() {
            let val = top.next();
            match val {
                Some(Document::Comment(_, _)) => {}
                Some(Document::Mapping(v)) => {
                    self.stack.push(v.iter());
                    self.path.push(DocPath::Name(""));
                    self.aggregate.push(true);
                }
                Some(Document::Sequence(v)) => {
                    self.stack.push(v.iter());
                    self.path.push(DocPath::Index(usize::MAX));
                    self.aggregate.push(true);
                }
                Some(Document::Compact(v)) => {
                    self.stack.push(std::slice::from_ref(&**v).iter());
                    self.aggregate.push(false);
                }
                Some(Document::Fragment(f)) => {
                    match self.path.last_mut() {
                        Some(DocPath::Name(ref mut n)) => match val.unwrap().as_kv() {
                            Ok((k, v)) => {
                                *n = k.as_str().expect("DocPath key");
                                self.stack.push(std::slice::from_ref(v).iter());
                            }
                            Err(_) => continue,
                        },
                        Some(DocPath::Index(_)) => match val.unwrap().as_value() {
                            Ok(v) => self.stack.push(std::slice::from_ref(v).iter()),
                            Err(_) => continue,
                        },
                        _ => {
                            self.stack.push(f.iter());
                        }
                    };
                    self.aggregate.push(false);
                }
                Some(_) => {
                    if let Some(DocPath::Index(ref mut i)) = self.path.last_mut() {
                        *i = i.wrapping_add(1);
                    }
                    return Some((self.path.clone(), val.unwrap()));
                }
                None => {
                    self.stack.pop();
                    if self.aggregate.pop() == Some(true) {
                        self.path.pop();
                    }
                }
            }
        }
        None
    }
}

impl<'a> Iterator for DocPathIterMut<'a> {
    type Item = (Vec<DocPath<'a>>, &'a mut Document);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.stack.last_mut() {
            let val = top.next();
            match val {
                Some(Document::Comment(_, _)) => {}
                Some(Document::Mapping(v)) => {
                    self.stack.push(v.iter_mut());
                    self.path.push(DocPath::Name(""));
                    self.aggregate.push(true);
                }
                Some(Document::Sequence(v)) => {
                    self.stack.push(v.iter_mut());
                    self.path.push(DocPath::Index(usize::MAX));
                    self.aggregate.push(true);
                }
                Some(Document::Compact(ref mut v)) => {
                    self.stack.push(std::slice::from_mut(&mut **v).iter_mut());
                    self.aggregate.push(false);
                }
                Some(Document::Fragment(_)) => {
                    let val = val.unwrap();
                    match self.path.last_mut() {
                        Some(DocPath::Name(ref mut n)) => match val.as_kv_mut() {
                            Ok((k, v)) => {
                                *n = k.as_str().expect("DocPath key");
                                self.stack.push(std::slice::from_mut(v).iter_mut());
                            }
                            Err(_) => continue,
                        },
                        Some(DocPath::Index(_)) => match val.as_value_mut() {
                            Ok(v) => self.stack.push(std::slice::from_mut(v).iter_mut()),
                            Err(_) => continue,
                        },
                        _ => {
                            // Unwrap is ok: we've already matched Document::Fragment.
                            self.stack.push(val.fragments_mut().unwrap().iter_mut());
                        }
                    };
                    self.aggregate.push(false);
                }
                Some(_) => {
                    if let Some(DocPath::Index(ref mut i)) = self.path.last_mut() {
                        *i = i.wrapping_add(1);
                    }
                    return Some((self.path.clone(), val.unwrap()));
                }
                None => {
                    self.stack.pop();
                    if self.aggregate.pop() == Some(true) {
                        self.path.pop();
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct SampleInner {
        k: u32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Sample {
        a: u32,
        b: u32,
        c: SampleInner,
        d: Vec<u32>,
    }

    const SAMPLE: &str = r#"
    {
        a: 1,
        b: 2,
        c: {
            k: 0,
        },
        d: [
            100,
            200,
        ]
    }"#;

    #[test]
    fn test_iter_path() -> Result<()> {
        let doc = Document::parse(SAMPLE)?;
        let items = doc
            .iter_path()
            .map(|(p, d)| {
                let path = p
                    .iter()
                    .map(DocPath::to_string)
                    .collect::<Vec<_>>()
                    .join(".");
                (path, d.try_into().unwrap())
            })
            .collect::<Vec<(String, u32)>>();
        assert_eq!(items[0], ("a".into(), 1));
        assert_eq!(items[1], ("b".into(), 2));
        assert_eq!(items[2], ("c.k".into(), 0));
        assert_eq!(items[3], ("d.0".into(), 100));
        assert_eq!(items[4], ("d.1".into(), 200));
        Ok(())
    }

    #[test]
    fn test_iter_path_mut() -> Result<()> {
        let expect = Sample {
            a: 1,
            b: 2,
            c: SampleInner { k: 99 },
            d: vec![100, 200],
        };

        // The sample text has "k: 0".  We examine the object path
        // and transform the value of that node into a 99.
        let sample = crate::Deserialize::try_from(SAMPLE)?
            .transform(|_, path_str, _doc| match path_str {
                "c.k" => Some(Document::Int(99u8.into())),
                _ => None,
            })
            .into::<Sample>()?;
        assert_eq!(sample, expect);
        Ok(())
    }
}
