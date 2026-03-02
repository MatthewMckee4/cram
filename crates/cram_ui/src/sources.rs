use std::collections::BTreeMap;
use std::path::PathBuf;

use cram_store::MultiStore;
use cram_store::SourceKind;
use cram_store::git::{self, SyncResult};
use egui::Ui;

use crate::style;

/// Transient per-source status after a sync operation.
pub struct SourceStatus {
    pub path: PathBuf,
    pub message: Option<String>,
}

/// A single source entry with its kind for display purposes.
struct SourceEntry {
    path: PathBuf,
    kind: SourceKind,
}

/// A group of linked sources that share a common git root (or are standalone).
struct SourceGroup {
    git_root: Option<PathBuf>,
    entries: Vec<SourceEntry>,
}

/// Build groups from the flat source list.
/// Sources sharing a git root are merged; non-git sources each get their own group.
fn build_groups(sources: &[(PathBuf, SourceKind)]) -> Vec<SourceGroup> {
    let mut by_root: BTreeMap<PathBuf, Vec<SourceEntry>> = BTreeMap::new();
    let mut standalone: Vec<SourceEntry> = Vec::new();

    for (path, kind) in sources {
        let search_path = match kind {
            SourceKind::Folder => path.clone(),
            SourceKind::File => path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| path.clone()),
        };
        let entry = SourceEntry {
            path: path.clone(),
            kind: *kind,
        };
        if let Some(root) = git::find_git_root(&search_path) {
            by_root.entry(root).or_default().push(entry);
        } else {
            standalone.push(entry);
        }
    }

    let mut groups: Vec<SourceGroup> = by_root
        .into_iter()
        .map(|(root, entries)| SourceGroup {
            git_root: Some(root),
            entries,
        })
        .collect();

    for entry in standalone {
        groups.push(SourceGroup {
            git_root: None,
            entries: vec![entry],
        });
    }

    groups
}

pub struct SourcesView;

impl SourcesView {
    pub fn show(
        ui: &mut Ui,
        multi_store: &mut MultiStore,
        sync_statuses: &mut Vec<SourceStatus>,
        error_message: &mut Option<String>,
    ) {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.heading("Linked Sources");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(style::accent_button("Link Folder")).clicked()
                        && let Some(dir) = rfd::FileDialog::new().pick_folder()
                    {
                        link_source(
                            multi_store,
                            dir,
                            SourceKind::Folder,
                            sync_statuses,
                            error_message,
                        );
                    }
                    if ui.add(style::accent_button("Find Deck")).clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("TOML deck files", &["toml"])
                            .pick_file()
                    {
                        link_source(
                            multi_store,
                            path,
                            SourceKind::File,
                            sync_statuses,
                            error_message,
                        );
                    }
                });
            });
            ui.separator();
            ui.add_space(12.0);

            let sources = multi_store.sources().clone();
            if sources.source.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("No linked sources yet");
                    ui.add_space(8.0);
                    ui.label("Click \"Find Deck\" or \"Link Folder\" to add deck sources.");
                });
                return;
            }

            let source_entries: Vec<(PathBuf, SourceKind)> = sources
                .source
                .iter()
                .map(|s| (s.path.clone(), s.kind))
                .collect();
            let groups = build_groups(&source_entries);
            let mut unlink_path: Option<PathBuf> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for group in &groups {
                    show_group(ui, group, sync_statuses, &mut unlink_path);
                    ui.add_space(8.0);
                }

                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    let has_syncable = source_entries.iter().any(|(p, kind)| {
                        let search = match kind {
                            SourceKind::Folder => p.clone(),
                            SourceKind::File => p
                                .parent()
                                .map(|pp| pp.to_path_buf())
                                .unwrap_or_else(|| p.clone()),
                        };
                        git::find_git_root(&search).is_some()
                    });
                    if has_syncable && ui.add(style::accent_button("Sync All")).clicked() {
                        let results = multi_store.sync_all();
                        for (path, result) in results {
                            let msg = sync_result_message(&result);
                            update_status(sync_statuses, path, msg);
                        }
                    }
                });
            });

            if let Some(path) = unlink_path {
                if let Err(e) = multi_store.unlink(&path) {
                    *error_message = Some(format!("Failed to unlink: {e}"));
                }
                sync_statuses.retain(|s| s.path != path);
            }
        });
    }
}

fn link_source(
    multi_store: &mut MultiStore,
    path: PathBuf,
    kind: SourceKind,
    sync_statuses: &mut Vec<SourceStatus>,
    error_message: &mut Option<String>,
) {
    match multi_store.link(path.clone(), kind) {
        Ok(true) => {
            sync_statuses.clear();
        }
        Ok(false) => {
            *error_message = Some(format!("Already linked: {}", path.display()));
        }
        Err(e) => {
            *error_message = Some(format!("Failed to link: {e}"));
        }
    }
}

fn show_group(
    ui: &mut Ui,
    group: &SourceGroup,
    sync_statuses: &mut Vec<SourceStatus>,
    unlink_path: &mut Option<PathBuf>,
) {
    style::card_frame(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width() - 32.0);
        ui.vertical(|ui| {
            if let Some(root) = &group.git_root {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(shorten_home(root)).strong().size(14.0));
                    ui.label(
                        egui::RichText::new("git repo")
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    let status_msg = sync_statuses
                        .iter()
                        .find(|s| s.path == *root)
                        .and_then(|s| s.message.as_deref());
                    if let Some(msg) = status_msg {
                        ui.label(
                            egui::RichText::new(format!("· {msg}"))
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(style::accent_button("Sync")).clicked() {
                            let result = git::pull(root);
                            let msg = sync_result_message(&result);
                            update_status(sync_statuses, root.clone(), msg);
                        }
                    });
                });
            }

            ui.add_space(4.0);

            for entry in &group.entries {
                show_source_entry(ui, entry, group.git_root.as_deref(), unlink_path);
            }
        });
    });
}

fn show_source_entry(
    ui: &mut Ui,
    entry: &SourceEntry,
    git_root: Option<&std::path::Path>,
    unlink_path: &mut Option<PathBuf>,
) {
    let display_path = match git_root {
        Some(root) => entry
            .path
            .strip_prefix(root)
            .map(|rel| rel.display().to_string())
            .unwrap_or_else(|_| shorten_home(&entry.path)),
        None => shorten_home(&entry.path),
    };

    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(6.0)
        .inner_margin(10.0)
        .outer_margin(egui::Margin::symmetric(0, 2))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width() - 16.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if git_root.is_some() {
                        ui.label(egui::RichText::new(&display_path).strong().size(13.0));
                    } else {
                        ui.label(egui::RichText::new(&display_path).strong().size(14.0));
                        ui.label(
                            egui::RichText::new("not a git repo")
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }

                    match entry.kind {
                        SourceKind::Folder => {
                            let deck_names = list_toml_names(&entry.path);
                            if deck_names.is_empty() {
                                ui.label(
                                    egui::RichText::new("no decks found")
                                        .small()
                                        .color(ui.visuals().weak_text_color()),
                                );
                            } else {
                                show_file_tree(ui, &deck_names);
                            }
                        }
                        SourceKind::File => {
                            ui.label(
                                egui::RichText::new("deck file")
                                    .small()
                                    .color(ui.visuals().weak_text_color()),
                            );
                        }
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(style::destructive_button("Unlink")).clicked() {
                        *unlink_path = Some(entry.path.clone());
                    }
                });
            });
        });
}

/// Render sorted relative paths as a file tree with box-drawing characters.
fn show_file_tree(ui: &mut Ui, names: &[String]) {
    let weak = ui.visuals().weak_text_color();

    // Build tree lines: group by directory prefix for visual nesting.
    let mut prev_parts: Vec<&str> = Vec::new();
    for name in names {
        let parts: Vec<&str> = name.split('/').collect();

        let common = prev_parts
            .iter()
            .zip(parts.iter())
            .take_while(|(a, b)| a == b)
            .count();
        for (depth, dir_name) in parts
            .iter()
            .enumerate()
            .take(parts.len().saturating_sub(1))
            .skip(common)
        {
            let indent = "  ".repeat(depth);
            ui.label(
                egui::RichText::new(format!("{indent}{dir_name}/"))
                    .small()
                    .color(weak),
            );
        }

        let depth = parts.len().saturating_sub(1);
        let indent = "  ".repeat(depth);
        let name_str = name.as_str();
        let leaf = parts.last().unwrap_or(&name_str);
        ui.label(
            egui::RichText::new(format!("{indent}{leaf}"))
                .small()
                .color(weak),
        );
        prev_parts = parts;
    }
}

fn sync_result_message(result: &SyncResult) -> String {
    match result {
        SyncResult::Pulled(msg) => format!("Pulled: {msg}"),
        SyncResult::AlreadyUpToDate => "Already up to date".to_string(),
        SyncResult::NotAGitRepo => "Not a git repo".to_string(),
        SyncResult::Error(e) => format!("Error: {e}"),
    }
}

fn update_status(statuses: &mut Vec<SourceStatus>, path: PathBuf, message: String) {
    if let Some(existing) = statuses.iter_mut().find(|s| s.path == path) {
        existing.message = Some(message);
    } else {
        statuses.push(SourceStatus {
            path,
            message: Some(message),
        });
    }
}

/// Recursively find `.toml` files and return relative paths (without extension) from `root`.
fn list_toml_names(root: &std::path::Path) -> Vec<String> {
    let files = cram_store::find_toml_files(root);
    let mut names: Vec<String> = files
        .iter()
        .filter_map(|p| {
            p.strip_prefix(root)
                .ok()
                .and_then(|rel| rel.with_extension("").to_str().map(String::from))
        })
        .collect();
    names.sort();
    names
}

fn shorten_home(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return format!("~/{}", rest.display());
    }
    path.display().to_string()
}
