use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Card {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            front: front.into(),
            back: back.into(),
            tags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_card_has_unique_id() {
        let a = Card::new("Q", "A");
        let b = Card::new("Q", "A");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_card_stores_front_and_back() {
        let card = Card::new("What is Rust?", "A systems language");
        assert_eq!(card.front, "What is Rust?");
        assert_eq!(card.back, "A systems language");
    }

    #[test]
    fn new_card_has_empty_tags() {
        let card = Card::new("Q", "A");
        assert!(card.tags.is_empty());
    }
}
