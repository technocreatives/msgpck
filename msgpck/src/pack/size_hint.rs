use core::ops::Add;

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
pub struct SizeHint {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl SizeHint {
    #[inline]
    pub fn precise(size: usize) -> Self {
        Self {
            min: Some(size),
            max: Some(size),
        }
    }
}

impl Add for SizeHint {
    type Output = Self;

    #[inline]
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
