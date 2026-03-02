# Cram — Feature Roadmap

This file tracks what needs building. Claude agents work through these in order.
Mark items ✅ when done. Add new items freely.

## Phase 1 — Core Polish (do first)
- [x] Stats view: show total cards, due today, retention rate, streak
- [x] Progress bar during study session (card N of M)
- [x] Keyboard shortcuts: Space=reveal, 1/2/3/4=rate, Esc=back
- [x] Dark/light theme toggle in settings
- [x] Search cards across all decks
- [x] Deck description field visible in deck list
- [ ] Empty state illustration when no decks

## Phase 2 — Power Features
- [ ] Import cards from CSV (front,back format)
- [ ] Export deck to CSV
- [ ] Card tags/categories within a deck
- [ ] Bulk card operations (select all, delete selected)
- [ ] Deck statistics: cards by interval bucket histogram
- [ ] Study session summary screen (cards reviewed, time taken, retention %)
- [ ] Undo last rating

## Phase 3 — Typst Excellence
- [ ] Syntax highlighting in the Typst editor (using egui's code_editor or similar)
- [ ] Auto-reload preview on keystroke with 300ms debounce
- [ ] Error display when Typst fails to compile
- [ ] Full-screen card preview mode
- [ ] Custom Typst preamble per deck (shared imports, macros)

## Phase 4 — Quality
- [ ] Integration tests in it/ directory
- [ ] Benchmark: render throughput (typst compile + rasterise)
- [ ] CI workflow (.github/workflows/ci.yml)
- [ ] Docs: fill out all zensical pages with real content
