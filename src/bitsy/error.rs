use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub enum PathSegment {
    Index(usize),
    Name(String),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Index(i) => write!(f, "[{}]", i),
            PathSegment::Name(n) => write!(f, ".{}", n),
        }
    }
}

#[derive(Debug)]
pub enum BitsyErrorKind {
    EndOfData,
    InvalidData(String),
    MissingVersion,
    MissingContext(String),
}

pub struct BitsyError {
    kind: BitsyErrorKind,
    bit: usize,
    path: Vec<PathSegment>,
}

impl BitsyError {
    pub fn new(kind: BitsyErrorKind, bit: usize) -> Self {
        Self {
            kind,
            bit,
            path: Vec::new(),
        }
    }
}

impl Debug for BitsyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at bit {} (byte {}). Path: {}",
            self.kind,
            self.bit,
            self.bit / 8,
            self.path
                .iter()
                .rev()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

impl Display for BitsyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for BitsyError {}

pub trait BitsyErrorExt {
    fn prepend_path(self, segment: impl Into<String>) -> Self;
    fn prepend_index(self, index: usize) -> Self;
}

impl BitsyErrorExt for BitsyError {
    fn prepend_path(mut self, segment: impl Into<String>) -> Self {
        self.path.push(PathSegment::Name(segment.into()));
        self
    }

    fn prepend_index(mut self, index: usize) -> Self {
        self.path.push(PathSegment::Index(index));
        self
    }
}
