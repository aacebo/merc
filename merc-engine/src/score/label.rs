use std::str::FromStr;

use merc_error::Error;

use crate::score::LabelCategory;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Label {
    Sentiment(SentimentLabel),
    Emotion(EmotionLabel),
    Outcome(OutcomeLabel),
    Context(ContextLabel),
}

impl Label {
    pub fn all() -> [Label; 20] {
        [
            Label::Sentiment(SentimentLabel::Positive),
            Label::Sentiment(SentimentLabel::Negative),
            Label::Emotion(EmotionLabel::Joy),
            Label::Emotion(EmotionLabel::Fear),
            Label::Emotion(EmotionLabel::Shame),
            Label::Emotion(EmotionLabel::Pride),
            Label::Emotion(EmotionLabel::Stress),
            Label::Emotion(EmotionLabel::Anger),
            Label::Emotion(EmotionLabel::Sad),
            Label::Outcome(OutcomeLabel::Success),
            Label::Outcome(OutcomeLabel::Failure),
            Label::Outcome(OutcomeLabel::Reward),
            Label::Outcome(OutcomeLabel::Punishment),
            Label::Outcome(OutcomeLabel::Decision),
            Label::Outcome(OutcomeLabel::Response),
            Label::Context(ContextLabel::Fact),
            Label::Context(ContextLabel::Time),
            Label::Context(ContextLabel::Place),
            Label::Context(ContextLabel::Person),
            Label::Context(ContextLabel::Social),
        ]
    }

    pub fn sentiment() -> [Label; 2] {
        [
            Label::Sentiment(SentimentLabel::Positive),
            Label::Sentiment(SentimentLabel::Negative),
        ]
    }

    pub fn emotion() -> [Label; 7] {
        [
            Label::Emotion(EmotionLabel::Joy),
            Label::Emotion(EmotionLabel::Fear),
            Label::Emotion(EmotionLabel::Shame),
            Label::Emotion(EmotionLabel::Pride),
            Label::Emotion(EmotionLabel::Stress),
            Label::Emotion(EmotionLabel::Anger),
            Label::Emotion(EmotionLabel::Sad),
        ]
    }

    pub fn outcome() -> [Label; 6] {
        [
            Label::Outcome(OutcomeLabel::Success),
            Label::Outcome(OutcomeLabel::Failure),
            Label::Outcome(OutcomeLabel::Reward),
            Label::Outcome(OutcomeLabel::Punishment),
            Label::Outcome(OutcomeLabel::Decision),
            Label::Outcome(OutcomeLabel::Response),
        ]
    }

    pub fn context() -> [Label; 5] {
        [
            Label::Context(ContextLabel::Fact),
            Label::Context(ContextLabel::Time),
            Label::Context(ContextLabel::Place),
            Label::Context(ContextLabel::Person),
            Label::Context(ContextLabel::Social),
        ]
    }

    pub fn category(&self) -> LabelCategory {
        match self {
            Self::Sentiment(_) => LabelCategory::Sentiment,
            Self::Emotion(_) => LabelCategory::Emotion,
            Self::Outcome(_) => LabelCategory::Outcome,
            Self::Context(_) => LabelCategory::Context,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sentiment(v) => v.as_str(),
            Self::Emotion(v) => v.as_str(),
            Self::Outcome(v) => v.as_str(),
            Self::Context(v) => v.as_str(),
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Sentiment(v) => v.hypothesis(),
            Self::Emotion(v) => v.hypothesis(),
            Self::Outcome(v) => v.hypothesis(),
            Self::Context(v) => v.hypothesis(),
        }
    }
}

impl FromStr for Label {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = SentimentLabel::from_str(s) {
            return Ok(Self::Sentiment(v));
        }

        if let Ok(v) = EmotionLabel::from_str(s) {
            return Ok(Self::Emotion(v));
        }

        if let Ok(v) = OutcomeLabel::from_str(s) {
            return Ok(Self::Outcome(v));
        }

        if let Ok(v) = ContextLabel::from_str(s) {
            return Ok(Self::Context(v));
        }

        Err(Error::builder()
            .message(&format!("'{}' is not a valid label", s))
            .build())
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentiment(v) => write!(f, "{}", v),
            Self::Emotion(v) => write!(f, "{}", v),
            Self::Outcome(v) => write!(f, "{}", v),
            Self::Context(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SentimentLabel {
    Negative,
    Positive,
}

impl SentimentLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Positive => "This text expresses a positive sentiment.",
            Self::Negative => "This text expresses a negative sentiment.",
        }
    }
}

impl From<SentimentLabel> for Label {
    fn from(value: SentimentLabel) -> Self {
        Self::Sentiment(value)
    }
}

impl FromStr for SentimentLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "positive" => Ok(Self::Positive),
            "negative" => Ok(Self::Negative),
            v => Err(Error::builder()
                .message(&format!("'{}' is not a valid sentiment label", v))
                .build()),
        }
    }
}

impl std::fmt::Display for SentimentLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive => write!(f, "positive"),
            Self::Negative => write!(f, "negative"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EmotionLabel {
    Joy,
    Fear,
    Shame,
    Pride,
    Stress,
    Anger,
    Sad,
}

impl EmotionLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Joy => "joy",
            Self::Fear => "fear",
            Self::Shame => "shame",
            Self::Pride => "pride",
            Self::Stress => "stress",
            Self::Anger => "anger",
            Self::Sad => "sad",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Joy => "This text expresses joy or happiness.",
            Self::Fear => "This text expresses fear or anxiety.",
            Self::Shame => "This text expresses shame or embarrassment.",
            Self::Pride => "This text expresses pride or accomplishment.",
            Self::Stress => "This text expresses stress or pressure.",
            Self::Anger => "This text expresses anger or frustration.",
            Self::Sad => "This text expresses sadness or grief.",
        }
    }
}

impl From<EmotionLabel> for Label {
    fn from(value: EmotionLabel) -> Self {
        Self::Emotion(value)
    }
}

impl FromStr for EmotionLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "joy" => Ok(Self::Joy),
            "fear" => Ok(Self::Fear),
            "shame" => Ok(Self::Shame),
            "pride" => Ok(Self::Pride),
            "stress" => Ok(Self::Stress),
            "anger" => Ok(Self::Anger),
            "sad" => Ok(Self::Sad),
            v => Err(Error::builder()
                .message(&format!("'{}' is not a valid emotion label", v))
                .build()),
        }
    }
}

impl std::fmt::Display for EmotionLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Joy => write!(f, "joy"),
            Self::Fear => write!(f, "fear"),
            Self::Shame => write!(f, "shame"),
            Self::Pride => write!(f, "pride"),
            Self::Stress => write!(f, "stress"),
            Self::Anger => write!(f, "anger"),
            Self::Sad => write!(f, "sad"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OutcomeLabel {
    Success,
    Failure,
    Reward,
    Punishment,
    Decision,
    Response,
}

impl OutcomeLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Reward => "reward",
            Self::Punishment => "punishment",
            Self::Decision => "decision",
            Self::Response => "response",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Success => "This text describes achieving a goal or success.",
            Self::Failure => "This text describes a failure or setback.",
            Self::Reward => "This text describes receiving a reward or benefit.",
            Self::Punishment => "This text describes a punishment or consequence.",
            Self::Decision => "This text describes making a decision or choice.",
            Self::Response => "This text describes a response to a prior action.",
        }
    }
}

impl From<OutcomeLabel> for Label {
    fn from(value: OutcomeLabel) -> Self {
        Self::Outcome(value)
    }
}

impl FromStr for OutcomeLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            "reward" => Ok(Self::Reward),
            "punishment" => Ok(Self::Punishment),
            "decision" => Ok(Self::Decision),
            "response" => Ok(Self::Response),
            v => Err(Error::builder()
                .message(&format!("'{}' is not a valid outcome label", v))
                .build()),
        }
    }
}

impl std::fmt::Display for OutcomeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Reward => write!(f, "reward"),
            Self::Punishment => write!(f, "punishment"),
            Self::Decision => write!(f, "decision"),
            Self::Response => write!(f, "response"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ContextLabel {
    Fact,
    Time,
    Place,
    Person,
    Social,
}

impl ContextLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Time => "time",
            Self::Place => "place",
            Self::Person => "person",
            Self::Social => "social",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Fact => "This text states a factual piece of information.",
            Self::Time => "This text references a specific time or date.",
            Self::Place => "This text references a specific location or place.",
            Self::Person => "This text contains information about a specific named person.",
            Self::Social => "This text describes a relationship or social dynamic.",
        }
    }
}

impl From<ContextLabel> for Label {
    fn from(value: ContextLabel) -> Self {
        Self::Context(value)
    }
}

impl FromStr for ContextLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fact" => Ok(Self::Fact),
            "time" => Ok(Self::Time),
            "place" => Ok(Self::Place),
            "person" => Ok(Self::Person),
            "social" => Ok(Self::Social),
            v => Err(Error::builder()
                .message(&format!("'{}' is not a valid context label", v))
                .build()),
        }
    }
}

impl std::fmt::Display for ContextLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fact => write!(f, "fact"),
            Self::Time => write!(f, "time"),
            Self::Place => write!(f, "place"),
            Self::Person => write!(f, "person"),
            Self::Social => write!(f, "social"),
        }
    }
}
