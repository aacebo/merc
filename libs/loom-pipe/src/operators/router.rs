use crate::{Build, Operator, Source};

/// Route: send input to one of several operators based on predicates
pub struct Router<Input, Output> {
    routes: Vec<(
        Box<dyn Fn(&Input) -> bool + Send + Sync>,
        Box<dyn FnOnce(Source<Input>) -> Source<Output> + Send>,
    )>,
    default: Option<Box<dyn FnOnce(Source<Input>) -> Source<Output> + Send>>,
}

impl<Input, Output> Router<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            default: None,
        }
    }

    pub fn route<P, Op>(mut self, predicate: P, op: Op) -> Self
    where
        P: Fn(&Input) -> bool + Send + Sync + 'static,
        Op: Operator<Input, Output = Output> + Send + 'static,
    {
        self.routes.push((
            Box::new(predicate),
            Box::new(move |src: Source<Input>| op.apply(src)),
        ));
        self
    }

    pub fn default<Op>(mut self, op: Op) -> Self
    where
        Op: Operator<Input, Output = Output> + Send + 'static,
    {
        self.default = Some(Box::new(move |src: Source<Input>| op.apply(src)));
        self
    }
}

impl<Input, Output> Default for Router<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output> Operator<Input> for Router<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    type Output = Option<Output>;

    fn apply(mut self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let input = src.build();

            // Find matching route
            for (predicate, route_fn) in self.routes.into_iter() {
                if predicate(&input) {
                    let output = route_fn(Source::from(input)).build();
                    return Some(output);
                }
            }

            // Try default
            if let Some(default_fn) = self.default.take() {
                let output = default_fn(Source::from(input)).build();
                return Some(output);
            }

            None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pipe;
    use crate::operators::Guard;

    #[test]
    fn matches_first_route() {
        let result = Source::from(15)
            .pipe(
                Router::new()
                    .route(|x| *x > 10, Guard::allow(|_| true))
                    .route(|x| *x > 5, Guard::block(|_| true)),
            )
            .build();
        assert_eq!(result, Some(Some(15)));
    }

    #[test]
    fn matches_second_route() {
        let result = Source::from(7)
            .pipe(
                Router::new()
                    .route(|x| *x > 10, Guard::allow(|_| true))
                    .route(|x| *x > 5, Guard::allow(|_| true)),
            )
            .build();
        assert_eq!(result, Some(Some(7)));
    }

    #[test]
    fn no_match_returns_none() {
        let result = Source::from(3)
            .pipe(
                Router::new()
                    .route(|x| *x > 10, Guard::allow(|_| true))
                    .route(|x| *x > 5, Guard::allow(|_| true)),
            )
            .build();
        assert_eq!(result, None);
    }

    #[test]
    fn uses_default_when_no_match() {
        let result = Source::from(3)
            .pipe(
                Router::new()
                    .route(|x| *x > 10, Guard::allow(|_| true))
                    .default(Guard::allow(|_| true)),
            )
            .build();
        assert_eq!(result, Some(Some(3)));
    }

    #[test]
    fn route_takes_precedence_over_default() {
        let result = Source::from(15)
            .pipe(
                Router::new()
                    .route(|x| *x > 10, Guard::allow(|_| true))
                    .default(Guard::block(|_| true)),
            )
            .build();
        assert_eq!(result, Some(Some(15)));
    }

    #[test]
    fn with_strings() {
        let result = Source::from("hello".to_string())
            .pipe(
                Router::new()
                    .route(|s: &String| s.starts_with("h"), Guard::allow(|_| true))
                    .route(|s: &String| s.starts_with("w"), Guard::block(|_| true)),
            )
            .build();
        assert_eq!(result, Some(Some("hello".to_string())));
    }

    #[test]
    fn new_empty() {
        let router: Router<i32, Option<i32>> = Router::new();
        let result = Source::from(42).pipe(router).build();
        assert_eq!(result, None);
    }
}
