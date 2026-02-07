use std::marker::PhantomData;

use crate::{Build, Operator, Pipe, Source};

/// Flatten operator - flattens nested sequences
pub struct Flatten<T> {
    _marker: PhantomData<T>,
}

impl<T> Flatten<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for Flatten<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Inner> Operator<Vec<Inner>> for Flatten<T>
where
    Inner: IntoIterator<Item = T> + Send + 'static,
    T: Send + 'static,
{
    type Output = Vec<T>;

    fn apply(self, src: Source<Vec<Inner>>) -> Source<Self::Output> {
        Source::new(move || src.build().into_iter().flatten().collect())
    }
}

/// FlatMap operator - maps and flattens in one step
pub struct FlatMap<F> {
    f: F,
}

impl<F> FlatMap<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<T, U, F> Operator<Vec<T>> for FlatMap<F>
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T) -> Vec<U> + Send + 'static,
{
    type Output = Vec<U>;

    fn apply(self, src: Source<Vec<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().into_iter().flat_map(|x| (self.f)(x)).collect())
    }
}

/// Chunk operator - splits into fixed-size chunks
pub struct Chunk {
    size: usize,
}

impl Chunk {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl<T> Operator<Vec<T>> for Chunk
where
    T: Clone + Send + 'static,
{
    type Output = Vec<Vec<T>>;

    fn apply(self, src: Source<Vec<T>>) -> Source<Self::Output> {
        Source::new(move || src.build().chunks(self.size).map(|c| c.to_vec()).collect())
    }
}

/// Window operator - sliding window over elements
pub struct Window {
    size: usize,
}

impl Window {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl<T> Operator<Vec<T>> for Window
where
    T: Clone + Send + 'static,
{
    type Output = Vec<Vec<T>>;

    fn apply(self, src: Source<Vec<T>>) -> Source<Self::Output> {
        Source::new(move || {
            let items = src.build();
            if items.len() < self.size {
                return vec![];
            }
            items.windows(self.size).map(|w| w.to_vec()).collect()
        })
    }
}

/// Concat operator - concatenates sequences
pub struct Concat<T> {
    other: Vec<T>,
}

impl<T> Concat<T> {
    pub fn new(other: Vec<T>) -> Self {
        Self { other }
    }
}

impl<T> Operator<Vec<T>> for Concat<T>
where
    T: Send + 'static,
{
    type Output = Vec<T>;

    fn apply(self, src: Source<Vec<T>>) -> Source<Self::Output> {
        Source::new(move || {
            let mut result = src.build();
            result.extend(self.other);
            result
        })
    }
}

/// Extension trait for sequence operations on Vec<T>
pub trait SequencePipe<T>: Pipe<Vec<T>> + Sized
where
    T: Send + 'static,
{
    /// Flattens nested sequences: Vec<Vec<T>> -> Vec<T>
    fn flatten<Inner>(self) -> Source<Vec<Inner>>
    where
        T: IntoIterator<Item = Inner>,
        Inner: Send + 'static,
    {
        self.pipe(Flatten::new())
    }

    /// Maps each element and flattens the results
    fn flat_map<U, F>(self, f: F) -> Source<Vec<U>>
    where
        U: Send + 'static,
        F: Fn(T) -> Vec<U> + Send + 'static,
    {
        self.pipe(FlatMap::new(f))
    }

    /// Splits into fixed-size chunks
    fn chunk(self, size: usize) -> Source<Vec<Vec<T>>>
    where
        T: Clone,
    {
        self.pipe(Chunk::new(size))
    }

    /// Creates overlapping windows of the specified size
    fn window(self, size: usize) -> Source<Vec<Vec<T>>>
    where
        T: Clone,
    {
        self.pipe(Window::new(size))
    }

    /// Concatenates another sequence to this one
    fn concat(self, other: Vec<T>) -> Source<Vec<T>> {
        self.pipe(Concat::new(other))
    }
}

impl<T: Send + 'static, P: Pipe<Vec<T>> + Sized> SequencePipe<T> for P {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_nested_vectors() {
        let result = Source::from(vec![vec![1, 2], vec![3, 4], vec![5]])
            .flatten()
            .build();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn flatten_empty() {
        let result: Vec<i32> = Source::from(Vec::<Vec<i32>>::new()).flatten().build();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn flat_map_expands() {
        let result = Source::from(vec![1, 2, 3])
            .flat_map(|x| vec![x, x * 10])
            .build();
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn flat_map_empty_results() {
        let result = Source::from(vec![1, 2, 3])
            .flat_map(|_| Vec::<i32>::new())
            .build();
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn chunk_exact_fit() {
        let result = Source::from(vec![1, 2, 3, 4]).chunk(2).build();
        assert_eq!(result, vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn chunk_with_remainder() {
        let result = Source::from(vec![1, 2, 3, 4, 5]).chunk(2).build();
        assert_eq!(result, vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn chunk_larger_than_input() {
        let result = Source::from(vec![1, 2]).chunk(5).build();
        assert_eq!(result, vec![vec![1, 2]]);
    }

    #[test]
    fn window_sliding() {
        let result = Source::from(vec![1, 2, 3, 4, 5]).window(3).build();
        assert_eq!(result, vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
    }

    #[test]
    fn window_exact_size() {
        let result = Source::from(vec![1, 2, 3]).window(3).build();
        assert_eq!(result, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn window_larger_than_input() {
        let result = Source::from(vec![1, 2]).window(5).build();
        assert_eq!(result, Vec::<Vec<i32>>::new());
    }

    #[test]
    fn concat_vectors() {
        let result = Source::from(vec![1, 2]).concat(vec![3, 4]).build();
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn concat_empty() {
        let result = Source::from(vec![1, 2]).concat(vec![]).build();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn chained_operations() {
        let result = Source::from(vec![1, 2, 3])
            .flat_map(|x| vec![x, x + 10])
            .chunk(3)
            .flatten()
            .build();
        assert_eq!(result, vec![1, 11, 2, 12, 3, 13]);
    }
}
