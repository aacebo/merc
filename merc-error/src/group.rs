use crate::Error;

#[derive(Debug, Default, Clone)]
pub struct ErrorGroup(Vec<Error>);

impl ErrorGroup {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add(mut self, error: Error) -> Self {
        self.0.push(error);
        self
    }
}

impl std::ops::Index<usize> for ErrorGroup {
    type Output = Error;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<const N: usize> From<[Error; N]> for ErrorGroup {
    fn from(value: [Error; N]) -> Self {
        Self(value.to_vec())
    }
}

impl From<&[Error]> for ErrorGroup {
    fn from(value: &[Error]) -> Self {
        Self(value.to_vec())
    }
}

impl From<Vec<Error>> for ErrorGroup {
    fn from(value: Vec<Error>) -> Self {
        Self(value)
    }
}

impl std::error::Error for ErrorGroup {}

impl std::fmt::Display for ErrorGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            write!(f, "{}", error)?;
        }

        Ok(())
    }
}
