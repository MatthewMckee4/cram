use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    id: Uuid,
    front: String,
    back: String,
    #[serde(default)]
    tags: Vec<String>,
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

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn front(&self) -> &str {
        &self.front
    }

    pub fn front_mut(&mut self) -> &mut String {
        &mut self.front
    }

    pub fn back(&self) -> &str {
        &self.back
    }

    pub fn back_mut(&mut self) -> &mut String {
        &mut self.back
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn tags_mut(&mut self) -> &mut Vec<String> {
        &mut self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_card_has_unique_id() {
        let a = Card::new("Q", "A");
        let b = Card::new("Q", "A");
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn new_card_stores_front_and_back() {
        let card = Card::new("What is Rust?", "A systems language");
        assert_eq!(card.front(), "What is Rust?");
        assert_eq!(card.back(), "A systems language");
    }

    #[test]
    fn new_card_has_empty_tags() {
        let card = Card::new("Q", "A");
        assert!(card.tags().is_empty());
    }

    #[test]
    fn front_mut_allows_modification() {
        let mut card = Card::new("Q", "A");
        card.front_mut().push_str(" updated");
        assert_eq!(card.front(), "Q updated");
    }

    #[test]
    fn back_mut_allows_modification() {
        let mut card = Card::new("Q", "A");
        card.back_mut().push_str(" updated");
        assert_eq!(card.back(), "A updated");
    }

    #[test]
    fn tags_mut_allows_modification() {
        let mut card = Card::new("Q", "A");
        card.tags_mut().push("math".into());
        assert_eq!(card.tags(), &["math"]);
    }
}
