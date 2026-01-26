pub struct ScoreContext<TBase> {
    base: TBase,
}

impl<TBase> From<TBase> for ScoreContext<TBase> {
    fn from(base: TBase) -> Self {
        Self { base }
    }
}

impl<TBase> std::ops::Deref for ScoreContext<TBase> {
    type Target = TBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
