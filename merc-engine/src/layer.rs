use merc_error::Result;

pub trait Layer {
    type In;
    type Out;

    fn invoke(&self, input: &mut Self::In) -> Result<Self::Out>;
}
