# Correspondence sources

This is the self-contained Typst facade from cletter, including its subordinate
cgreet, cfarewell, cdate and cink sources. The Rust binary depends on cletter too.
The correspondence libraries own greeting/title parsing, closings, dates,
signature helpers, lowercase locale IDs and explicit orthography conventions.
Harvard owns its subject wording, English greeting overrides and composition.

[Source identities](source.json) record the upstream source version and exact
SHA-256 of every copied file. [The family manifest](vendor/manifest.json) records
the subordinate versions and hashes. A version identifies the source candidate;
registry publication is a separate release gate. Rendering is offline.

Update correspondence behavior in its owning upstream library first. Refresh
cletter's self-contained Typst tree, copy its files here, and update source.json
with hashes of those exact bytes. Keep this README local and retain upstream
licenses. ccvl checks the hashes and excludes vendored sources from its formatter;
it never edits a private copy of the language rules. Re-run correspondence
conformance upstream and ccvl's full render gate after an update.

Orthography conversion is explicit and limited to caller-selected prose.
Diagnostics never mutate input. Preserve names, exact quotes, URLs, source
material and explicit choices; a possible spelling match is not proof of error.
