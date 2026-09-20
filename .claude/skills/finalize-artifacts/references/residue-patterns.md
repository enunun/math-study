# Residue patterns

Each pattern shows a kind of conversation residue and how to rewrite it. Examples are given in English and Japanese; apply the same idea to any language.

Keep a statement when the reader genuinely needs it. The "Keep" notes mark the usual exceptions.

## 1. Addressing the requester

Sentences that answer the person who asked instead of informing the reader.

| Before | After |
| --- | --- |
| Per your request, the proof is written in two parts. | The proof has two parts. |
| ご指示に沿って、解説を短くまとめました。 | (Delete the lead-in and present the short explanation.) |

## 2. Announced constraints

The work was shaped by a rule, and the artifact recites the rule. Apply the rule silently.

| Before | After |
| --- | --- |
| This note avoids calculus, so only algebra is used. | (Delete. Give the algebraic argument.) |
| 難しい記号は使わない方針で書いています。 | (Delete. Use plain wording and define each symbol where it first appears.) |
| No third-party packages are used in this script. | (Delete. The imports already show it.) |

Keep: a requirement the reader must satisfy, such as "Requires Python 3.11 or later."

## 3. Rejected alternatives

Mentions of options that were weighed and dropped.

| Before | After |
| --- | --- |
| A dictionary is used rather than a class, since a class felt too heavy. | Scores are kept in a dictionary. |
| 最初は図で説明する案もありましたが、式のみにしました。 | (Delete. Present the formulas.) |

Keep: a design note that explains a non-obvious choice for future maintainers, written as a standalone reason ("A dictionary is enough: there are only a handful of keys.").

## 4. Revision history

References to earlier drafts or to what changed.

| Before | After |
| --- | --- |
| This version now handles empty input as well. | Empty input is handled. |
| // switched to a for-loop after review | (Delete, or say what the loop does if that is not obvious.) |
| 修正済み: 例題3の答えを訂正しました。 | (Delete. Present the corrected answer.) |

Keep: a real changelog, release notes, or a deprecation notice the reader depends on.

## 5. Meta-commentary about the artifact

Sentences about why a part exists or what the author was aiming for.

| Before | After |
| --- | --- |
| This section exists to make the material approachable for newcomers. | (Delete. Make the section itself approachable.) |
| 理解しやすくするために、補足を追加しています。 | (Delete the lead-in and keep only the supplement.) |
| The reader is assumed to have no prior background. | (Delete. Adjust pacing and definitions instead.) |

## 6. Intent examples that became content

An example given to explain a preference leaks into the deliverable, or the deliverable ends up being about the example.

| Symptom | Fix |
| --- | --- |
| The user illustrated "don't mention tool X" and the note now says tool X is not needed. | Omit tool X. Use the illustration only to learn what to avoid. |
| A sample value typed in a prompt appears in every code sample. | Replace it with a neutral, domain-appropriate value. |

## 7. Process and tool traces

References to how the work was carried out.

| Before | After |
| --- | --- |
| Drafted by an AI assistant from the instructions above. | (Delete unless the user or a policy requires disclosure.) |
| # TODO: confirm with the user | (Resolve it, or delete it.) |
| 調べたところ、次のことが分かりました。 | (State the findings directly.) |

## 8. Defensive or apologetic hedging

Caveats added only to pre-empt feedback from the conversation.

| Before | After |
| --- | --- |
| This may not be quite what you had in mind, but... | (Delete.) |
| 念のため申し添えますが、〜 | (Keep the content only if the reader needs it; drop the lead-in.) |

## Quick test

For any suspicious sentence, ask: "Would this sentence exist if the artifact had been written in one pass by someone who already knew the audience?" If not, rewrite or remove it.
