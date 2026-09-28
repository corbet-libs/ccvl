# Harvard with optically centred headings

This substyle balances visible whitespace around headings while retaining the
standard CV's wording, line breaks and spacing within entries. It supports the
same German and English 2-, 3- and 4-page presets. Harvard standard remains the
default.

Titles move together with their rules inside their allocated layout slots.
The education page adds 0.18 pt on each side of its four headings, or 0.508 mm
in total. These offsets are calibrated for the existing Archivo hierarchy;
changing fonts or title structure requires another visual check.

| Locale | Two pages | Three pages | Four pages |
| --- | --- | --- | --- |
| de-ch | [PDF](de/ch/pdf/cv-2.pdf) | [PDF](de/ch/pdf/cv-3.pdf) | [PDF](de/ch/pdf/cv-4.pdf) |
| en-ch | [PDF](en/ch/pdf/cv-2.pdf) | [PDF](en/ch/pdf/cv-3.pdf) | [PDF](en/ch/pdf/cv-4.pdf) |

```sh
bash ./ccvl build-cv en-ch 4 --style harvard --substyle aligned
```

[D+](../d-plus/README.md) develops this further by redistributing space within
the continuation pages to establish a more consistent rhythm across the CV.
