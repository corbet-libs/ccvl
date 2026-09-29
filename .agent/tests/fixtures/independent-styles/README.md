# Independent style test fixtures

These source-only fixtures exercise portrait/landscape composition, A4/US Letter
selection, locale defaults, settings validation and portable-renderer parity.
They are not shipped document choices and have no checked-in PDFs or previews.
The user-facing catalogue contains Harvard and Cluster.

Rust tests copy these sources into an isolated temporary workspace through
`test_support::independent_styles`. The original workspace is never registered
with these test styles. Fixture paths retain their workspace-relative structure
so the same native and portable rendering interfaces remain under test.

The fixture wording is synthetic demonstration text, not candidate evidence.
Its existing personal-content license is retained in the root REUSE annotations.
