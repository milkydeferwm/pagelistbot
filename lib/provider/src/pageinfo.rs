//! Definition and implementations for `PageInfo` and `Pair`.

use core::{
    cmp::Ordering,
    fmt, mem,
};
use mwtitle::Title;
use std::error::Error;

/// a struct holding the queried wiki page information.
#[derive(Debug, Clone)]
pub struct PageInfo {
    title: Option<Title>,
    exists: Option<bool>,
    redirect: Option<bool>,
    assoc_title: Option<Title>,
    assoc_exists: Option<bool>,
    assoc_redirect: Option<bool>,
}

impl PageInfo {
    /// creates a new `PageInfo` instance.
    pub fn new(
        title: Option<Title>, exists: Option<bool>, redirect: Option<bool>,
        assoc_title: Option<Title>, assoc_exists: Option<bool>, assoc_redirect: Option<bool>
    ) -> Self {
        Self { title, exists, redirect, assoc_title, assoc_exists, assoc_redirect }
    }

    pub fn new_swap(&self) -> Self {
        let mut new = self.clone();
        new.swap();
        new
    }

    /// get a reference to the title, returns an error if such value is not known aka not stored.
    pub fn get_title(&self) -> Result<&Title, PageInfoError> {
        self.title.as_ref().ok_or(PageInfoError::UnknownValue)
    }

    /// get a bool indicating whether this page exists on the wiki, returns an error if such value is not known aka not stored.
    pub fn get_exists(&self) -> Result<bool, PageInfoError> {
        self.exists.ok_or(PageInfoError::UnknownValue)
    }

    /// get a bool indicating whether this page is a redirect page, returns an error if such value is not known aka not stored.
    pub fn get_isredir(&self) -> Result<bool, PageInfoError> {
        self.redirect.ok_or(PageInfoError::UnknownValue)
    }

    /// Swap the subject page's information and the associated page's information.
    pub fn swap(&mut self) {
        mem::swap(&mut self.title, &mut self.assoc_title);
        mem::swap(&mut self.exists, &mut self.assoc_exists);
        mem::swap(&mut self.redirect, &mut self.assoc_redirect);
    }
}

impl From<PageInfo> for Title {
    fn from(f: PageInfo) -> Self {
        f.title.unwrap()
    }
}

impl PartialOrd for PageInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PageInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.title.cmp(&other.title)
    }
}

impl Eq for PageInfo {}
impl PartialEq for PageInfo {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageInfoError {
    UnknownValue,
}

impl Error for PageInfoError {}
impl fmt::Display for PageInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownValue => write!(f, "unknown value"),
        }
    }
}
