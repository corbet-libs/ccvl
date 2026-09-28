# ccvl

[![CI](https://github.com/corbet-libs/ccvl/actions/workflows/ci.yml/badge.svg)](https://github.com/corbet-libs/ccvl/actions/workflows/ci.yml)
[![Skill evaluation](https://github.com/corbet-libs/ccvl/actions/workflows/skill-eval.yml/badge.svg)](https://github.com/corbet-libs/ccvl/actions/workflows/skill-eval.yml)
[Verification and release checks](.agent/docs/ci.md)

ccvl is a local-first, forkable CV and application system built as a native
Rust binary with an embedded Typst engine. It ships a real bilingual career
profile as its working example, a controllable Harvard-style presentation
layer, strict document checks, and portable agent skills for the complete
application loop.

The showcase is intentionally a real CV rather than fictional sample data. It
demonstrates the system at production quality and makes its author discoverable
for suitable roles. It is reference-only personal content, not reusable
template wording. New users replace it with their own verified facts.

## Showcase and open application

[View the bilingual CV and cover-letter showcase](.agent/docs/showcase.md).
The personal content is visible for professional evaluation, but is not a
reusable template.

## A simple workspace

The product has four domains:

```text
.agent/                                  implementation and agent workflows
interview/                               knowledge and evidence about the user
cvl/                                     approved general CV and cover letter
opportunities/<organisation>/<position>/ one concrete job and its documents
```

`.github/` contains hosting automation and `LICENSES/` contains the complete
legal texts. There is no separate market map. General preferences discovered
with the user belong in `interview/`; company and role research belongs beside
the concrete job in `opportunities/`.

Included are:

- Harvard German and English CVs with checked two-, three-, and four-page
  variants, a five-line Summary, and a measured target-neutral cover letter;
- [D+ spacing for Harvard](cvl/cv/harvard/d-plus/README.md) and a
  [clustered opening](cvl/cv/cluster/README.md), whose seven example stations
  deliberately remain unfinished Lorem Ipsum;
- two independent demonstration styles with four substyles, portrait and
  landscape compositions, and explicit A4 / US Letter layouts;
- [a rendered gallery and full folder overview](cvl/README.md);
- one validated `application.toml` per concrete opportunity;
- a shared Rust and Typst engine with bundled fonts and reproducible PDF
  output;
- eight agent skills for setup, evidence-backed profiles, style creation, CV work,
  applications, interview preparation, upskilling, and outcome tracking;
- privacy and provenance rules for keeping personal application data in a
  private downstream repository.

## Quick start

On Linux x86_64, no Git or Typst experience is required. [Download and extract the Linux bundle](https://github.com/corbet-libs/ccvl/releases/latest), or
clone the repository if you already use Git. Open the folder in a
filesystem-capable coding agent and ask it to set up ccvl using `AGENTS.md`.
The complete novice and terminal workflows are in [Getting
started](.agent/docs/getting-started.md).

```sh
git clone https://github.com/corbet-libs/ccvl.git
cd ccvl
bash ./ccvl setup
bash ./ccvl check
bash ./ccvl build
bash ./ccvl new-opportunity example-org strategy-lead
# Complete the new application.toml with the ccvl-apply skill, then:
bash ./ccvl build-opportunity example-org strategy-lead
```

Generated general documents are written to
`cvl/cv/<style>/<substyle>/<language>/<region>/pdf/cv-{2,3,4}.pdf` and
`cvl/cl/<style>/<substyle>/<language>/<region>/pdf/cl.pdf`.
The shipped Harvard CV substyles are `standard`, `compact`, `aligned` and `d-plus`; cover-letter substyles are
`left-rule` and `frame`. Use `build-cv en-ch 4 --style harvard --substyle compact` or
`build-cl en-ch --style harvard --substyle frame` with your platform launcher.
Other styles own their layouts and page presets; see [.agent/docs/styles.md](.agent/docs/styles.md).
Opportunity-specific documents are written beside their job record under
`opportunities/<organisation>/<position>/pdfs/` (rendered PDFs) and
`opportunities/<organisation>/<position>/typst/` (resolved standalone
Typst copies).

The Linux x86_64 bundle includes the tested, optimized executable and its
matching workspace. Its ordinary Linux runtime is verified on Ubuntu 24.04
with glibc 2.39. Linux ARM64, macOS and Windows have no verified prebuilt release.
Git, Rust, and an external Typst installation are unnecessary for normal use.

For an existing checkout, setup downloads the checksum-verified runtime that
matches its compiler source. It rejects older binaries, and every launcher
checks the installed runtime again before executing a command. Personal CV,
interview, and opportunity changes do not require recompilation. Developers
can explicitly opt into a locked source build with `bash ./ccvl setup
--from-source` on Linux/macOS or `.\ccvl.cmd setup --from-source` on Windows.
These native developer paths do not establish binary release support on those
other platforms.

The [release contract](.agent/docs/releases.md) describes the required native
builds, shared artifacts, dependency caches, and publication gate.

## Make it yours

The workflow moves in one direction:

```text
interview/ -> cvl/ -> opportunities/<organisation>/<position>/
     ^                         |
     +----- verified facts ----+
```

Start with `ccvl-profile`: import sources or answer one question at a time
while the agent maintains `interview/profile.md`, `interview/journal.md`, and
`interview/stations.toml`. Once the evidence and fixed station layout are
complete, replace the general showcase under `cvl/` with approved wording.
Create one directory for each concrete job; its posting, research, tailored
documents, interview preparation, submission record, and outcome all stay
together.

A private standalone repository may retain ccvl as `upstream`; a public fork
must remove the original author's personal content before publishing its
replacement. See [Private downstreams](.agent/docs/private-downstream.md).

The checked-in showcase is visual and implementation evidence, never evidence
about another user. Claims may be selected and compressed from that user's own
evidence, but must never be invented.

## Agent workflows

Canonical skills live under `.agent/skills/`:

- `ccvl-install`
- `ccvl-profile`
- `ccvl-style`
- `ccvl-cv`
- `ccvl-apply`
- `ccvl-interview`
- `ccvl-upskill`
- `ccvl-outcome`

Repository-wide operating rules are in [AGENTS.md](AGENTS.md). The [data
model](.agent/docs/data-model.md), [testing contract](.agent/docs/testing.md),
and [skill map](.agent/docs/skills.md) document the boundaries in detail.

## License

ccvl uses path-specific licensing:

- software, Typst sources, scripts, and skills: FSL-1.1-ALv2;
- documentation and neutral scaffolds: CC-BY-4.0;
- personal showcase data and Typst content, generated signature, and rendered
  showcase PDFs: LicenseRef-CCVL-Personal-Content;
- bundled fonts: OFL-1.1.

FSL is a Fair Source license, not an OSI Open Source license. Each published
version becomes available under Apache-2.0 two years after publication. See
[LICENSE.md](LICENSE.md) and [REUSE.toml](REUSE.toml) for the exact mapping.
