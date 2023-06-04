//! Three-way `Result` type.
//! 
//! This type extends the `Result` type to include a `Warn` variant.

pub enum TrioResult<T, W, E> {
    Ok(T),
    Warn(W),
    Err(E),
}

impl<T, W, E> TrioResult<T, W, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    pub fn is_warn(&self) -> bool {
        matches!(self, Self::Warn(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
}
