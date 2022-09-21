//! Functions for manipulating tags on files.
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::color::Color;
use crate::xattr::{list_xattrs, remove_xattr, set_xattr, Xattr};
use crate::{Error, Result, WUTAG_NAMESPACE};

pub const DEFAULT_COLOR: Color = Color::BrightWhite;

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct Tag {
    name: String,
    color: Color,
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Tag {
    pub fn new<S>(name: S, color: Color) -> Self
    where
        S: Into<String>,
    {
        Tag {
            name: name.into(),
            color,
        }
    }

    pub fn random<S>(name: S, colors: &[Color]) -> Self
    where
        S: Into<String>,
    {
        let mut rng = thread_rng();
        Tag::new(
            name,
            colors.choose(&mut rng).cloned().unwrap_or(DEFAULT_COLOR),
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn set_color(&mut self, color: &Color) {
        self.color = *color;
    }

    fn hash(&self) -> String {
        format!("{}.{}", WUTAG_NAMESPACE, base64::encode(&self.name))
    }

    /// Tags the file at the given `path` with this tag. If the tag exists returns an error.
    pub fn save_to<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        for tag in list_tags(path.as_ref())? {
            if &tag == self {
                return Err(Error::TagExists);
            }
        }
        set_xattr(path, self.hash().as_str(), "")
    }

    /// Removes this tag from the file at the given `path`. If the tag doesn't exists returns
    /// [Error::TagNotFound](wutag::Error::TagNotFound)
    pub fn remove_from<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let hash = self.hash();

        for xattr in list_xattrs(path.as_ref())? {
            let key = xattr.key();
            // make sure to only remove attributes corresponding to this namespace
            if key == hash {
                return remove_xattr(path, key);
            }
        }

        Err(Error::TagNotFound(self.name.clone()))
    }

    /// Consumes this tag returing it's name
    pub fn into_name(self) -> String {
        self.name
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

macro_rules! next_or_else {
    ($it:ident, $msg:expr) => {
        $it.next()
            .ok_or_else(|| Error::InvalidTagKey($msg.to_string()))
    };
}

impl TryFrom<Xattr> for Tag {
    type Error = Error;
    fn try_from(xattr: Xattr) -> Result<Self> {
        let key = xattr.key();

        let mut elems = key.split("wutag.");

        let ns = next_or_else!(elems, "missing namespace `user`")?;
        if ns != "user." {
            return Err(Error::InvalidTagKey(format!(
                "invalid namespace `{}`, valid namespace is `user`",
                ns
            )));
        }

        let tag_bytes = next_or_else!(elems, "missing tag")?;
        let tag = serde_cbor::from_slice(&base64::decode(tag_bytes.as_bytes())?)?;

        Ok(tag)
    }
}

pub fn get_tag<P, T>(path: P, tag: T) -> Result<Tag>
where
    P: AsRef<Path>,
    T: AsRef<str>,
{
    let path = path.as_ref();
    let tag = tag.as_ref();
    for _tag in list_xattrs(path)?.into_iter().flat_map(Tag::try_from) {
        if _tag.name == tag {
            return Ok(_tag);
        }
    }

    Err(Error::TagNotFound(tag.to_string()))
}

/// Lists tags of the file at the given `path`.
pub fn list_tags<P>(path: P) -> Result<Vec<Tag>>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        let mut tags = Vec::new();
        let it = attrs
            .into_iter()
            .filter(|xattr| xattr.key().starts_with(WUTAG_NAMESPACE))
            .map(Tag::try_from);

        for tag in it.flatten() {
            tags.push(tag);
        }
        tags
    })
}

/// Lists tags of the file at the given `path` as a [BTreeSet](BTreeSet).
pub fn list_tags_btree<P>(path: P) -> Result<BTreeSet<Tag>>
where
    P: AsRef<Path>,
{
    list_xattrs(path).map(|attrs| {
        let mut tags = BTreeSet::new();
        let it = attrs
            .into_iter()
            .filter(|xattr| xattr.key().starts_with(WUTAG_NAMESPACE))
            .map(Tag::try_from);

        for tag in it.flatten() {
            tags.insert(tag);
        }
        tags
    })
}

/// Clears all tags of the file at the given `path`.
pub fn clear_tags<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    for xattr in list_xattrs(path.as_ref())?
        .iter()
        .filter(|xattr| xattr.key().starts_with(WUTAG_NAMESPACE))
    {
        remove_xattr(path.as_ref(), xattr.key())?;
    }

    Ok(())
}

/// Checks whether the given path has any tags.
///
/// Returns an Error if the list of tags couldn't be aquired.
pub fn has_tags<P>(path: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    list_tags(path).map(|tags| !tags.is_empty())
}
