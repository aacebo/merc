use std::str::FromStr;

use rust_bert::pipelines::common::ModelType as RustBertModelType;
use serde::{Deserialize, Serialize};

/// Model architecture type for zero-shot classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModelType {
    #[default]
    Bart,
    Bert,
    DistilBert,
    Deberta,
    DebertaV2,
    Roberta,
    XLMRoberta,
    Electra,
    Marian,
    MobileBert,
    T5,
    LongT5,
    Albert,
    XLNet,
    GPT2,
    GPTJ,
    OpenAiGpt,
    Reformer,
    ProphetNet,
    Longformer,
    Pegasus,
    GPTNeo,
    MBart,
    M2M100,
    NLLB,
    FNet,
    Custom(String),
}

impl ModelType {
    /// Returns the string representation of the model type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bart => "bart",
            Self::Bert => "bert",
            Self::DistilBert => "distilbert",
            Self::Deberta => "deberta",
            Self::DebertaV2 => "deberta_v2",
            Self::Roberta => "roberta",
            Self::XLMRoberta => "xlm_roberta",
            Self::Electra => "electra",
            Self::Marian => "marian",
            Self::MobileBert => "mobilebert",
            Self::T5 => "t5",
            Self::LongT5 => "longt5",
            Self::Albert => "albert",
            Self::XLNet => "xlnet",
            Self::GPT2 => "gpt2",
            Self::GPTJ => "gptj",
            Self::OpenAiGpt => "openai_gpt",
            Self::Reformer => "reformer",
            Self::ProphetNet => "prophetnet",
            Self::Longformer => "longformer",
            Self::Pegasus => "pegasus",
            Self::GPTNeo => "gpt_neo",
            Self::MBart => "mbart",
            Self::M2M100 => "m2m100",
            Self::NLLB => "nllb",
            Self::FNet => "fnet",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Returns true if this is an encoder-only model (BERT-like architecture)
    pub fn is_encoder_only(&self) -> bool {
        matches!(
            self,
            Self::Bert
                | Self::DistilBert
                | Self::Deberta
                | Self::DebertaV2
                | Self::Roberta
                | Self::XLMRoberta
                | Self::Electra
                | Self::MobileBert
                | Self::Albert
                | Self::Longformer
                | Self::FNet
        )
    }

    /// Returns true if this is a decoder-only model (GPT-like architecture)
    pub fn is_decoder_only(&self) -> bool {
        matches!(
            self,
            Self::GPT2 | Self::GPTJ | Self::OpenAiGpt | Self::GPTNeo | Self::Reformer
        )
    }

    /// Returns true if this is an encoder-decoder model (seq2seq architecture)
    pub fn is_encoder_decoder(&self) -> bool {
        matches!(
            self,
            Self::Bart
                | Self::MBart
                | Self::T5
                | Self::LongT5
                | Self::Marian
                | Self::Pegasus
                | Self::ProphetNet
                | Self::M2M100
                | Self::NLLB
        )
    }

    /// Returns true if this model is designed for multilingual use
    pub fn is_multilingual(&self) -> bool {
        matches!(
            self,
            Self::XLMRoberta | Self::MBart | Self::M2M100 | Self::NLLB
        )
    }

    /// Returns true if this model is primarily designed for translation
    pub fn is_translation(&self) -> bool {
        matches!(self, Self::Marian | Self::M2M100 | Self::NLLB)
    }

    /// Returns true if this model is primarily designed for summarization
    pub fn is_summarization(&self) -> bool {
        matches!(self, Self::Pegasus | Self::Bart | Self::T5 | Self::LongT5)
    }

    /// Returns true if this model supports zero-shot classification via NLI
    pub fn supports_zero_shot(&self) -> bool {
        matches!(
            self,
            Self::Bart
                | Self::MBart
                | Self::Bert
                | Self::DistilBert
                | Self::Deberta
                | Self::DebertaV2
                | Self::Roberta
                | Self::XLMRoberta
                | Self::Albert
                | Self::Longformer
        )
    }

    /// Returns true if this model supports long context inputs
    pub fn supports_long_context(&self) -> bool {
        matches!(self, Self::Longformer | Self::LongT5 | Self::Reformer)
    }

    /// Returns true if this is a custom/unknown model type
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }
}

impl FromStr for ModelType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "bart" => Self::Bart,
            "bert" => Self::Bert,
            "distilbert" => Self::DistilBert,
            "deberta" => Self::Deberta,
            "deberta_v2" | "debertav2" => Self::DebertaV2,
            "roberta" => Self::Roberta,
            "xlm_roberta" | "xlmroberta" => Self::XLMRoberta,
            "electra" => Self::Electra,
            "marian" => Self::Marian,
            "mobilebert" => Self::MobileBert,
            "t5" => Self::T5,
            "longt5" => Self::LongT5,
            "albert" => Self::Albert,
            "xlnet" => Self::XLNet,
            "gpt2" => Self::GPT2,
            "gptj" | "gpt_j" => Self::GPTJ,
            "openai_gpt" | "openaigpt" => Self::OpenAiGpt,
            "reformer" => Self::Reformer,
            "prophetnet" => Self::ProphetNet,
            "longformer" => Self::Longformer,
            "pegasus" => Self::Pegasus,
            "gpt_neo" | "gptneo" => Self::GPTNeo,
            "mbart" => Self::MBart,
            "m2m100" => Self::M2M100,
            "nllb" => Self::NLLB,
            "fnet" => Self::FNet,
            other => Self::Custom(other.to_string()),
        })
    }
}

impl From<ModelType> for RustBertModelType {
    fn from(model_type: ModelType) -> Self {
        match model_type {
            ModelType::Bart => Self::Bart,
            ModelType::Bert => Self::Bert,
            ModelType::DistilBert => Self::DistilBert,
            ModelType::Deberta => Self::Deberta,
            ModelType::DebertaV2 => Self::DebertaV2,
            ModelType::Roberta => Self::Roberta,
            ModelType::XLMRoberta => Self::XLMRoberta,
            ModelType::Electra => Self::Electra,
            ModelType::Marian => Self::Marian,
            ModelType::MobileBert => Self::MobileBert,
            ModelType::T5 => Self::T5,
            ModelType::LongT5 => Self::LongT5,
            ModelType::Albert => Self::Albert,
            ModelType::XLNet => Self::XLNet,
            ModelType::GPT2 => Self::GPT2,
            ModelType::GPTJ => Self::GPTJ,
            ModelType::OpenAiGpt => Self::OpenAiGpt,
            ModelType::Reformer => Self::Reformer,
            ModelType::ProphetNet => Self::ProphetNet,
            ModelType::Longformer => Self::Longformer,
            ModelType::Pegasus => Self::Pegasus,
            ModelType::GPTNeo => Self::GPTNeo,
            ModelType::MBart => Self::MBart,
            ModelType::M2M100 => Self::M2M100,
            ModelType::NLLB => Self::NLLB,
            ModelType::FNet => Self::FNet,
            ModelType::Custom(_) => Self::Bart, // fallback for custom types
        }
    }
}

impl From<RustBertModelType> for ModelType {
    fn from(model_type: RustBertModelType) -> Self {
        match model_type {
            RustBertModelType::Bart => Self::Bart,
            RustBertModelType::Bert => Self::Bert,
            RustBertModelType::DistilBert => Self::DistilBert,
            RustBertModelType::Deberta => Self::Deberta,
            RustBertModelType::DebertaV2 => Self::DebertaV2,
            RustBertModelType::Roberta => Self::Roberta,
            RustBertModelType::XLMRoberta => Self::XLMRoberta,
            RustBertModelType::Electra => Self::Electra,
            RustBertModelType::Marian => Self::Marian,
            RustBertModelType::MobileBert => Self::MobileBert,
            RustBertModelType::T5 => Self::T5,
            RustBertModelType::LongT5 => Self::LongT5,
            RustBertModelType::Albert => Self::Albert,
            RustBertModelType::XLNet => Self::XLNet,
            RustBertModelType::GPT2 => Self::GPT2,
            RustBertModelType::GPTJ => Self::GPTJ,
            RustBertModelType::OpenAiGpt => Self::OpenAiGpt,
            RustBertModelType::Reformer => Self::Reformer,
            RustBertModelType::ProphetNet => Self::ProphetNet,
            RustBertModelType::Longformer => Self::Longformer,
            RustBertModelType::Pegasus => Self::Pegasus,
            RustBertModelType::GPTNeo => Self::GPTNeo,
            RustBertModelType::MBart => Self::MBart,
            RustBertModelType::M2M100 => Self::M2M100,
            RustBertModelType::NLLB => Self::NLLB,
            RustBertModelType::FNet => Self::FNet,
            #[allow(unreachable_patterns)]
            _ => Self::Bart, // fallback for ONNX or other future variants
        }
    }
}
