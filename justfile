# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2

# ccvl shortcuts. Every recipe delegates to the checked-in `bash ./ccvl`
# dispatcher (embedded engine only); just adds tab-completion, nothing more.

set positional-arguments

default:
    @just --list

build:
    bash ./ccvl build

check:
    bash ./ccvl check

measure:
    bash ./ccvl measure

# Rebuild one tailored opportunity on every change to its leaf adapter,
# record, or generated typst copies (PDFs stay out of the hash so a render
# never retriggers itself).
watch org pos:
    bash ./ccvl watch-opportunity {{org}} {{pos}}

# Optional arguments use the same syntax as the CLI, including --style/--substyle.
watch-cv locale *args:
    bash ./ccvl watch-cv "$@"

watch-cl locale *args:
    bash ./ccvl watch-cl "$@"
