# ccvl-core

Portable CCVL style contracts, versioned asset bundles and in-memory document rendering.
Styles remain upstream in CCVL; consumers provide application and profile records separately.

Enable the `render` feature to use `render::StyleRenderer`. A bundle selects an exact
style, substyle, locale, page count and paper size. `StyleBundle::with_record` binds
user data to its asset hash. Rendering uses only bundled assets and explicit inputs.

CCVL exports bundles with `export-style` and renders them with `render-style`.
A compatible style contribution changes the bundle, without copying templates into
consumer applications. Consumers must explicitly select a new bundle version.

The core has no workspace database, filesystem adapter, shell runner or chat provider.
Draft validation checks the style contract and selected variant; CCVL's editorial
readiness and application-submission policy remain separate responsibilities.
