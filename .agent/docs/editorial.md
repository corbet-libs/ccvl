# Evidence and application writing

For every ccvl user, make the strongest truthful and persuasive case for the
intended audience. These rules apply across styles and candidate backgrounds;
the creator's showcase is one example, never the default candidate profile.
They concern claims and writing, independently of page layout and national
correspondence conventions.

Instruction prose in this repository stays English; other languages appear
only inside quoted exhibits. Per locale exhibits live beside the general
guidance under one general rule: resolve the lowercase BCP 47 language
tag to its primary subtag and load `.agent/docs/examples-<subtag>.md`
when it exists (`de-ch` and `de-li` both resolve to `examples-de.md`).
English examples stay inline and serve as the fallback.

## Three sources, three questions

| Source | Establishes |
|---|---|
| User instructions and stated purpose | The requested document and explicit choices |
| Archived posting and attributable research | The role's requirements and employer context |
| Original candidate evidence or explicit confirmation | What the candidate can claim |

A derived CV cannot independently prove its own assertions. Claim IDs and
`profile-status --verify-sources` establish traceability and structural
consistency; the reviewer still checks whether the source supports the final
wording. Include conflicting relevant evidence, not just favourable excerpts.
A missing source is uncertainty; a source that contradicts the claim may
establish error. Preserve time, attribution, actual scope and estimates.

## Persuasive embellishment without lying

Actively strengthen the presentation: select compelling facts, emphasize their
value, use confident language, build a persuasive narrative and translate real
work into terms the employer understands. Source wording is not a ceiling on
the quality or ambition of the prose. Explicit user confirmations are valid
support; do not demand a separate document for an already confirmed fact.

The boundary is factual meaning and what a reasonable reader would infer.
Do not invent or materially misrepresent credentials, employment, authority,
responsibility, scale, results, numbers or the candidate's contribution. A
stronger description is valid when those implications remain defensible.
For example, a spreadsheet tool that produces reports can be a “reporting
solution”; “led a department-wide reporting transformation” additionally claims
leadership and scale that require support. These examples supply no candidate
facts.

“Modeled approximately 12% savings” cannot become “delivered 12% savings.”
Independent work can show delivery skill without inventing an employer.
Planned learning belongs in preparation until demonstrated or confirmed.
Short skill labels can remain when supported by the evidence base; they do
not imply mastery beyond that evidence.

Preserve qualifications when removing them changes the likely understanding
of what happened, such as projected versus achieved results. Remove defensive
filler, timid hedges and unnecessary caveats when factual meaning is unchanged.
The actor should improve persuasion; the critic should preserve legitimate
strength and identify a specific unsupported implication before calling strong
wording a factual defect. Do not force literal paraphrase or apply blanket bans
on embellishment.

## Select for the reader

A targeted letter answers the most important advertised tasks with supported
experience, results and skills. Then explain useful motivation and employer
relevance; use remaining CV material only if it helps. Transferable evidence
can matter even when its exact terminology is absent from the posting. Relate
it explicitly without pretending an adjacent task was the requested task.

A general CV or open letter follows the declared audience and purpose. Mark
vacancy-specific criteria inapplicable rather than inventing a company need.
The personal public showcase remains reference-only content under its own
license, never reusable candidate evidence.

## MECE argument and coverage

MECE means mutually exclusive and collectively exhaustive. It is a core
content requirement for every style and writing recipe. Plan distinct arguments
that together cover the material dimensions of the audience, purpose and
selected format; completeness does not mean reproducing the whole career.

Before drafting, map each material role priority (or general-purpose dimension)
to supporting evidence and an argument. Mark adjacent evidence, missing support
and inapplicable criteria honestly in preparation notes. An absent CV keyword
alone does not prove a capability gap. Use known support while resolving
material gaps; do not invent qualifications or insert unsolicited gap apologies
to make a coverage map look complete.

Give each paragraph, bullet and highlight a clear job. Combine equivalent
selling points, separate different contributions and check for supported
priorities crowded out by repetition. One fact may support several requirements
without being retold for each one. Preserve one canonical station/fact owner;
a concise summary or highlight may point to fuller evidence with a distinct
signposting role. Assess repetition by meaning and purpose, not matching words.

After drafting, check both directions: what does each block add, and where is
each material dimension addressed or accounted for as a gap? A letter can
develop selected arguments while the CV supplies supporting detail. Within the
AIDA letter contract, MECE checks the distinct contributions and coverage of
its prescribed paragraphs and highlights.

## AIDA paragraph and line contract

In ccvl, AIDA names the complete 26-body-line letter structure implemented by
the Harvard cover-letter contract. Its argument progression, six paragraphs
and line budgets are one contract:

| Stage | Paragraph | Role | Body lines |
|---|---:|---|---:|
| Attention | 1 | Positioning and immediate relevance | 3 |
| Interest | 2 | Primary evidence of relevant ability | 5 |
| Interest | 3 | Complementary evidence | 5 |
| Desire | 4 | Useful contribution supported by the combined evidence | 5 |
| Desire | 5 | Fit with this employer or the declared audience | 5 |
| Action | 6 | Warm close and invitation to talk | 3 |

The five one-line highlights sit between paragraphs 3 and 4, connecting the
evidence to its value for the reader. They are additional to the 26 body lines.
The [AIDA contract](cover-letter.md) defines the measured line, highlight and
page requirements. AIDA is not a separate rhetorical option within that
structure, and a three-paragraph letter is not this AIDA contract. Other
document structures retain their declared contracts; do not silently compress
AIDA or impose it on a document that uses another structure.
The stages are drafting metadata, not printed section labels. Do not flag
ordinary uses of “attention,” “interest,” “desire” or “action” as label leakage.

## Register and locale

Use clear, active, concrete and professional language. Dashes follow a rule in
two steps. The validator always rejects em dashes, ellipses, doubled hyphens,
and dashes used as punctuation, including the spaced parenthetical dash;
rephrase those with a comma, period, or colon. Every remaining hyphen is listed
as a warning with its location: keep necessary German compounds such as
RAG-Systeme or Cloud-Ökonomie, and rephrase the rest. A suspended hyphen
before und, oder, or a lowercase continuation counts as a compound, not
as punctuation. A specific task or
result usually does more work than generic enthusiasm, stacked adjectives or
unexplained management vocabulary. Remove repetition and paragraphs explaining
what that paragraph should contain. Avoid treating every conventional phrase
as an error: a warm short close, supported skill label or user's preferred
wording can be appropriate.

Keep distinctions that carry factual meaning: estimated versus measured,
contributed versus led, planned versus completed, prototype versus production.
An explicit requested disclosure takes precedence over a preference for
brevity. Do not claim these editorial heuristics detect AI authorship.

Author locale identifiers in lowercase, including language and country.
Reusable locale resolution/canonicalization, locale tables, spelling, greetings,
closings, dates and correspondence behavior belong to cletter and its family.
ccvl passes selections and explicit user/style overrides to their public
interfaces and consumes the results. Add missing reusable functionality upstream;
do not maintain competing rules, tables or algorithms in ccvl code or skills.
Apply library orthographic transformations only to appropriate generated prose.
Preserve proper names, exact quotations, URLs and original evidence. When a
safe text boundary cannot be determined, surface the specific issue rather
than globally replacing characters. Locale does not silently select a new paper
size.

## Review standard

Read original sources, final extracted text and every actual rendered page.
Report supported findings under [the review protocol](review.md), separating
errors, uncertainty and optional preferences. A reviewer may return no findings;
there is no objection quota. Readiness requires current artifacts, mechanical
success, complete coverage and no unresolved material factual defect.
MECE findings identify the overlapping arguments or omitted supported priority
and its consequence. A factual wording finding identifies the unsupported reader
inference; strong tone alone is insufficient. Corrections should retain or
improve the strongest defensible case, without adding facts or needless hedges.
