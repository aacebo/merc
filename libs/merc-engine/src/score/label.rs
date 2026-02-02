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
    pub fn all() -> [Self; 26] {
        [
            Self::Sentiment(SentimentLabel::Positive),
            Self::Sentiment(SentimentLabel::Negative),
            Self::Sentiment(SentimentLabel::Neutral),
            Self::Emotion(EmotionLabel::Joy),
            Self::Emotion(EmotionLabel::Fear),
            Self::Emotion(EmotionLabel::Shame),
            Self::Emotion(EmotionLabel::Pride),
            Self::Emotion(EmotionLabel::Stress),
            Self::Emotion(EmotionLabel::Anger),
            Self::Emotion(EmotionLabel::Sad),
            Self::Outcome(OutcomeLabel::Success),
            Self::Outcome(OutcomeLabel::Failure),
            Self::Outcome(OutcomeLabel::Reward),
            Self::Outcome(OutcomeLabel::Punishment),
            Self::Outcome(OutcomeLabel::Decision),
            Self::Outcome(OutcomeLabel::Progress),
            Self::Outcome(OutcomeLabel::Conflict),
            Self::Context(ContextLabel::Fact),
            Self::Context(ContextLabel::Time),
            Self::Context(ContextLabel::Place),
            Self::Context(ContextLabel::Entity),
            Self::Context(ContextLabel::Phatic),
            Self::Context(ContextLabel::Preference),
            Self::Context(ContextLabel::Plan),
            Self::Context(ContextLabel::Goal),
            Self::Context(ContextLabel::Task),
        ]
    }

    pub fn sentiment() -> [Self; 3] {
        [
            Self::Sentiment(SentimentLabel::Positive),
            Self::Sentiment(SentimentLabel::Negative),
            Self::Sentiment(SentimentLabel::Neutral),
        ]
    }

    pub fn emotion() -> [Self; 7] {
        [
            Self::Emotion(EmotionLabel::Joy),
            Self::Emotion(EmotionLabel::Fear),
            Self::Emotion(EmotionLabel::Shame),
            Self::Emotion(EmotionLabel::Pride),
            Self::Emotion(EmotionLabel::Stress),
            Self::Emotion(EmotionLabel::Anger),
            Self::Emotion(EmotionLabel::Sad),
        ]
    }

    pub fn outcome() -> [Self; 7] {
        [
            Self::Outcome(OutcomeLabel::Success),
            Self::Outcome(OutcomeLabel::Failure),
            Self::Outcome(OutcomeLabel::Reward),
            Self::Outcome(OutcomeLabel::Punishment),
            Self::Outcome(OutcomeLabel::Decision),
            Self::Outcome(OutcomeLabel::Progress),
            Self::Outcome(OutcomeLabel::Conflict),
        ]
    }

    pub fn context() -> [Self; 9] {
        [
            Self::Context(ContextLabel::Fact),
            Self::Context(ContextLabel::Time),
            Self::Context(ContextLabel::Place),
            Self::Context(ContextLabel::Entity),
            Self::Context(ContextLabel::Phatic),
            Self::Context(ContextLabel::Preference),
            Self::Context(ContextLabel::Plan),
            Self::Context(ContextLabel::Goal),
            Self::Context(ContextLabel::Task),
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

    pub fn threshold(&self) -> f32 {
        match self {
            Self::Sentiment(v) => v.threshold(),
            Self::Emotion(v) => v.threshold(),
            Self::Outcome(v) => v.threshold(),
            Self::Context(v) => v.threshold(),
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::Sentiment(v) => v.weight(),
            Self::Emotion(v) => v.weight(),
            Self::Outcome(v) => v.weight(),
            Self::Context(v) => v.weight(),
        }
    }

    /// Platt scaling parameter A for calibration.
    /// P(y|x) = 1 / (1 + exp(-Ax - B))
    pub fn platt_a(&self) -> f32 {
        match self {
            Self::Sentiment(v) => v.platt_a(),
            Self::Emotion(v) => v.platt_a(),
            Self::Outcome(v) => v.platt_a(),
            Self::Context(v) => v.platt_a(),
        }
    }

    /// Platt scaling parameter B for calibration.
    pub fn platt_b(&self) -> f32 {
        match self {
            Self::Sentiment(v) => v.platt_b(),
            Self::Emotion(v) => v.platt_b(),
            Self::Outcome(v) => v.platt_b(),
            Self::Context(v) => v.platt_b(),
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
    Neutral,
    Positive,
}

impl SentimentLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Neutral => "neutral",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Positive => "The speaker is expressing a positive, happy, or optimistic sentiment.",
            Self::Negative => "The speaker is expressing a negative, unhappy, or pessimistic sentiment.",
            Self::Neutral => "The speaker is expressing a neutral or matter-of-fact sentiment without strong emotion.",
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::Negative => 0.35,
            Self::Positive => 0.30,
            Self::Neutral => 0.10,
        }
    }

    pub fn threshold(&self) -> f32 {
        0.70
    }

    /// Platt scaling parameter A (identity: 1.0)
    pub fn platt_a(&self) -> f32 {
        1.0
    }

    /// Platt scaling parameter B (identity: 0.0)
    pub fn platt_b(&self) -> f32 {
        0.0
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
            "neutral" => Ok(Self::Neutral),
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
            Self::Neutral => write!(f, "neutral"),
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
            Self::Joy => "The speaker is feeling joyful, happy, or excited about something.",
            Self::Fear => "The speaker is feeling afraid, anxious, or worried about something.",
            Self::Shame => "The speaker is feeling ashamed, embarrassed, or guilty about something.",
            Self::Pride => "The speaker is feeling proud, accomplished, or satisfied with an achievement.",
            Self::Stress => "The speaker is feeling stressed, overwhelmed, or under pressure.",
            Self::Anger => "The speaker is feeling angry, frustrated, or irritated about something.",
            Self::Sad => "The speaker is feeling sad, upset, or grieving about something.",
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            // Emotion/sentiment (low-medium; mostly metadata)
            Self::Stress => 0.45,
            Self::Fear => 0.40,
            Self::Anger => 0.40,
            Self::Sad => 0.40,
            Self::Shame => 0.35,
            Self::Pride => 0.30,
            Self::Joy => 0.30,
        }
    }

    pub fn threshold(&self) -> f32 {
        0.70
    }

    /// Platt scaling parameter A (identity: 1.0)
    pub fn platt_a(&self) -> f32 {
        1.0
    }

    /// Platt scaling parameter B (identity: 0.0)
    pub fn platt_b(&self) -> f32 {
        0.0
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
    Progress,
    Conflict,
}

impl OutcomeLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Reward => "reward",
            Self::Punishment => "punishment",
            Self::Decision => "decision",
            Self::Progress => "progress",
            Self::Conflict => "conflict",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Success => "The speaker is describing a success, achievement, or accomplishment.",
            Self::Failure => "The speaker is describing a failure, setback, or something that went wrong.",
            Self::Reward => "The speaker is describing receiving a reward, benefit, or positive outcome.",
            Self::Punishment => "The speaker is describing a punishment, penalty, or negative consequence.",
            Self::Decision => "The speaker has made or is announcing a decision or choice.",
            Self::Progress => "The speaker is describing progress, completion, or forward movement on something.",
            Self::Conflict => "The speaker is describing a disagreement, conflict, argument, or interpersonal tension.",
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            // Outcome (medium-high)
            Self::Decision => 0.80,
            Self::Progress => 0.65,
            Self::Conflict => 0.65,
            Self::Success => 0.55,
            Self::Failure => 0.55,
            Self::Reward => 0.45,
            Self::Punishment => 0.45,
        }
    }

    pub fn threshold(&self) -> f32 {
        0.70
    }

    /// Platt scaling parameter A (identity: 1.0)
    pub fn platt_a(&self) -> f32 {
        1.0
    }

    /// Platt scaling parameter B (identity: 0.0)
    pub fn platt_b(&self) -> f32 {
        0.0
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
            "progress" => Ok(Self::Progress),
            "conflict" => Ok(Self::Conflict),
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
            Self::Progress => write!(f, "progress"),
            Self::Conflict => write!(f, "conflict"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ContextLabel {
    Fact,
    Time,
    Place,
    Entity,
    Phatic,
    Preference,
    Plan,
    Goal,
    Task,
}

impl ContextLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Time => "time",
            Self::Place => "place",
            Self::Entity => "entity",
            Self::Phatic => "phatic",
            Self::Preference => "preference",
            Self::Plan => "plan",
            Self::Goal => "goal",
            Self::Task => "task",
        }
    }

    pub fn hypothesis(&self) -> &'static str {
        match self {
            Self::Fact => "The speaker is stating a factual piece of information that should be remembered.",
            Self::Time => "The speaker is mentioning a specific time, date, or deadline.",
            Self::Place => "The speaker is mentioning a specific location, place, or address.",
            Self::Entity => "The speaker is mentioning a specific named person, organization, or entity.",
            Self::Phatic => "This is just social pleasantry, small talk, or acknowledgment with no substantive information.",
            Self::Preference => "The speaker is expressing a personal preference, like, dislike, or opinion.",
            Self::Plan => "The speaker is describing a plan, intention, or commitment to do something.",
            Self::Goal => "The speaker is describing a goal, objective, aspiration, or something they want to achieve.",
            Self::Task => "The speaker is describing something they need to do, remember, or a task to complete.",
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            // Memory-bearing context (high impact)
            Self::Task => 1.00,
            Self::Plan => 0.90,
            Self::Goal => 0.90,
            Self::Preference => 0.85,
            Self::Fact => 0.80,

            // Useful metadata/context (medium)
            Self::Entity => 0.65,
            Self::Time => 0.55,
            Self::Place => 0.55,

            // Phatic should be strong as a detector, but not “memory importance”
            Self::Phatic => 0.40,
        }
    }

    pub fn threshold(&self) -> f32 {
        match self {
            // Special / noisy labels: require higher confidence
            Self::Phatic => 0.80,
            Self::Entity => 0.75,
            // Memory-bearing: allow a bit lower to catch more
            Self::Task => 0.65,
            Self::Plan => 0.65,
            Self::Goal => 0.65,
            Self::Preference => 0.65,
            // Default
            _ => 0.70,
        }
    }

    /// Platt scaling parameter A (identity: 1.0)
    pub fn platt_a(&self) -> f32 {
        1.0
    }

    /// Platt scaling parameter B (identity: 0.0)
    pub fn platt_b(&self) -> f32 {
        0.0
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
            "entity" => Ok(Self::Entity),
            "phatic" => Ok(Self::Phatic),
            "preference" => Ok(Self::Preference),
            "plan" => Ok(Self::Plan),
            "goal" => Ok(Self::Goal),
            "task" => Ok(Self::Task),
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
            Self::Entity => write!(f, "entity"),
            Self::Phatic => write!(f, "phatic"),
            Self::Preference => write!(f, "preference"),
            Self::Plan => write!(f, "plan"),
            Self::Goal => write!(f, "goal"),
            Self::Task => write!(f, "task"),
        }
    }
}
