# Getting started

You can use ccvl without knowing Git, Rust, Typst, or package managers. You need
a folder that a coding agent can read and a terminal it can use. Native Linux
x86_64/aarch64, macOS Intel/Apple Silicon, and Windows x86_64/ARM64 are
supported.

## With a coding agent

[Download and extract your platform bundle](https://github.com/corbet-labs/ccvl/releases/latest), open
the extracted folder in Codex or another filesystem-capable coding agent, and
use this prompt:

> Set up this ccvl workspace. Read AGENTS.md, use the ccvl-install skill and the
> checked-in platform dispatcher, verify the complete general CVL, then help me
> build my evidence-backed profile in `interview/` and replace the approved
> document sources in `cvl/`. Keep every concrete job, its research, and its
> CV/CL under `opportunities/`. Interview me until the CV has enough verified
> material for the fixed layout contract, and keep a visible journal as we go.
> Ask me about facts, not tooling, unless the
> harness reaches an unavoidable system-permission decision.

The agent should first run `bash ./ccvl bootstrap` on Linux/macOS or
`.\ccvl.cmd bootstrap` on Windows. That command is read-only and shows the exact
plan. Because the prompt explicitly requests setup, it may then run the matching
`setup` command. Setup:

- detects empty, partial, and already complete environments on the current OS;
- uses the compiled binary included in the platform bundle, or downloads the
  checksum-verified runtime matching an existing source checkout;
- checks the binary's embedded runtime identity before installation and refuses
  stale or unavailable binaries; it never silently installs a compiler;
- keeps the binary below `.agent/cache/ccvl/`; the document compiler,
  formatter, and font pack are embedded;
- runs the full deterministic suite before declaring success.

Re-running setup is safe. If the matching binary is already ready,
it makes no changes and performs the same verification.

## From a terminal

No Git is required. In the extracted folder on Linux or macOS:

```sh
bash ./ccvl bootstrap
bash ./ccvl setup
```

On Windows, use Command Prompt or PowerShell:

```powershell
.\ccvl.cmd bootstrap
.\ccvl.cmd setup
```

`bootstrap` only reports. `setup` is the explicit instruction to install what
the report identifies. For an already managed environment, install the listed
dependencies however you prefer and run:

```sh
bash ./ccvl doctor
bash ./ccvl check
```

Developers who intentionally change compiler source can use `setup --from-source`
to build with stable Rust meeting the `Cargo.toml` minimum. Normal users run the precompiled bundles.

All product commands are available through `bash ./ccvl help` or
`.\ccvl.cmd help`.

## What happens next

Start with `ccvl-profile`. It builds a private, source-linked fact base in
`interview/` before the public general CVL is replaced. You can attach or paste
a CV, drop several sources into `interview/imports/`, or answer one
conversational question at a time. The agent maintains
`interview/journal.md` so you can inspect what has already been captured.

The first CV page needs 6–8 full experience stations. The second needs exactly
10 supporting stations with two bullets each; page 3 needs exactly 10 projects
with two bullets each; page 4 keeps a fixed 3×3 competency structure. `ccvl
profile-status` reports the counts and forces another interview or allocation
pass when the structure is incomplete or overcrowded. The agent must never reuse the showcase author's claims or wording
as yours: they are reference-only personal content, not a template. It must ask
rather than fill an evidence gap with a plausible statement. The complete
algorithm is documented in [Profile interview and station
allocation](profile-interview.md).
