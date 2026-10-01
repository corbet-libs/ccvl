# Harvard D+

D+ transfers the cluster study's spacing improvements to the original
chronological CV. Its wording, page order, line breaks, type sizes and five-line
justified summary are retained. Bullets use natural word spacing. D+ is the
default Harvard substyle and the workspace's default CV; `standard` remains
available as an explicit choice.

The chronological opening retains its established entry rhythm. On education,
projects and competencies, headings have more consistent visible outer gaps;
space moves into the gaps between roles, bullets and stations. Optical title
offsets move headings with their rules without changing body line breaks.

| Logical page | Entry adjustment | Bullet adjustment | Other spacing |
| --- | --- | --- | --- |
| 1: chronological opening | inherited | inherited | optically centred headings |
| 2: education | −0.25 mm | +0.14 mm | role +0.14 mm; heading-after −0.24 mm; heading outer +0.20 mm per side versus aligned |
| 3: projects | +0.975 mm | +0.36 mm | large heading outer gaps −3 mm per side |
| 4: competencies | +1.43 mm | +0.46 mm | large heading outer gaps −3 mm per side |

Adjustments are relative to `aligned`, and apply to the renderer's spacing
slots, not directly to ink-to-ink measurements. Logical content pages keep the
same spacing in the 2-, 3- and 4-page presets; automatic pagination never selects
a different set of gaps. These values are calibrated for the shipped text and
Archivo font, so new content still needs measurement and visual review.

| Locale | Two pages | Three pages | Four pages |
| --- | --- | --- | --- |
| de-ch | [PDF](de/ch/pdf/cv-2.pdf) | [PDF](de/ch/pdf/cv-3.pdf) | [PDF](de/ch/pdf/cv-4.pdf) |
| en-ch | [PDF](en/ch/pdf/cv-2.pdf) | [PDF](en/ch/pdf/cv-3.pdf) | [PDF](en/ch/pdf/cv-4.pdf) |

```sh
bash ./ccvl build-cv en-ch 4 --style harvard --substyle d-plus
```
