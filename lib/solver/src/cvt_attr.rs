//! Convert attributes to configs.

use ast::{Attribute, Modifier, Span};
use core::fmt;
use crate::core::{Solver, SolverError};
use intorinf::IntOrInf;
use provider::{
    core::FilterRedirect,
    LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig,
};
use std::{collections::{HashSet, HashMap}, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeError<'e> {
    /// This attribute conflicts with another attribute.
    Conflict(Span<'e>),
    /// This attribute is a duplicate of another attribute.
    Duplicate(Span<'e>),
    /// This attribute is invalid under this operation.
    Invalid,
}

impl<'e> fmt::Display for AttributeError<'e> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeError::Conflict(another) => write!(f, "attribute conflicts with another attribute at [{}:{}]", another.location_line(), another.get_utf8_column()),
            AttributeError::Duplicate(another) => write!(f, "attribute duplicates with another attribute at [{}:{}]", another.location_line(), another.get_utf8_column()),
            AttributeError::Invalid => write!(f, "invalid attribute"),
        }
    }
}

impl<'e> Error for AttributeError<'e> {}
unsafe impl<'e> Send for AttributeError<'e> {}

/// Convert a collection of `Attribute`s into a `LinksConfig` and a limit.
pub fn links_config_from_attributes<'e, S>(attrs: &[Attribute<'e>]) -> Result<(LinksConfig, Option<IntOrInf>), SolverError<'e, S>>
where
    S: Solver,
{
    // core things
    let mut config = LinksConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span<'e>> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `BackLinksConfig` and a limit.
pub fn backlinks_config_from_attributes<'e, S>(attrs: &[Attribute<'e>]) -> Result<(BackLinksConfig, Option<IntOrInf>), SolverError<'e, S>>
where
    S: Solver,
{
    // core things
    let mut config = BackLinksConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span<'e>> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Conflict(*span)));
                    } else {
                        resolved_at.insert("noredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Conflict(*span)));
                    } else {
                        resolved_at.insert("onlyredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                Modifier::Direct(item) => {
                    if let Some(span) = resolved_at.get("direct") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("direct", item.get_span());
                        config.direct = true;
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `EmbedsConfig` and a limit.
pub fn embeds_config_from_attributes<'e, S>(attrs: &[Attribute<'e>]) -> Result<(EmbedsConfig, Option<IntOrInf>), SolverError<'e, S>>
where
    S: Solver,
{
    // core things
    let mut config = EmbedsConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span<'e>> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Conflict(*span)));
                    } else {
                        resolved_at.insert("noredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Conflict(*span)));
                    } else {
                        resolved_at.insert("onlyredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `CategoryMembersConfig` and a limit and a depth.
pub fn categorymembers_config_from_attributes<'e, S>(attrs: &[Attribute<'e>]) -> Result<(CategoryMembersConfig, Option<IntOrInf>, Option<IntOrInf>), SolverError<'e, S>>
where
    S: Solver,
{
    // core things
    let mut config = CategoryMembersConfig::default();
    let mut limit: Option<IntOrInf> = None;
    let mut depth: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span<'e>> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::Depth(item) => {
                    if let Some(span) = resolved_at.get("depth") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("depth", item.get_span());
                        depth = Some(item.val.val);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit, depth))
}

/// Convert a collection of `Attribute`s into a `PrefixConfig` and a limit.
pub fn prefix_config_from_attributes<'e, S>(attrs: &[Attribute<'e>]) -> Result<(PrefixConfig, Option<IntOrInf>), SolverError<'e, S>>
where
    S: Solver,
{
    // core things
    let mut config = PrefixConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span<'e>> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Conflict(*span)));
                    } else {
                        resolved_at.insert("noredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Duplicate(*span)));
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Conflict(*span)));
                    } else {
                        resolved_at.insert("onlyredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}
