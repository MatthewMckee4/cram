use std::collections::HashMap;
use std::time::Instant;

use egui::{Context, TextureHandle};

const MAX_CACHE_SIZE: usize = 100;

struct CacheEntry {
    texture: TextureHandle,
    last_access: Instant,
}

/// An LRU texture cache that evicts least-recently-used entries when the
/// cache exceeds [`MAX_CACHE_SIZE`].
pub struct TextureCache {
    entries: HashMap<String, CacheEntry>,
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all entries from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Checks whether a key exists in the cache without updating access time.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Looks up a cached texture by key, updating its last-access time.
    pub fn get(&mut self, key: &str) -> Option<TextureHandle> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_access = Instant::now();
            return Some(entry.texture.clone());
        }
        None
    }

    /// Inserts a texture into the cache. If the cache is at capacity, the
    /// least-recently-used entry is evicted first.
    pub fn insert(&mut self, key: String, texture: TextureHandle) {
        self.entries.insert(
            key,
            CacheEntry {
                texture,
                last_access: Instant::now(),
            },
        );
        self.evict_if_needed();
    }

    /// Renders the given Typst source into a texture, returning a cached
    /// version when available. When `width_pt` is `Some`, the Typst page
    /// is constrained to that width (in points) so content wraps to fit.
    pub fn get_or_render_with_width(
        &mut self,
        ctx: &Context,
        key: &str,
        source: &str,
        dark_mode: bool,
        width_pt: Option<f32>,
    ) -> Result<TextureHandle, String> {
        if let Some(handle) = self.get(key) {
            return Ok(handle);
        }
        let png = cram_render::render_with_width(source, dark_mode, width_pt)
            .map_err(|e| e.to_string())?;
        let img = image::load_from_memory(&png).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let handle = ctx.load_texture(key, ci, egui::TextureOptions::LINEAR);
        self.insert(key.to_string(), handle.clone());
        Ok(handle)
    }

    fn evict_if_needed(&mut self) {
        while self.entries.len() > MAX_CACHE_SIZE {
            if let Some(lru_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_access)
                .map(|(key, _)| key.clone())
            {
                self.entries.remove(&lru_key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> Context {
        let ctx = Context::default();
        ctx.memory_mut(|_| {});
        ctx
    }

    fn dummy_texture(ctx: &Context, name: &str) -> TextureHandle {
        let ci = egui::ColorImage::new([1, 1], vec![egui::Color32::RED]);
        ctx.load_texture(name, ci, egui::TextureOptions::NEAREST)
    }

    #[test]
    fn insert_and_get() {
        let ctx = test_ctx();
        let mut cache = TextureCache::new();

        let tex = dummy_texture(&ctx, "a");
        cache.insert("a".to_string(), tex.clone());

        assert_eq!(cache.len(), 1);
        assert!(cache.get("a").is_some());
        assert!(cache.get("missing").is_none());
    }

    #[test]
    fn clear_removes_all() {
        let ctx = test_ctx();
        let mut cache = TextureCache::new();

        for i in 0..5 {
            let key = format!("key-{i}");
            cache.insert(key.clone(), dummy_texture(&ctx, &key));
        }
        assert_eq!(cache.len(), 5);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn evicts_lru_when_over_capacity() {
        let ctx = test_ctx();
        let mut cache = TextureCache::new();

        for i in 0..MAX_CACHE_SIZE {
            let key = format!("key-{i}");
            cache.insert(key.clone(), dummy_texture(&ctx, &key));
        }
        assert_eq!(cache.len(), MAX_CACHE_SIZE);

        cache.get("key-0");

        let extra = dummy_texture(&ctx, "extra");
        cache.insert("extra".to_string(), extra);

        assert_eq!(cache.len(), MAX_CACHE_SIZE);
        assert!(
            cache.get("key-0").is_some(),
            "recently accessed key-0 should survive eviction"
        );
        assert!(
            cache.get("key-1").is_none(),
            "least recently used key-1 should be evicted"
        );
    }

    #[test]
    fn stays_within_capacity_after_many_inserts() {
        let ctx = test_ctx();
        let mut cache = TextureCache::new();

        for i in 0..(MAX_CACHE_SIZE + 50) {
            let key = format!("key-{i}");
            cache.insert(key.clone(), dummy_texture(&ctx, &key));
        }
        assert!(cache.len() <= MAX_CACHE_SIZE);
    }

    #[test]
    fn get_updates_access_time() {
        let ctx = test_ctx();
        let mut cache = TextureCache::new();

        let tex_a = dummy_texture(&ctx, "a");
        cache.insert("a".to_string(), tex_a);

        let tex_b = dummy_texture(&ctx, "b");
        cache.insert("b".to_string(), tex_b);

        cache.get("a");

        for i in 0..MAX_CACHE_SIZE {
            let key = format!("fill-{i}");
            cache.insert(key.clone(), dummy_texture(&ctx, &key));
        }

        assert!(cache.get("b").is_none(), "b should have been evicted");
    }

    #[test]
    fn default_is_empty() {
        let cache = TextureCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn quantize_width_rounds_to_step() {
        use super::super::quantize_width;
        assert_eq!(quantize_width(600.0), 600);
        assert_eq!(quantize_width(624.0), 600);
        assert_eq!(quantize_width(625.0), 650);
        assert_eq!(quantize_width(675.0), 700);
    }

    #[test]
    fn quantize_width_enforces_minimum() {
        use super::super::quantize_width;
        assert_eq!(quantize_width(50.0), 100);
        assert_eq!(quantize_width(10.0), 100);
        assert_eq!(quantize_width(0.0), 100);
    }
}
