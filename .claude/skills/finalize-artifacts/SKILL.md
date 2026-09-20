---
name: finalize-artifacts
description: Finish a deliverable before handing it over. Removes wording that only makes sense to someone who saw the working conversation (requests, constraints, rejected options, revision history) and fixes inconsistencies left by repeated edits. Use after producing or substantially rewriting any artifact such as documents, code comments, config files, commit messages, or PR descriptions.
---

# finalize-artifacts

A deliverable is read by people who never saw the conversation that produced it. This skill turns a working draft into something that reads as if it had been written for those readers from the start.

## When to use

Run it after you create or substantially rewrite an artifact, and before you report the work as done. Typical artifacts:

- Documents, READMEs, guides, notes
- Source code, code comments, docstrings
- Config files and scripts
- Commit messages and PR descriptions

Skip it when nothing was produced (answering a question, explaining code, read-only commands).

## Procedure

1. **Collect the artifacts.** List every file you created or substantially changed in this task, plus any commit message or PR description you drafted. Read each one in full.
2. **Remove conversation residue.** For every sentence that refers to how or why the artifact was made rather than to its subject, decide what it is:
   - *Needed by the reader*: keep it, and phrase it for the reader.
   - *Only shaped the work*: delete it, and let the effect show in structure, wording, and defaults.
   - *Only records the conversation*: delete it.

   See [references/residue-patterns.md](references/residue-patterns.md) for the common kinds and how to rewrite them.
3. **Check consistency.** Repeated rounds of edits leave seams: mixed terminology, shifting tone, orphaned sections, duplicated points. Work through [references/consistency-checklist.md](references/consistency-checklist.md).
4. **Verify nothing was lost.** Confirm that the artifact still does what it was meant to do: facts, code behavior, commands, and requirements the reader depends on must be unchanged. If a fix would alter meaning, leave it and mention it in the final reply.

## Rules

- Edit the files in place. Do not create a second copy.
- Prefer rewriting a sentence to annotating it. Never fix residue by adding an explanation.
- Do not delete content just because it looks like residue. A constraint, caveat, or history note stays if the reader needs it (a changelog entry, a compatibility warning, a documented limitation).
- Constraints given during the work should be honored by the design, not announced. Omit the excluded thing instead of stating that it is excluded.
- Examples the user supplied to explain their intent are guidance about audience, tone, and level of detail. Do not let them become content unless the artifact truly needs them.
- Keep the artifact's language, format, and voice.
- Do not add a changelog of your cleanup to the artifact. In the final reply, mention what you changed in a sentence or two at most, unless the user asks for a full report.

## Done when

Someone opening the artifact cold cannot tell what was asked, what was tried, or what was corrected along the way, and the artifact speaks in one consistent voice.
