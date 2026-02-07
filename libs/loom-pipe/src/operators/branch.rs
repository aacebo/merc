use std::marker::PhantomData;

use crate::{Build, Operator, Pipe, Source};

/// Branch operator for conditional execution
pub struct Branch<Input, Output, C, T, E>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    condition: C,
    then_fn: T,
    else_fn: Option<E>,
    _marker: PhantomData<fn(Input) -> Output>,
}

impl<Input, Output, C, T, E> Branch<Input, Output, C, T, E>
where
    Input: Send + 'static,
    Output: Send + 'static,
    C: FnOnce(&Input) -> bool + Send + 'static,
    T: FnOnce(Input) -> Output + Send + 'static,
    E: FnOnce(Input) -> Output + Send + 'static,
{
    pub fn new(condition: C, then_fn: T, else_fn: Option<E>) -> Self {
        Self {
            condition,
            then_fn,
            else_fn,
            _marker: PhantomData,
        }
    }
}

impl<Input, Output, C, T, E> Operator<Input> for Branch<Input, Output, C, T, E>
where
    Input: Send + 'static,
    Output: Send + 'static,
    C: FnOnce(&Input) -> bool + Send + 'static,
    T: FnOnce(Input) -> Output + Send + 'static,
    E: FnOnce(Input) -> Output + Send + 'static,
{
    type Output = Output;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let value = src.build();
            if (self.condition)(&value) {
                (self.then_fn)(value)
            } else if let Some(else_fn) = self.else_fn {
                (else_fn)(value)
            } else {
                panic!("Branch: condition was false but no else branch provided")
            }
        })
    }
}

/// Branch operator that passes through input when condition is false
pub struct BranchPassthrough<Input, C, T>
where
    Input: Send + 'static,
{
    condition: C,
    then_fn: T,
    _marker: PhantomData<fn(Input) -> Input>,
}

impl<Input, C, T> BranchPassthrough<Input, C, T>
where
    Input: Send + 'static,
    C: FnOnce(&Input) -> bool + Send + 'static,
    T: FnOnce(Input) -> Input + Send + 'static,
{
    pub fn new(condition: C, then_fn: T) -> Self {
        Self {
            condition,
            then_fn,
            _marker: PhantomData,
        }
    }
}

impl<Input, C, T> Operator<Input> for BranchPassthrough<Input, C, T>
where
    Input: Send + 'static,
    C: FnOnce(&Input) -> bool + Send + 'static,
    T: FnOnce(Input) -> Input + Send + 'static,
{
    type Output = Input;

    fn apply(self, src: Source<Input>) -> Source<Self::Output> {
        Source::new(move || {
            let value = src.build();
            if (self.condition)(&value) {
                (self.then_fn)(value)
            } else {
                value
            }
        })
    }
}

// ============================================================================
// Builder Pattern
// ============================================================================

/// Extension trait for starting branch execution
pub trait BranchPipe<T>: Pipe<T> + Sized
where
    T: Send + 'static,
{
    fn branch(self) -> BranchBuilderInit<T, Self> {
        BranchBuilderInit::new(self)
    }
}

impl<T: Send + 'static, P: Pipe<T> + Sized> BranchPipe<T> for P {}

/// Initial builder state - needs condition
pub struct BranchBuilderInit<T, P> {
    source: P,
    _marker: PhantomData<T>,
}

impl<T, P> BranchBuilderInit<T, P>
where
    T: Send + 'static,
    P: Pipe<T>,
{
    fn new(source: P) -> Self {
        Self {
            source,
            _marker: PhantomData,
        }
    }

    /// Set the condition for branching
    pub fn when<C>(self, condition: C) -> BranchBuilderWithCondition<T, P, C>
    where
        C: FnOnce(&T) -> bool + Send + 'static,
    {
        BranchBuilderWithCondition {
            source: self.source,
            condition,
            _marker: PhantomData,
        }
    }
}

/// Builder with condition set - needs then branch
pub struct BranchBuilderWithCondition<T, P, C> {
    source: P,
    condition: C,
    _marker: PhantomData<T>,
}

impl<T, P, C> BranchBuilderWithCondition<T, P, C>
where
    T: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
{
    /// Set the then branch (executed when condition is true)
    pub fn then<O, F>(self, then_fn: F) -> BranchBuilderWithThen<T, O, P, C, F>
    where
        O: Send + 'static,
        F: FnOnce(T) -> O + Send + 'static,
    {
        BranchBuilderWithThen {
            source: self.source,
            condition: self.condition,
            then_fn,
            _marker: PhantomData,
        }
    }
}

/// Builder with condition and then - can add else or build (if T == O)
pub struct BranchBuilderWithThen<T, O, P, C, F> {
    source: P,
    condition: C,
    then_fn: F,
    _marker: PhantomData<(T, O)>,
}

impl<T, O, P, C, F> BranchBuilderWithThen<T, O, P, C, F>
where
    T: Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    F: FnOnce(T) -> O + Send + 'static,
{
    /// Set the else branch (executed when condition is false)
    pub fn or_else<E>(self, else_fn: E) -> BranchBuilderComplete<T, O, P, C, F, E>
    where
        E: FnOnce(T) -> O + Send + 'static,
    {
        BranchBuilderComplete {
            source: self.source,
            condition: self.condition,
            then_fn: self.then_fn,
            else_fn,
            _marker: PhantomData,
        }
    }
}

// Allow build() when T == O (passthrough case)
impl<T, P, C, F> BranchBuilderWithThen<T, T, P, C, F>
where
    T: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    F: FnOnce(T) -> T + Send + 'static,
{
    /// Build the branch - passes through input when condition is false
    pub fn done(self) -> Source<T> {
        self.source
            .pipe(BranchPassthrough::new(self.condition, self.then_fn))
    }
}

impl<T, P, C, F> Build for BranchBuilderWithThen<T, T, P, C, F>
where
    T: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    F: FnOnce(T) -> T + Send + 'static,
{
    type Output = T;

    fn build(self) -> Self::Output {
        self.done().build()
    }
}

impl<T, P, C, F> Pipe<T> for BranchBuilderWithThen<T, T, P, C, F>
where
    T: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    F: FnOnce(T) -> T + Send + 'static,
{
    fn pipe<Op: Operator<T>>(self, op: Op) -> Source<Op::Output> {
        self.done().pipe(op)
    }
}

/// Complete builder with condition, then, and else
pub struct BranchBuilderComplete<T, O, P, C, Then, Else> {
    source: P,
    condition: C,
    then_fn: Then,
    else_fn: Else,
    _marker: PhantomData<(T, O)>,
}

impl<T, O, P, C, Then, Else> BranchBuilderComplete<T, O, P, C, Then, Else>
where
    T: Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    Then: FnOnce(T) -> O + Send + 'static,
    Else: FnOnce(T) -> O + Send + 'static,
{
    /// Build the branch operator
    pub fn done(self) -> Source<O> {
        self.source.pipe(Branch::new(
            self.condition,
            self.then_fn,
            Some(self.else_fn),
        ))
    }
}

impl<T, O, P, C, Then, Else> Build for BranchBuilderComplete<T, O, P, C, Then, Else>
where
    T: Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    Then: FnOnce(T) -> O + Send + 'static,
    Else: FnOnce(T) -> O + Send + 'static,
{
    type Output = O;

    fn build(self) -> Self::Output {
        self.done().build()
    }
}

impl<T, O, P, C, Then, Else> Pipe<O> for BranchBuilderComplete<T, O, P, C, Then, Else>
where
    T: Send + 'static,
    O: Send + 'static,
    P: Pipe<T>,
    C: FnOnce(&T) -> bool + Send + 'static,
    Then: FnOnce(T) -> O + Send + 'static,
    Else: FnOnce(T) -> O + Send + 'static,
{
    fn pipe<Op: Operator<O>>(self, op: Op) -> Source<Op::Output> {
        self.done().pipe(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_with_else_true_condition() {
        let result = Source::from(10)
            .branch()
            .when(|x| *x > 5)
            .then(|x| x * 2)
            .or_else(|x| x + 100)
            .build();

        assert_eq!(result, 20);
    }

    #[test]
    fn branch_with_else_false_condition() {
        let result = Source::from(3)
            .branch()
            .when(|x| *x > 5)
            .then(|x| x * 2)
            .or_else(|x| x + 100)
            .build();

        assert_eq!(result, 103);
    }

    #[test]
    fn branch_passthrough_true_condition() {
        let result = Source::from(10)
            .branch()
            .when(|x| *x > 5)
            .then(|x| x * 2)
            .build();

        assert_eq!(result, 20);
    }

    #[test]
    fn branch_passthrough_false_condition() {
        let result = Source::from(3)
            .branch()
            .when(|x| *x > 5)
            .then(|x| x * 2)
            .build();

        assert_eq!(result, 3); // passthrough
    }

    #[test]
    fn branch_with_strings() {
        let result = Source::from("hello".to_string())
            .branch()
            .when(|s| s.len() > 3)
            .then(|s| s.to_uppercase())
            .or_else(|s| s.repeat(2))
            .build();

        assert_eq!(result, "HELLO");
    }

    #[test]
    fn branch_with_type_change() {
        let result = Source::from(42)
            .branch()
            .when(|x| *x > 0)
            .then(|x| format!("positive: {}", x))
            .or_else(|x| format!("non-positive: {}", x))
            .build();

        assert_eq!(result, "positive: 42");
    }

    #[test]
    fn branch_chainable() {
        use crate::MapPipe;

        let result = Source::from(10)
            .branch()
            .when(|x| *x > 5)
            .then(|x| x * 2)
            .or_else(|x| x + 100)
            .map(|x| x + 1)
            .build();

        assert_eq!(result, 21);
    }

    #[test]
    fn branch_passthrough_chainable() {
        use crate::MapPipe;

        let result = Source::from(10)
            .branch()
            .when(|x| *x > 5)
            .then(|x| x * 2)
            .map(|x| x + 1)
            .build();

        assert_eq!(result, 21);
    }
}
