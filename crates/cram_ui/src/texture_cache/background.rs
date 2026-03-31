use std::collections::HashSet;
use std::sync::mpsc;

use egui::Context;

use super::TextureCache;

const MAX_BACKGROUND_RENDERS: usize = 4;

struct CompletedRender {
    key: String,
    png: Result<Vec<u8>, String>,
}

/// Renders card textures on background threads to avoid blocking the UI.
pub struct BackgroundRenderer {
    sender: mpsc::Sender<CompletedRender>,
    receiver: mpsc::Receiver<CompletedRender>,
    pending: HashSet<String>,
}

impl Default for BackgroundRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundRenderer {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            pending: HashSet::new(),
        }
    }

    /// Queues a render on a background thread. Skipped if already cached,
    /// already pending, or at the concurrency limit.
    pub fn request_render(
        &mut self,
        cache: &TextureCache,
        key: String,
        source: String,
        dark_mode: bool,
        width_pt: Option<f32>,
    ) {
        if cache.contains(&key)
            || self.pending.contains(&key)
            || self.pending.len() >= MAX_BACKGROUND_RENDERS
        {
            return;
        }
        self.pending.insert(key.clone());
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let png = cram_render::render_with_width(&source, dark_mode, width_pt)
                .map_err(|e| e.to_string());
            let _ = sender.send(CompletedRender { key, png });
        });
    }

    /// Loads completed background renders into the texture cache.
    pub fn poll_completed(&mut self, cache: &mut TextureCache, ctx: &Context) {
        while let Ok(render) = self.receiver.try_recv() {
            self.pending.remove(&render.key);
            if let Ok(png) = render.png
                && let Ok(img) = image::load_from_memory(&png)
            {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                let handle = ctx.load_texture(&render.key, ci, egui::TextureOptions::LINEAR);
                cache.insert(render.key, handle);
            }
        }
    }
}
