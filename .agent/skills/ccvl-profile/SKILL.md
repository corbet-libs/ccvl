---
name: ccvl-profile
description: Interview or import a candidate into an evidence-backed profile and a full MECE CV station plan before writing the general CVL.
---

# Build a full candidate profile

Collect enough truthful material to fill the CV well. Do not treat a short
first answer or a short existing CV as the complete life record.

Resolve the configured CV style before allocating presentation slots. The page
counts, source paths and station-marker protocol below describe Harvard.
Another style defines its own presentation contract; evidence, truthful scope,
inspectable notes and unique fact ownership apply to every style.
Use [editorial guidance](../../docs/editorial.md) when turning those facts into
application wording: persuasive embellishment and MECE coverage apply to every
candidate, independently of the creator's showcase.

## Protect the user before writing

Inspect the repository's publication destination before storing personal data.
If `origin` is public or its visibility cannot be established, explain that
`cvl/` is publishable and wait for confirmation before placing the new user's
identity or approved career wording there. Keep interview evidence private.
Never require Git knowledge.

Accept three input modes and combine them when useful:

1. read files the user attached or placed in `interview/imports/`;
2. extract a CV or profile pasted into the conversation;
3. conduct the conversational interview below.

Keep source documents read-only. Cross-reference them before resolving dates,
titles, scope, or metrics. Ask only for gaps; do not make the user repeat facts
already present in a source.

At the start, list every attached, pasted, and `interview/imports/` source that
will be used. Read all usable sources before interviewing. If sources conflict,
show the exact alternatives and resolve them one at a time with the user rather
than silently choosing one.

## Initialise visible progress

Before the first interview question:

- create `interview/profile.md` from
  `.agent/scaffolds/interview/profile.md`;
- create `interview/journal.md` from
  `.agent/scaffolds/interview/profile-journal.md`;
- replace the author's example `interview/stations.toml` with the neutral
  `.agent/scaffolds/interview/stations.toml` before recording the new candidate;
- tell the user that the journal is updated continuously and can be inspected
  at any time. They may also paste a document or add one to
  `interview/imports/`.

The profile is an informal, information-rich working portrait. The journal is
an append-only account of questions, answers, extracted facts, conflicts, slot
progress, and deferred prompts. Neither is polished application copy.

After every meaningful answer or imported source, update both files before
asking the next question. Preserve the user's substance and uncertainty. Then
update `interview/stations.toml` and run the platform command:

```text
Linux/macOS: bash ./ccvl profile-status
Windows:     .\ccvl.cmd profile-status
```

Show the user the short page counts periodically, especially after a “nothing
else” answer. Point them to `interview/journal.md` if they seem impatient or
want proof that the conversation is producing work.

## What counts as a station

A station is one visible CV entry with its own heading, context or period, and
supporting content. A bullet is not a station. A compact standalone credential,
award, or school line does not count and must never be added merely to pad the
total.

The deterministic layout contract is:

- page 1, experience: 6–8 stations; target 7; 9 is overcrowded;
- page 2, supporting record: exactly 10 stations with exactly 2 bullets each;
- page 3, projects and initiatives: exactly 10 entries with exactly 2 bullets each;
- page 4, competencies: exactly 3 groups × 3 blocks × 3 keyword lines;
- only verified, assigned stations count.

Underfill and overfill are not successful completion. They prompt another
collection or allocation iteration. If the user explicitly stops, respect
that, preserve all work, and state which page remains underfilled instead of
pretending the CV is ready.

## Conversational collection loop

Ask one main question at a time in the user's language. Start broadly:

> What experiences have you had? You can start with your current or most recent role.

In German:

> Welche Erfahrungen hattest du bisher? Du kannst mit deiner aktuellen oder jüngsten Rolle anfangen.

For each answer:

1. write the raw answer and its source to the journal;
2. identify one or more possible stations without forcing a final section;
3. deepen the most promising item conversationally: period, context,
   responsibility, actions, result or scale, tools, and available evidence;
4. record atomic facts with stable IDs and one owner each;
5. ask “What further experiences have you had?” or its natural equivalent.

Do not ask a 20-question form. Prefer one useful follow-up, persist the answer,
show progress, and continue. The user may answer in fragments, paste a long
history, or provide documents at any point.

If the user first says there is nothing more while page 1 has fewer than six
stations, explain briefly:

> Page 1 looks best with 6–8 substantial experiences; we currently have X. We
> can move on for now, but I will revisit this later with more specific memory
> cues because independent and unpaid work can count truthfully too.

Change topic rather than arguing. Collect page-2 material, then revisit page 1
once with cues grounded in what is already known. Explore, as applicable:

- earlier, temporary, part-time, internship, contract, or freelance roles;
- side ventures, repairs, products, open source, and sustained hobby work;
- research, thesis delivery, teaching, mentoring, and knowledge transfer;
- volunteering, associations, events, committees, leadership, and care or
  other substantial responsibility.

Be assertive about discovering relevant material, not about inventing it.
Substantial independent work may be presented as experience when it shows
real responsibility and output, but its `kind` remains truthful. Never turn it
into employment, a customer engagement, a registered business, adoption, or
revenue without evidence.

Portray every candidate as advantageously as the evidence allows. Look for
ownership, ingenuity, scale, learning speed, and outcomes that a modest user
may omit; choose the strongest accurate frame and plain language. Actively
embellish emphasis and narrative without changing factual meaning or reasonable
reader inference. The user's source phrasing need not be copied literally;
retain meaningful qualifications while removing unnecessary caveats in polished
copy. Preserve the original evidence and journal.

For page 2, ask across education, professional development, credentials,
research, publications, awards, communities, volunteering, and meaningful
personal responsibility. Categories are flexible; the page must still contain
exactly ten stations with two substantive bullets each. Keep collecting while
either a station or one of its bullets lacks verified content.

For page 3, inventory products, projects, initiatives, research deliveries,
community work, and substantial independent builds until ten distinct entries
can each support two verified bullets. A project shown here may draw on the
working profile, but it must not repeat a fact already allocated to pages 1 or
2. If fewer than ten survive MECE allocation, return to specific prompts rather
than shrinking the layout or inventing filler.

For page 4, derive exactly nine mutually exclusive competency blocks from the
verified profile and arrange them as three coherent groups of three. Each block
contains exactly three keyword lines. Keywords may name evidenced knowledge,
tools, methods, and domains, but they must not manufacture employment, ownership,
results, or proficiency that the evidence does not support.

## Allocate economically and MECE

Collect first, allocate second. Many facts can support several narratives, but
each fact ID and each station has exactly one final home. Separate the truthful
nature of an activity (`kind`) from its best presentation section. For example,
a substantial independent project may belong on page 1 under Experience while
remaining `independent-work` or `project`, never employment.

When there are too many candidates, rank them by relevance, evidence strength,
distinctiveness, recency, and demonstrated impact. Merge only facts that form
one coherent station. Move rather than copy. Keep unused material in the
working profile and leave its station unassigned so future opportunities can
reuse it without creating CV duplication.

Unique fact ownership does not prohibit one fact supporting several role
requirements or a concise Summary/highlight signposting fuller evidence. MECE
also requires collective coverage of the material audience/purpose dimensions:
record supported, adjacent and missing evidence during preparation, so repeated
selling points do not hide an unaddressed priority. Missing support never
authorises invented capability or an unsolicited gap apology in the document.

Use `.agent/scaffolds/interview/stations.toml` exactly. Claims are `verified`,
`conflicted`, or `unverified`; explicit user confirmation is valid provenance.
Only verified facts may enter the rendered CV.

## Completion gate

Once allocation is ready, populate `cvl/profile.toml` with the approved public
contact fields using `.agent/scaffolds/interview/profile.toml` as its shape.
Then write the shared entries in `cvl/cv/harvard/src/entries-{de,en}.typ` and the
locale content records below each document substyle in plain
recruiter-readable language while retaining recognised specialist terms.
Place `// ccvl-station: <station-id>` immediately before the `#cv-h[...]` of
every full station on pages 1 and 2. Place `// ccvl-project: <project-id>` before
every project on page 3 and `// ccvl-competency: <competency-id>` before every
competency block on page 4. Do not mark compact standalone lines.
Run:

```text
Linux/macOS: bash ./ccvl profile-status --verify-sources
Windows:     .\ccvl.cmd profile-status --verify-sources
```

That command must confirm the plan is ready and that both CV sources satisfy
the complete fixed layout. Then run the full check. Before publication, show
the exact identifier and claim manifest to the user. The checked-in author's
content is reference-only personal material and must never become evidence or
wording for another person.
