---
name: translate-book
description: Translation delivery workflow for async-book-cn. Use whenever translating any part of the book (SUMMARY entries, chapter bodies, footnotes, code comments), bilingual side-by-side format, handling of code blocks and TODOs, preserving source typos, and SUMMARY/document title consistency checks.
---

# async-book-cn Translation Workflow

## Delivery Format: Bilingual Side-by-Side

- Each paragraph: English original on top, Chinese translation immediately below, paragraph by paragraph
- Headings are treated the same way: English heading on top, Chinese heading below (e.g., `## 移动语义` follows `## Move semantics`)
- Footnotes (`[^name]:`) are paired one by one
- The user reviews the translation themselves and deletes the English originals once confirmed. **Do not restore English that the user has deleted** — that is a sign of approval, not lost content

## Code Blocks and TODOs

- Code blocks are kept as-is by default and are not translated
- When the user explicitly asks for a code comment to be translated, replace it in place with Chinese, keeping the `//` prefix; do not produce a side-by-side version
- TODO markers stay in English, with a full-width colon: `TODO：xxx`; TODO outline lines in the body are also handled with the bilingual side-by-side format

## Source Typos

- Spelling and grammar errors in the original are preserved as-is and never fixed silently (example: `abrubt` should be `abrupt`)
- After finishing a piece, list these typos in a summary and let the user decide whether to fix them

## SUMMARY.md Consistency

- Entry text in `src/SUMMARY.md` must match the corresponding document's top-level heading exactly
- If the document already has a Chinese title → update SUMMARY to match the document
- If the document title is still English → translate the document title first, then sync SUMMARY

## Untranslated Chapters

- When a chapter body is untranslated as a whole, do not translate its subheadings individually; flag their existence to the user and leave them for the full translation pass
