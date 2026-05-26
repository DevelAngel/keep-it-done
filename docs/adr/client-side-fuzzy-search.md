---
status: accepted
date: 2026-05-26
---

# Client-Side Fuzzy Search with Subsequence Matching

## Context and Problem Statement

As the family task list grows beyond ~100 tasks, scrolling through "All Open" or "What I Finished" to find a specific task becomes tedious.
Context filtering narrows by situational tags but cannot answer "Where was that thing about the dentist?" — a free-text recall question.
How should search be implemented given a 500 MB RAM server and ADHD-friendly UX requirements?

Two independent decisions are intertwined:

1. **Where does search run?** — server-side vs. client-side
2. **What kind of matching?** — exact substring, full-text with stemming, subsequence fuzzy, or semantic embeddings

## Decision Drivers

- Server has only 500 MB RAM — additional indexing or ML models are impractical
- Self-hosted privacy — no external API calls for search
- Task titles are short (5–30 chars) and the corpus is small (hundreds, not millions)
- ADHD-friendly: results must appear instantly while typing — no submit-and-wait
- Typo tolerance matters — mobile keyboards produce frequent near-miss input
- German and English mixed usage — stemming is non-trivial across languages

## Considered Options

- Client-side exact substring matching (`str::contains`)
- Client-side subsequence fuzzy matching (`sublime_fuzzy`)
- Server-side full-text index (Tantivy with BM25)
- Server-side semantic search (local embeddings via ONNX)
- API-based semantic search (OpenAI / Ollama embeddings)

## Decision Outcome

Chosen option: "Client-side subsequence fuzzy matching with `sublime_fuzzy`", because it runs entirely in the browser (zero server RAM), tolerates typos and partial input via subsequence matching, and adds only a single zero-dependency crate to the WASM bundle.

Matching algorithm:

- User input is split into whitespace-separated words
- Each word is matched independently against a haystack of `summary + category + contexts` using `sublime_fuzzy::best_match`
- All words must match (AND logic) — same semantics as context filtering
- `sublime_fuzzy` is case-insensitive by default and scores word-start matches, consecutive character runs, and case-exact hits
- No server round-trip — filtering happens after data is already hydrated on the client

Match behaviour examples:

| Query        | Target       | Result | Why                          |
| ------------ | ------------ | ------ | ---------------------------- |
| `milch`      | Milch kaufen | ✅     | Exact subsequence            |
| `mlch`       | Milch kaufen | ✅     | m…l-c-h subsequence          |
| `mkf`        | Milch kaufen | ✅     | M…k…f word-start subsequence |
| `milch haus` | Milch kaufen | ✗      | "haus" not in haystack       |
| `milx`       | Milch kaufen | ✗      | 'x' not present              |

### Consequences

- Good, because zero server memory impact — all computation in the browser
- Good, because instant feedback while typing — no network latency
- Good, because `sublime_fuzzy` is pure Rust with zero dependencies — compiles cleanly to WASM
- Good, because subsequence matching handles mobile-keyboard typos (skipped characters)
- Good, because word splitting enables multi-term refinement ("milch kauf" narrows more than "milch")
- Bad, because subsequence matching does not handle character _substitution_ typos ("Milx" for "Milch") — only skipped characters
- Bad, because no stemming — "Einkäufe" does not match "einkaufen"
- Bad, because no relevance ranking across groups — results keep their original sort order within each category

## Pros and Cons of the Options

### Client-side exact substring matching

- Good, because zero dependencies — `str::contains` is built-in
- Good, because predictable behaviour — users understand "the text must appear"
- Bad, because no typo tolerance at all — "Milc" matches, "mlch" does not
- Bad, because not what users expect when they hear "search"

### Client-side subsequence fuzzy matching (`sublime_fuzzy`)

- Good, because typo-tolerant via character subsequence
- Good, because zero dependencies, pure Rust, WASM-compatible
- Good, because proven algorithm — Sublime Text's search, widely understood
- Neutral, because single-character queries match almost everything — expected fuzzy-finder behaviour
- Bad, because no character substitution tolerance

### Server-side full-text index (Tantivy)

- Good, because proper tokenisation, stemming, BM25 relevance scoring
- Good, because handles "Einkäufe" ↔ "einkaufen" via German stemmer
- Bad, because Tantivy index consumes server RAM — unacceptable on 500 MB
- Bad, because adds network latency to every keystroke (or requires debouncing)
- Bad, because significant crate size and complexity

### Server-side semantic search (local embeddings)

- Good, because true semantic understanding — "Aufgaben für draußen" finds "Rasen mähen"
- Bad, because embedding models require 80–200 MB RAM — more than the task data itself
- Bad, because embedding computation at startup adds seconds of delay
- Bad, because multilingual models (DE + EN) are larger and less accurate on short strings

### API-based semantic search

- Good, because best quality with minimal implementation effort
- Bad, because violates the self-hosted, privacy-first principle
- Bad, because introduces external dependency, latency, and cost
- Bad, because family task titles sent to third parties

## More Information

The `sublime_fuzzy` crate (v0.7.0) implements Sublime Text's scoring algorithm.
Key scoring factors:

- **Word starts** — the `k` in `milch_kaufen` gets a bonus
- **Consecutive matches** — accumulative bonus for adjacent character hits
- **Case match** — exact case match gets a small bonus on top of case-insensitive matching
- **Distance penalty** — gaps between matched characters reduce the score

The crate strips whitespace from patterns internally, which is why we split into words and match each independently rather than passing the full query.
This also gives us AND semantics for free.

Search state is per-view in `HashMap<View, String>`, following the same pattern as context filters.
Currently enabled for All Open and What I Finished — the other views (Upcoming, Quick Wins, Recently Changed) have small enough result sets that search adds little value.

If stemming becomes necessary, Tantivy can be added as a server-side _complement_ — not replacement — by pre-filtering candidates on the server and doing fuzzy refinement on the client.
This layered approach would keep the server index small (only open tasks) while preserving client-side responsiveness.
