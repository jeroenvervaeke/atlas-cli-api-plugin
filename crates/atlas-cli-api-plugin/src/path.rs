use std::{borrow::Cow, fmt::Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Path<'a>(pub Vec<PathSegment<'a>>);

impl Display for Path<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, segment) in self.0.iter().enumerate() {
            if idx != 0 {
                write!(f, "/")?;
            }

            segment.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PathSegment<'a> {
    Constant(Cow<'a, str>),
    Id(Cow<'a, str>),
}

impl Display for PathSegment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Constant(constant) => write!(f, "{constant}"),
            PathSegment::Id(id) => write!(f, "{id}"),
        }
    }
}

impl<'a> Path<'a> {
    pub fn from_str(str: &'a str) -> Option<Self> {
        let segments: Vec<_> = str
            .split('/')
            .into_iter()
            .filter_map(PathSegment::from_str)
            .collect();

        if segments.is_empty() {
            return None;
        }

        Some(Path(segments))
    }

    pub fn from_str_owned(str: &'a str) -> Option<Path<'static>> {
        let segments: Vec<_> = str
            .split('/')
            .into_iter()
            .filter_map(PathSegment::from_str_owned)
            .collect();

        if segments.is_empty() {
            return None;
        }

        Some(Path(segments))
    }

    pub fn consts<'b>(&'b self) -> impl Iterator<Item = &'b str> {
        self.0.iter().filter_map(|s| match s {
            PathSegment::Constant(c) => Some(c.as_ref()),
            PathSegment::Id(_) => None,
        })
    }
}

impl<'a> PathSegment<'a> {
    pub fn from_str(str: &'a str) -> Option<Self> {
        if str.is_empty() {
            return None;
        }

        // try to strip { and } once, if one of the operations fails this returns none
        Some(
            match str.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
                Some(id) => PathSegment::Id(Cow::Borrowed(id)),
                None => PathSegment::Constant(Cow::Borrowed(str)),
            },
        )
    }

    pub fn from_str_owned(str: &'a str) -> Option<PathSegment<'static>> {
        if str.is_empty() {
            return None;
        }

        // try to strip { and } once, if one of the operations fails this returns none
        Some(
            match str.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
                Some(id) => PathSegment::Id(Cow::Owned(id.to_owned())),
                None => PathSegment::Constant(Cow::Owned(str.to_owned())),
            },
        )
    }
}

// impl order for path segment
// order id above constant, no particular reason
impl<'a> Ord for PathSegment<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (PathSegment::Constant(v1), PathSegment::Constant(v2)) => v1.cmp(v2),
            (PathSegment::Constant(_), PathSegment::Id(_)) => std::cmp::Ordering::Greater,
            (PathSegment::Id(_), PathSegment::Constant(_)) => std::cmp::Ordering::Less,
            (PathSegment::Id(v1), PathSegment::Id(v2)) => v1.cmp(v2),
        }
    }
}

impl<'a> PartialOrd for PathSegment<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
