use ast::Span;
use core::{fmt::{self, Display, Debug}};
use provider::{DataProvider, PageInfoError};
use std::error::Error;

#[derive(PartialEq, Eq)]
#[non_exhaustive]
pub enum RuntimeWarning<P: DataProvider> {
    Provider { span: Span, warn: P::Warn },
    ResultLimitExceeded { span: Span, limit: usize },
}

impl<P> Error for RuntimeWarning<P>
where
    P: DataProvider,
    P::Warn: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RuntimeWarning::Provider { warn, .. } => Some(warn),
            RuntimeWarning::ResultLimitExceeded { .. } => None,
        }
    }
}

impl<P> Display for RuntimeWarning<P>
where
    P: DataProvider,
    P::Warn: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeWarning::Provider { span, warn } => f.write_fmt(format_args!("provider warning at `{}:{}`: {}", span.start, span.end, warn)),
            RuntimeWarning::ResultLimitExceeded { span, limit } => f.write_fmt(format_args!("result limit `{}` exceeded at `{}:{}`", limit, span.start, span.end)),
        }
    }
}

impl<P> Debug for RuntimeWarning<P>
where
    P: DataProvider,
    P::Warn: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider { span, warn } => f.debug_struct("Provider").field("span", span).field("warn", warn).finish(),
            Self::ResultLimitExceeded { span, limit } => f.debug_struct("ResultLimitExceeded").field("span", span).field("limit", limit).finish(),
        }
    }
}

#[derive(PartialEq, Eq)]
#[non_exhaustive]
pub enum RuntimeError<P: DataProvider> {
    Provider { span: Span, error: P::Error },
    PageInfo { span: Span, error: PageInfoError },
}

impl<P> Error for RuntimeError<P>
where
    P: DataProvider,
    P::Error: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RuntimeError::Provider { error, .. } => Some(error),
            RuntimeError::PageInfo { error, .. } => Some(error),
        }
    }
}

impl<P> Display for RuntimeError<P>
where
    P: DataProvider,
    P::Error: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Provider { span, error } => f.write_fmt(format_args!("provider error at `{}:{}`: {}", span.start, span.end, error)),
            RuntimeError::PageInfo { span, error } => f.write_fmt(format_args!("page info error at `{}:{}`: {}", span.start, span.end, error)),
        }
    }
}

impl<P> Debug for RuntimeError<P>
where
    P: DataProvider,
    P::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider { span, error } => f.debug_struct("Provider").field("span", span).field("error", error).finish(),
            Self::PageInfo { span, error } => f.debug_struct("PageInfo").field("span", span).field("error", error).finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticError {
    /// This attribute conflicts with another attribute.
    ConflictAttribute { span: Span, other: Span },
    /// This attribute is a duplicate of another attribute.
    DuplicateAttribute { span: Span, other: Span },
    /// This attribute is invalid under this operation.
    InvalidAttribute { span: Span },
}

impl Error for SemanticError {}
impl Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictAttribute { span, other } => f.write_fmt(format_args!("conflict attributes at `{}:{}` and `{}:{}`", span.start, span.end, other.start, other.end)),
            Self::DuplicateAttribute { span, other } => f.write_fmt(format_args!("duplicate attributes at `{}:{}` and `{}:{}`", span.start, span.end, other.start, other.end)),
            Self::InvalidAttribute { span } => f.write_fmt(format_args!("invalid attribute at `{}:{}`", span.start, span.end)),
        }
    }
}
