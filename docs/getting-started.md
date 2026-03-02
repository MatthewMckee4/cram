# Getting Started

## Creating your first deck

1. Launch Cram — it opens to the deck list view
2. Click **+ New Deck** and enter a name
3. A sample "Rust Basics" deck is created automatically on first launch

## Adding cards

1. Click **Edit** on any deck to open the card editor
2. Click **+ Add Card** to create a new card
3. Write Typst markup in the **Front** and **Back** fields
4. The preview updates automatically as you type (with 300ms debounce)
5. Click **Save** to persist changes, or **Full Screen** to preview at full size

## Studying

1. Click **Study** on a deck with due cards
2. The front of the card is shown — press **Space** or click **Show Answer** to reveal the back
3. Rate your recall using the buttons or keyboard shortcuts:
   - **1** — Again (complete blackout, reset interval)
   - **2** — Hard (correct with difficulty)
   - **3** — Good (correct with some effort)
   - **4** — Easy (perfect recall)
4. Press **Esc** to return to the deck list at any time
5. After all due cards are reviewed, a session summary shows your retention and time

## Using the preamble

Each deck has an optional **Typst preamble** — shared code prepended to every card when rendering. Use it for common imports, text size settings, or macros:

```typst
#set text(size: 14pt)
#set page(margin: 1em)
```

Open the preamble editor via the collapsible **Deck Preamble** section in the card editor.

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| Space | Reveal answer |
| 1 | Rate: Again |
| 2 | Rate: Hard |
| 3 | Rate: Good |
| 4 | Rate: Easy |
| Esc | Back to deck list |

## Import and export

- **Import CSV:** Click **Import CSV** on a deck, paste CSV lines (one card per line: `front,back`), and click Import
- **Export CSV:** Click **Export CSV** to generate a comma-separated dump of all cards in the deck
