# Getting Started

## Creating your first deck

1. Launch Cram -- it opens to the deck list view
1. Click **+ New Deck** and enter a name
1. A sample "Rust Basics" deck is created automatically on first launch

## Adding cards

1. Click **Edit** on any deck to open the card editor
1. Click **+ Add Card** to create a new card
1. Write Typst markup in the **Front** and **Back** fields
1. The preview updates automatically as you type (with a 300ms debounce)
1. Click **Save** to persist changes, or **Full Screen** to preview at full size

## Studying

1. Click **Study** on any deck to begin a session
1. Cards are presented in a random order
1. The front of the card is shown -- press **Space** or click **Show Answer** to reveal the back
1. Press **Space** or click **Next** to advance to the next card
1. Press **Esc** to return to the deck list at any time
1. After all cards are reviewed, a session summary shows the number of cards reviewed and the elapsed time

## Using the preamble

Each deck has an optional **Typst preamble** -- shared code prepended to every card when rendering. Use it for common imports, text size settings, or macros:

```typst
#set text(size: 14pt)
#set page(margin: 1em)
```

Open the preamble editor via the collapsible **Deck Preamble** section in the card editor.

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| Space | Show answer / Next card |
| Esc | Back to deck list |
