use crate::{Build, Operator, Pipe, Source};

/// Filter: gate a single value based on a predicate (T -> Option<T>)
pub struct Filter<T> {
    predicate: Box<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> Filter<T>
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

    /// Create a filter that allows values matching the predicate (alias for new)
    pub fn allow<P>(predicate: P) -> Self
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self::new(predicate)
    }

    /// Create a filter that blocks values matching the predicate
    pub fn block<P>(predicate: P) -> Self
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self::new(move |x| !predicate(x))
    }
}

impl<T> Operator<T> for Filter<T>
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

/// Extension trait for filtering single values (T -> Option<T>)
pub trait FilterPipe<T>: Pipe<T> + Sized
where
    T: Send + 'static,
{
    fn filter<P>(self, predicate: P) -> Source<Option<T>>
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.pipe(Filter::new(predicate))
    }

    fn filter_allow<P>(self, predicate: P) -> Source<Option<T>>
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.pipe(Filter::allow(predicate))
    }

    fn filter_block<P>(self, predicate: P) -> Source<Option<T>>
    where
        P: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.pipe(Filter::block(predicate))
    }
}

impl<T: Send + 'static, P: Pipe<T> + Sized> FilterPipe<T> for P {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_matching_single_value() {
        let result = Source::from(42).pipe(Filter::allow(|x| *x > 0)).build();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn blocks_non_matching_single_value() {
        let result = Source::from(-5).pipe(Filter::allow(|x| *x > 0)).build();
        assert_eq!(result, None);
    }

    #[test]
    fn block_blocks_matching_value() {
        let result = Source::from(42).pipe(Filter::block(|x| *x > 0)).build();
        assert_eq!(result, None);
    }

    #[test]
    fn block_allows_non_matching_value() {
        let result = Source::from(-5).pipe(Filter::block(|x| *x > 0)).build();
        assert_eq!(result, Some(-5));
    }

    #[test]
    fn filter_pipe_trait_single() {
        let result = Source::from(42).filter(|x| *x > 0).build();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn filter_allow_pipe_trait() {
        let result = Source::from(42).filter_allow(|x| *x > 0).build();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn filter_block_pipe_trait() {
        let result = Source::from(42).filter_block(|x| *x > 0).build();
        assert_eq!(result, None);
    }
}
