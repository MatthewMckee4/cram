use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub front: String,
    pub back: String,
    pub due: chrono::NaiveDate,
    pub interval: f64,
    pub ease: f64,
    pub reps: u32,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Card {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            front: front.into(),
            back: back.into(),
            due: Utc::now().date_naive(),
            interval: 1.0,
            ease: 2.5,
            reps: 0,
            tags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_card_defaults() {
        let card = Card::new("front", "back");
        assert_eq!(card.ease, 2.5);
        assert_eq!(card.interval, 1.0);
        assert_eq!(card.reps, 0);
        assert!(card.tags.is_empty());
    }
}
