use chrono::NaiveDateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::ValidationError;

#[derive(Clone, Serialize, Deserialize)]
pub struct Head {
    pub counter: usize,
    pub timestamp: NaiveDateTime,
}

impl Head {
    pub fn new() -> Self {
        Head {
            counter: 0,
            timestamp: Utc::now().naive_utc(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Flags {
    pub religious: bool,
    pub political: bool,
    pub racist: bool,
    pub sexist: bool,
}

impl Flags {
    pub fn new(religious: bool, political: bool, racist: bool, sexist: bool) -> Self {
        Flags {
            religious,
            political,
            racist,
            sexist,
        }
    }

    pub fn default() -> Self {
        Flags {
            religious: false,
            political: false,
            racist: false,
            sexist: false,
        }
    }

    // Переключатели

    pub fn religious_coup(&mut self) -> Self {
        self.religious = !self.religious;
        return self.clone();
    }

    pub fn political_coup(&mut self) -> Self {
        self.political = !self.political;
        return self.clone();
    }

    pub fn racist_coup(&mut self) -> Self {
        self.racist = !self.racist;
        return self.clone();
    }

    pub fn sexist_coup(&mut self) -> Self {
        self.sexist = !self.sexist;
        return self.clone();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tail {
    pub flags: Flags,

    pub author: String,

    pub tags: Vec<String>,

    #[serde(rename = "language")]
    pub lang: String,
}

pub(crate) fn default_tags() -> Vec<String> {
    return vec![String::from("general")];
}

pub(crate) fn validate_lang(lang: &str) -> Result<(), ValidationError> {
    for l in super::SUPPORTED_LANGUAGES.clone() {
        if lang == l {
            return Ok(());
        }
    }

    return Err(ValidationError::new("custom"));
}

impl Tail {
    pub fn new(flags: Flags, lang: &String, author: String, tags: &Vec<String>) -> Self {
        Tail {
            flags,
            lang: lang.to_string(),
            author,
            tags: tags.to_vec(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Shrimp<B>
where
    B: Serialize,
{
    #[serde(rename = "_id")]
    pub id: String,

    #[serde(rename = "_header")]
    pub head: Head,

    #[serde(flatten)]
    pub body: B,

    #[serde(rename = "_meta-data")]
    pub tail: Tail,
}

pub trait Paws<B> {}

impl<B> Shrimp<B>
where
    B: Serialize,
    B: Paws<B>,
{
    pub fn new(body: B, tail: Tail) -> Self {
        Shrimp {
            id: Uuid::new_v4().to_string(),
            head: Head::new(),
            body,
            tail,
        }
    }
}
