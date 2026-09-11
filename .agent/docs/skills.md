# Skill map

ccvl declares nine canonical skills in `ccvl.json`. Together they
cover the portable application lifecycle without creating another data domain.

| Skill | Owns | Does not own |
|---|---|---|
| `ccvl-install` | environment diagnosis, local bootstrap, verification | profile or document edits |
| `ccvl-profile` | source ingestion, conversational journal, claim states, preferences, station coverage | company or job research |
| `ccvl-style` | independent styles, substyles, explicit layout defaults, rendered examples and style contracts | candidate claims or private research |
| `ccvl-cv` | approved CV selection, persuasive truthful MECE wording, ATS and rendering within an existing style | unsupported claims |
| `ccvl-apply` | a targeted application package or general/open letter, persuasive truthful MECE writing and review coordination | submission authority |
| `ccvl-review` | independent support, persuasion, MECE and rendered-page review with cited findings | draft/evidence edits or submission authority |
| `ccvl-interview` | opportunity-specific preparation and honest practice | general user profile or outcome history |
| `ccvl-upskill` | evidenced gaps, learning priorities, proof plan | automatic enrolment or proficiency claims |
| `ccvl-outcome` | observed events, exact feedback, funnel evidence, calibration | invented causes |

General facts, direction, and preferences learned about the candidate go to
`interview/`. Company and role research exists only inside a concrete
`opportunities/<organisation>/<position>/` package. ccvl does not maintain a
standalone market map.

Profile expansion, behavioural evidence, and writing style are facets of one
verified profile. Opportunity evaluation, research, cover-letter drafting,
application-form preparation belong to one concrete application. Independent
review also supports general CVs and open letters using their declared purpose;
it does not invent a target employer. Job-interview preparation and outcomes stay beside the same
application even though separate skills govern their distinct safeguards.

Country-specific portal scrapers are intentionally not bundled. ccvl accepts a
posting or authorised reference locally; persistent discovery must arrive
through an explicit, reviewed connection. Sending, signing, buying, enrolling,
recording, and portal submission remain separate user-authorised actions.


The author skills share [editorial guidance](editorial.md): for every candidate,
actively embellish framing and narrative within defensible factual meaning,
and organize distinct arguments that cover the material audience/purpose
dimensions. The critic preserves supported persuasion and identifies specific
unsupported implications, overlapping arguments or omitted supported priorities.
AIDA is optional and independent of style geometry; MECE applies with any
recipe. Shared locale and correspondence behavior belongs to cletter/family
public interfaces; skills pass selections and overrides without local rule
tables or duplicate algorithms.

`ccvl-review` uses
[the review protocol](review.md) in fresh context, reading original evidence
and every rendered page. Its CLI freezes inputs and validates findings; callers
supply the model orchestration, and no OS sandbox is claimed. Decision cases
live in `.agent/tests/skill-cases.json`; independent artifact evaluations are
routed from [testing](testing.md).
