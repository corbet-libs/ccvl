# Profile interview and station allocation

The profile interview solves layout underfill by finding truthful material
before the CV is written. Pagination cannot make a sparse career record look
full. The system therefore maintains a private working profile and journal,
then applies a deterministic station gate to the publishable CV plan.

## Working artifacts

| Artifact | Purpose | Publication status |
|---|---|---|
| `interview/imports/` | read-only source-document inbox | private and ignored |
| `interview/profile.md` | informal, source-linked working portrait | private and ignored |
| `interview/journal.md` | visible interview progress and deferred questions | private and ignored |
| `interview/stations.toml` | selected and unassigned station candidates | private working state |
| `cvl/cv/<substyle>/<lang>/ch/` and `cvl/cl/<substyle>/<lang>/ch/` | approved rendered wording and presentation | publishable candidate data |

The journal is written before the next question is asked. A user can inspect it
at any time, paste information directly into it, or provide more documents.

## Fixed layout contract (Harvard CV)

A station is a full visual CV entry with its own heading, context or period,
and supporting content. Individual bullets and compact standalone lines do not
count. The counts below are the Harvard family contract, shared by its
`standard` and `compact` CV substyles — not engine law for a future second
family.

| Page | Role | Fixed structure |
|---|---|---|
| 1 | experience | 6–8 stations; target 7 |
| 2 | supporting record across useful sections | exactly 10 stations × 2 bullets |
| 3 | projects and initiatives | exactly 10 entries × 2 bullets |
| 4 | competencies | exactly 3 groups × 3 blocks × 3 keyword lines |

Only verified stations with a page and one section count. Nine page-1 stations
are overcrowded; fewer than six are underfilled. Every other page count and
listed per-entry line count is exact. A failed bound triggers another interview
or allocation pass.

A selected station uses this neutral shape:

```json
{
  "id": "inventory-automation",
  "label": "Independent inventory automation",
  "anchor": "Independent project | 2024–present",
  "kind": "project",
  "status": "verified",
  "page": 1,
  "section": "experience",
  "experience_eligible": true,
  "facts": [
    { "id": "inventory-time", "text": "Reduced recurring manual inventory work" }
  ],
  "source_refs": ["user-confirmed:YYYY-MM-DD"]
}
```

An unresolved or surplus candidate uses `page: null` and an empty `section`.
It remains available for later tailoring without appearing twice.

## Deterministic state machine

1. **Protect:** determine whether personal data would be published and warn
   before writing to a public destination.
2. **Ingest:** inventory attached, pasted, and `interview/imports/` sources; preserve
   them read-only and cross-reference conflicts.
3. **Capture:** ask one broad question, save the answer, and create candidate
   stations and atomic facts without prematurely choosing sections.
4. **Deepen:** obtain enough detail for each promising station: period,
   context, responsibility, action, result or scale, tools, and provenance.
5. **Measure:** run `ccvl profile-status`; select the next question from the
   underfilled page and unresolved high-value candidates.
6. **Revisit:** treat “nothing else” as exhaustion of one prompt, not proof that
   no other experience exists. Change topic, then return once with specific
   memory cues. Respect an explicit request to stop and report the underfill.
7. **Allocate:** give every station one page and section; give every atomic fact
   one station owner. Rank surplus material and leave it unassigned for future
   tailoring.
8. **Verify:** compare the plan with every locale source in each selected
   substyle, render, measure, and run the complete workspace check.

## Question scheduler

The scheduler starts with “What experiences have you had?” and follows the
user's answer rather than presenting a long questionnaire. After each answer it
chooses one of four actions:

- deepen a promising station that lacks responsibility, result, or provenance;
- ask for another experience while page 1 is below six;
- change to page-2 material when the user is fatigued or page 1 is ready;
- revisit page 1 later with cues drawn from the working profile.

Recall cues cover paid and unpaid roles, internships, temporary work,
independent work, side ventures, repairs, products, open source, sustained
hobbies, research, teaching, mentoring, leadership, volunteering, communities,
events, and substantial personal responsibility. Page-2 prompts cover
education, professional development, credentials, publications, awards, and
other supporting milestones. Page-3 prompts identify ten distinct projects and
initiatives that can support two verified bullets each. Page-4 synthesis groups
evidenced tools, methods, and domains into exactly three MECE groups of three
blocks, each with three keyword lines.

The method is ambitious about finding value and conservative about factual
scope. Independent work may become an Experience station if it demonstrates
real responsibility and output, but its nature remains independent work. It is
never relabelled as employment, customers, formal business activity, adoption,
or revenue without evidence.

## MECE allocation

Facts are collected before presentation buckets are finalised. Each fact has a
stable ID, and duplicate ownership is a validation error. A station records a
heading label, a context-or-period anchor, and its truthful `kind` separately
from its selected `section`, allowing economical
placement without changing what happened. Moving is allowed; copying is not.

Run the gate at any time:

```sh
bash ./ccvl profile-status
bash ./ccvl profile-status --verify-sources
```

The first command reports station coverage and targeted next prompts. The
second proves that every selected locale source contains the planned page-1 and page-2
station IDs, exactly ten two-bullet projects, and the same 3×3 competency
structure. Entries use `// ccvl-station: <station-id>`, `// ccvl-project:
<project-id>`, or `// ccvl-competency: <competency-id>` directly before their
`#cv-h[...]`; compact lines have no marker.
