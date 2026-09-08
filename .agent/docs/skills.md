# Skill map

ccvl declares eight canonical skills in `ccvl.json`. Together they
cover the portable application lifecycle without creating another data domain.

| Skill | Owns | Does not own |
|---|---|---|
| `ccvl-install` | environment diagnosis, local bootstrap, verification | profile or document edits |
| `ccvl-profile` | source ingestion, conversational journal, claim states, preferences, station coverage | company or job research |
| `ccvl-style` | independent styles, substyles, explicit layout defaults, rendered examples and style contracts | candidate claims or private research |
| `ccvl-cv` | approved CV selection, wording, ATS and rendering within an existing style | unsupported claims |
| `ccvl-apply` | one concrete job, attributable research, fit decision, Summary, letter, review | submission authority |
| `ccvl-interview` | opportunity-specific preparation and honest practice | general user profile or outcome history |
| `ccvl-upskill` | evidenced gaps, learning priorities, proof plan | automatic enrolment or proficiency claims |
| `ccvl-outcome` | observed events, exact feedback, funnel evidence, calibration | invented causes |

General facts, direction, and preferences learned about the candidate go to
`interview/`. Company and role research exists only inside a concrete
`opportunities/<organisation>/<position>/` package. ccvl does not maintain a
standalone market map.

Profile expansion, behavioural evidence, and writing style are facets of one
verified profile. Opportunity evaluation, research, cover-letter drafting,
application-form preparation, and an independent review belong to one concrete
application. Job-interview preparation and outcomes stay beside the same
application even though separate skills govern their distinct safeguards.

Country-specific portal scrapers are intentionally not bundled. ccvl accepts a
posting or authorised reference locally; persistent discovery must arrive
through an explicit, reviewed connection. Sending, signing, buying, enrolling,
recording, and portal submission remain separate user-authorised actions.
