// Back-compat shim: document-style moved to harvard.typ, which also owns
// the shared style registry (default-style, known-styles, load-style).
// Prefer importing from harvard.typ in new templates.
#import "harvard.typ": default-style, document-style, known-styles, load-style
