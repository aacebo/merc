use std::rc::Rc;

use merc_error::Result;

use crate::{Context, Map, Value};

pub trait Layer {
    fn invoke(&self, ctx: &Context) -> Result<LayerResult>;
}

#[derive(Debug, Clone)]
pub struct LayerResult {
    pub meta: Map,

    data: Rc<dyn Value>,
}

impl LayerResult {
    pub fn new<T: Value>(data: T) -> Self {
        Self {
            meta: Map::default(),
            data: Rc::new(data),
        }
    }

    pub fn data<T: Value>(&self) -> &T {
        (self.data.as_ref()).as_any().downcast_ref().unwrap()
    }
}
