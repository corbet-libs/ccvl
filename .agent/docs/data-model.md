# Workspace data model

Every fact has one owner. The agent may derive approved document wording from
verified evidence, but it must not maintain competing copies of the same
working record.

| Domain | Unit | Contains | Must not contain |
|---|---|---|---|
| `interview/` | candidate | imports, rich profile, journal, evidence, preferences, conflicts, station allocation | tailored application copy |
| `cvl/` | candidate document | approved general CV/CL sources, public render profile, generated showcase | raw evidence or one vacancy's tailoring |
| `opportunities/` | organisation key + position key | posting, company/role research, tailored CV/CL, preparation, submission, outcome | duplicated general profile or a separate market map |

## Candidate evidence

The private working profile in `interview/profile.md` gives every candidate
claim one of three states:

- `verified`: supported by a named source or explicit user confirmation;
- `conflicted`: sources disagree and the conflict remains visible;
- `unverified`: potentially useful, but prohibited from document output.

Absence of evidence is not evidence of absence. Estimates, inferences, hobby
work, and side initiatives retain their real scope.

The profile interview maintains three complementary records:

- `interview/profile.md` is the informal, information-rich portrait and claim
  ledger;
- `interview/journal.md` records inputs, progress, conflicts, preferences, and
  deferred prompts;
- `interview/stations.toml` assigns verified atomic facts to CV stations.

A station has one truthful kind, one final page and section, and one or more
uniquely owned fact IDs. The distinction between kind and placement lets
substantial independent work appear under Experience without becoming a false
employment claim. See [Profile interview and station
allocation](profile-interview.md).

`cvl/profile.toml` is deliberately narrower: it contains approved public
fields needed to render the CV and cover letter. It is not a second rich
profile or evidence store.

## Opportunity identity

The stable identity is its path:

```text
opportunities/<organisation-key>/<position-key>/
```

Keys use lowercase ASCII letters, numbers, hyphens, or underscores. There are
no generic `companies/` or `positions/` levels. The canonical tailored record
is `application.toml`; rendered PDFs belong in the adjacent `pdfs/`
directory and the resolved standalone Typst copies they were built from in
the adjacent `typst/` directory.

## Locale and language codes

Locale and language codes are always lowercase: `de-ch`, never `de-CH`.
This covers `options.language` in every `application.toml`, the locale keys
in `ccvl.json` and `cvl/profile.toml`, the `expected-language` assertions in
the Typst templates, and every locale argument passed to the correspondence
libraries. The correspondence family (`cgreet`, `cfarewell`, `cink`,
`cletter`) keeps canonical BCP 47 table keys internally but resolves inputs
case-insensitively, so lowercase input yields exactly the same output.
Rendered PDFs still carry canonical BCP 47 metadata (`/Lang(de-CH)`) because
the Typst engine canonicalizes the `lang`/`region` text properties itself;
that is engine output, not a code we write.

The record owns:

- posting source and role context;
- attributable organisation research and fit notes;
- language and application date;
- selected CV page count and exactly five tailored Summary lines;
- one CV substyle (`standard`|`compact`) and, when enabled, one cover-letter
  substyle (`left-rule`|`frame`), defaulting to `standard` / `left-rule`
  through `options.cv_substyle` and `options.cl_substyle`;
- whether a cover letter is required;
- when enabled, six measured paragraphs and five measured highlights.

Posting archives and correspondence may sit beside the record but never
duplicate its tailored fields. Interview preparation, submission records, and
outcomes stay in the same job directory without retroactively rewriting what
was submitted.

Broad market research is outside ccvl's storage model. If exploration reveals
a durable fact or preference about the user, record it in `interview/`. If it
concerns a real company or job, create or update that opportunity.
