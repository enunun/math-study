# Consistency checklist

Repeated edits and partial rewrites leave seams. Read the whole artifact once, top to bottom, with these questions in mind. Fix only what is inconsistent; do not restyle parts that already agree with each other.

## Terminology

- Is each concept called by one name throughout? (Not "workspace" in one place and "project folder" in another; not "ユーザー" and "利用者" mixed for the same role.)
- Are abbreviations spelled out on first use and used the same way afterwards?
- Do code identifiers, file names, and commands in the prose match the ones actually used in the code or config?

## Voice and tone

- Is there one register (formal or casual) and one grammatical person throughout? (English: don't switch between "you" and "we"; Japanese: don't mix です・ます with だ・である outside quotations and code comments that follow a different convention.)
- Does the level of detail stay steady? A single step explained in three paragraphs next to steps of one line usually marks a patched-in section.

## Structure

- Do headings describe the content for the reader, and follow a parallel pattern at the same level?
- Are sections in the order a reader needs them? A late addition often sits at the end instead of where it belongs.
- Are there duplicated points, or two sections that answer the same question?
- Are numbered steps, cross-references, and links still correct after insertions and deletions?
- Does the introduction still describe what the artifact now contains?

## Code and config

- Do comments describe the current code rather than an earlier version?
- Are there leftovers: commented-out blocks, unused imports or variables, TODOs that are already resolved, debug output?
- Are naming, formatting, and error-handling style consistent within the file and with neighboring files?

## Commit messages and PR descriptions

- Does the subject describe the change itself, not the task that prompted it?
- Does the body explain what and why for a reviewer who has no access to the conversation?
- Is anything mentioned that is not actually in the diff?

## Final read

Read the artifact once more as a first-time reader. Anything that makes you wonder "why is this here?" or "what was this replacing?" is still a seam.
