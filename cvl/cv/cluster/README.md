# Cluster opening — D+

D+ is the selected typesetting milestone: three categories with **2–3–2
stations**, each with three bullets. The summary is justified; bullets retain
natural word spacing and a ragged right edge. The seven stations and the public
summary deliberately contain **unfinished Lorem Ipsum example content**.
Replace that content before using this opening as a real CV.

The opening uses Archivo and the Harvard hierarchy. Approximately 3.3 mm of
visible space surrounds each category heading; smaller gaps distinguish roles,
bullets and stations. Space recovered around headings is distributed within
entries on the longer continuation pages. Text is never shortened to fit.

| Locale | Opening | Five-page comparison |
| --- | --- | --- |
| de-ch | [D+](d-plus/de/ch/pdf/cv-1.pdf) | [D+ with Harvard continuation](d-plus/de/ch/pdf/comparison-5.pdf) |
| en-ch | [D+](d-plus/en/ch/pdf/cv-1.pdf) | [D+ with Harvard continuation](d-plus/en/ch/pdf/comparison-5.pdf) |

The comparison contains the clustered opening, Harvard D+ pages 2–4, and the
original chronological opening as page 5. The last page is a reference, not an
additional career section. The [Harvard D+ substyle](../harvard/d-plus/README.md)
also works as an ordinary chronological CV without the clustered opening.

```sh
bash ./ccvl build-cv en-ch 1 --style cluster --substyle d-plus
bash .agent/scripts/build-cluster-comparison.sh
```

Earlier geometry samples remain available: `standard` has 2–2–2 stations,
`middle-three` has 2–3–2, and `middle-three-spaced` balances the category gaps.
D+ is the cluster default. The workspace's default CV remains Harvard standard.
Fonts, wording and line counts affect visible gaps: remeasure after changing
them rather than treating these calibrated values as universal constants.
