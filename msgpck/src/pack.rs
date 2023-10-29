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

    #[cfg(feature = "alloc")]
    fn pack_vec(&self) -> Result<Vec<u8>, PackError> {
        let min_size = self.size_hint().min.unwrap_or(0);
        let mut writer = Vec::with_capacity(min_size);
        self.pack(&mut writer)?;
        Ok(writer)
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
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum PackError {
    #[cfg_attr(feature = "std", error("Write error"))]
    WriteError { source: WriteError },
}

impl From<WriteError> for PackError {
    fn from(source: WriteError) -> Self {
        Self::WriteError { source }
    }
}
