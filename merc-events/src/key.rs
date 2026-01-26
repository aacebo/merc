#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    Memory(MemoryAction),
    Facet(FacetAction),
}

impl Key {
    pub fn memory(action: MemoryAction) -> Self {
        Self::Memory(action)
    }

    pub fn facet(action: FacetAction) -> Self {
        Self::Facet(action)
    }

    pub fn exchange(&self) -> &str {
        match self {
            Self::Memory(_) => "memory",
            Self::Facet(_) => "facet",
        }
    }

    pub fn queue(&self) -> &str {
        match self {
            Self::Memory(v) => v.name(),
            Self::Facet(v) => v.name(),
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(v) => write!(f, "memory.{}", v),
            Self::Facet(v) => write!(f, "facet.{}", v),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAction {
    Create,
    Update,
}

impl MemoryAction {
    pub fn name(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
        }
    }
}

impl std::fmt::Display for MemoryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FacetAction {
    Create,
    Update,
}

impl FacetAction {
    pub fn name(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
        }
    }
}

impl std::fmt::Display for FacetAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
