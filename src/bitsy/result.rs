use super::error::{BitsyError, BitsyErrorExt};

pub type BitsyResult<T> = Result<T, BitsyError>;

impl<T> BitsyErrorExt for BitsyResult<T> {
    fn prepend_path(self, segment: impl Into<String>) -> Self {
        self.map_err(|e| e.prepend_path(segment))
    }

    fn prepend_index(self, index: usize) -> Self {
        self.map_err(|e| e.prepend_index(index))
    }
}
