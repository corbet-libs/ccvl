# Modern CV style

**Scaffold only.** The design of Modern and its `timeline` substyle has not
been defined yet. The folders and files exist so the engine discovers, builds
and checks the style; the renderer prints only the approved profile header.

| Path | State |
| --- | --- |
| `style.toml` | `de-ch` / `en-ch`, portrait A4, one page, substyles `standard` (default) and `timeline`; `slot-3` to `slot-5` are empty slots |
| `contract.toml`, `scaffold.toml` | No content fields yet |
| `content/<language>/ch/wording.toml` | Empty `[cv]` table |
| `<substyle>/substyle.toml` | No settings yet |
| `layout.typ` | Placeholder renderer: profile header only |
| `../../shared/modern/defaults.toml` | `document-v1` adapter baseline copied from Cluster |

Page count, fonts, paper, content fields and the difference between the
substyles are open decisions.
