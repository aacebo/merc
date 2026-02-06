mod attr;
pub mod consumers;
mod emitter;
mod level;
mod otype;
mod span;

pub use attr::*;
pub use emitter::*;
pub use level::*;
pub use otype::*;
pub use span::*;

use loom_core::value::Value;

pub trait Emitter {
    fn emit(&self, signal: Signal);
}

pub trait Consumer {
    fn consume(&self, ty: Type, name: &str, handler: &dyn FnOnce(Signal));
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Signal {
    otype: Type,
    level: Level,
    name: String,
    attributes: Attributes,
    created_at: std::time::SystemTime,
}

impl Signal {
    pub fn new() -> SignalBuilder {
        SignalBuilder::new()
    }

    pub fn otype(&self) -> Type {
        self.otype
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }
}

#[derive(Default)]
pub struct SignalBuilder {
    otype: Option<Type>,
    level: Option<Level>,
    name: Option<String>,
    attributes: AttributesBuilder,
}

impl SignalBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn otype(mut self, otype: Type) -> Self {
        self.otype = Some(otype);
        self
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn attr(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.attributes = self.attributes.attr(key, value);
        self
    }

    pub fn attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = self.attributes.merge(attributes);
        self
    }

    pub fn build(self) -> Signal {
        Signal {
            otype: self.otype.unwrap_or(Type::Event),
            level: self.level.unwrap_or(Level::Info),
            name: self.name.unwrap_or_default(),
            attributes: self.attributes.build(),
            created_at: std::time::SystemTime::now(),
        }
    }
}
