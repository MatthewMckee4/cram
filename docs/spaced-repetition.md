# Spaced Repetition

Cram uses the **SM-2 algorithm** to schedule card reviews. The core idea: review cards just before you'd forget them, spacing reviews further apart as you learn.

## How it works

Each card tracks three values:

- **Interval** — days until the next review (starts at 1)
- **Ease** — a multiplier reflecting how easy the card is (range: 1.3 to 2.5, default: 2.5)
- **Reps** — number of successful reviews

## Rating scale

After seeing the answer, rate your recall:

| Rating | Meaning | Interval effect | Ease effect |
|--------|---------|----------------|-------------|
| 1 — Again | Complete blackout | Reset to 1 day | -0.2 |
| 2 — Hard | Correct with difficulty | Interval x 1.2 | -0.15 |
| 3 — Good | Correct with effort | Interval x ease | No change |
| 4 — Easy | Perfect recall | Interval x ease x 1.3 | +0.1 |

The ease factor is clamped to the range [1.3, 2.5], preventing cards from becoming impossibly hard or trivially easy to schedule.

## Example progression

A new card with ease 2.5:

1. **Day 1:** Review, rate Good -> interval becomes 2.5 days
2. **Day 3:** Review, rate Good -> interval becomes 6.25 days
3. **Day 9:** Review, rate Easy -> interval becomes ~20 days, ease rises to 2.5
4. **Day 29:** Review, rate Hard -> interval becomes ~24 days, ease drops to 2.35

If you rate **Again** at any point, the interval resets to 1 day and you start over.

## Session summary

After completing all due cards in a session, Cram shows:

- **Cards reviewed** — total cards in the session
- **Retention** — percentage rated Good or Easy
- **Time** — elapsed time for the session

## Undo

Made a mistake? Click **Undo Last Rating** during a study session to restore the previous card's scheduling state. This reverts the interval, ease, reps, and due date to their pre-review values.

## Statistics

The **Stats** view shows global metrics:

- Total cards across all decks
- Cards due today
- Cards previously reviewed
- Retention rate (percentage of cards not due)
- Study streak (consecutive days with at least one review)

Per-deck statistics include an **interval histogram** showing how many cards fall into each maturity bucket (New, Learning, Young, Mature, Expert).
