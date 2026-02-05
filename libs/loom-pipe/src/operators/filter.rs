use crate::{Build, Operator, Source};

/// Filter items in a Vec based on a predicate
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
}

impl<T> Operator<Vec<T>> for Filter<T>
where
    T: Send + 'static,
{
    type Output = Vec<T>;

    fn apply(self, src: Source<Vec<T>>) -> Source<Self::Output> {
        Source::new(move || {
            let items = src.build();
            items
                .into_iter()
                .filter(|item| (self.predicate)(item))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;

    #[test]
    fn keeps_matching_items() {
        let result = Source::from(vec![1, 2, 3, 4, 5])
            .pipe(Filter::new(|x| *x > 2))
            .build();
        assert_eq!(result, vec![3, 4, 5]);
    }

    #[test]
    fn removes_all_non_matching() {
        let result = Source::from(vec![1, 2, 3])
            .pipe(Filter::new(|x| *x > 10))
            .build();
        assert!(result.is_empty());
    }

    #[test]
    fn keeps_all_matching() {
        let result = Source::from(vec![10, 20, 30])
            .pipe(Filter::new(|x| *x > 0))
            .build();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn empty_vec() {
        let result = Source::from(Vec::<i32>::new())
            .pipe(Filter::new(|x| *x > 0))
            .build();
        assert!(result.is_empty());
    }

    #[test]
    fn strings_by_length() {
        let result = Source::from(vec![
            "a".to_string(),
            "abc".to_string(),
            "ab".to_string(),
            "abcd".to_string(),
        ])
        .pipe(Filter::new(|s: &String| s.len() >= 3))
        .build();
        assert_eq!(result, vec!["abc".to_string(), "abcd".to_string()]);
    }
}
