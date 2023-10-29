use std::ops::Add;

use crate::writers::{MsgWriter, WriteError};

pub trait MsgPck {
    fn pack(&self, writer: &mut dyn MsgWriter) -> Result<(), PackError>;

    /// How big will the message be when packed?
    ///
    /// # Returns
    /// Tuple of `(min, max)`
    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Default)]
pub struct SizeHint {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl SizeHint {
    pub fn precise(size: usize) -> Self {
        Self {
            min: Some(size),
            max: Some(size),
        }
    }
}

impl Add for SizeHint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            min: match (self.min, rhs.min) {
                (Some(a), Some(b)) => Some(a + b),
                _ => None,
            },
            max: match (self.max, rhs.max) {
                (Some(a), Some(b)) => Some(a + b),
                _ => None,
            },
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum PackError {
    WriteError(WriteError),
}

impl From<WriteError> for PackError {
    fn from(e: WriteError) -> Self {
        Self::WriteError(e)
    }
}
