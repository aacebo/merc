use crate::{Build, Operator, Source};

/// Guard: conditionally allow or block pipeline continuation
/// Returns Option<T> - Some(input) if allowed, None if blocked
pub struct Guard<T> {
    predicate: Box<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> Guard<T>
where
    T: Send + 'static,
{
    pub fn new<P>(predicate: P) -> Self
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
        }
    }

    /// Create a guard that allows values matching the predicate
    pub fn allow<P>(predicate: P) -> Self
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self::new(predicate)
    }

    /// Create a guard that blocks values matching the predicate
    pub fn block<P>(predicate: P) -> Self
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self::new(move |x| !predicate(x))
    }
}

impl<T> Operator<T> for Guard<T>
where
    T: Send + 'static,
{
    type Output = Option<T>;

    fn apply(self, src: Source<T>) -> Source<Self::Output> {
        Source::new(move || {
            let input = src.build();
            if (self.predicate)(&input) {
                Some(input)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;

    #[test]
    fn allows_matching_value() {
        let result = Source::from(42).pipe(Guard::allow(|x| *x > 0)).build();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn blocks_non_matching_value() {
        let result = Source::from(-5).pipe(Guard::allow(|x| *x > 0)).build();
        assert_eq!(result, None);
    }

    #[test]
    fn block_blocks_matching_value() {
        let result = Source::from(42).pipe(Guard::block(|x| *x > 0)).build();
        assert_eq!(result, None);
    }

    #[test]
    fn block_allows_non_matching_value() {
        let result = Source::from(-5).pipe(Guard::block(|x| *x > 0)).build();
        assert_eq!(result, Some(-5));
    }

    #[test]
    fn with_string() {
        let result = Source::from("hello".to_string())
            .pipe(Guard::allow(|s: &String| s.len() > 3))
            .build();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn with_empty_string() {
        let result = Source::from("".to_string())
            .pipe(Guard::allow(|s: &String| !s.is_empty()))
            .build();
        assert_eq!(result, None);
    }

    #[test]
    fn new_same_as_allow() {
        let result1 = Source::from(10).pipe(Guard::new(|x| *x > 5)).build();
        let result2 = Source::from(10).pipe(Guard::allow(|x| *x > 5)).build();
        assert_eq!(result1, result2);
    }
}
