pub enum Label {
    Sentiment(Sentiment),
    Emotion(Emotion),
    Outcome(Outcome),
    Contextual(Contextual),
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentiment(v) => write!(f, "{}", v),
            Self::Emotion(v) => write!(f, "{}", v),
            Self::Outcome(v) => write!(f, "{}", v),
            Self::Contextual(v) => write!(f, "{}", v),
        }
    }
}

pub enum Sentiment {
    Negative,
    Positive,
}

impl std::fmt::Display for Sentiment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive => write!(f, "positive"),
            Self::Negative => write!(f, "negative"),
        }
    }
}

pub enum Emotion {
    Joy,
    Fear,
    Shame,
    Pride,
    Stress,
    Anger,
    Sad,
}

impl std::fmt::Display for Emotion {
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

pub enum Outcome {
    Success,
    Failure,
    Reward,
    Punishment,
    Decision,
    Response,
}

impl std::fmt::Display for Outcome {
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

pub enum Contextual {
    Fact,
    Time,
    Place,
    Person,
    Social,
}

impl std::fmt::Display for Contextual {
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
