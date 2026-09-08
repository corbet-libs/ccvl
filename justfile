# SPDX-FileCopyrightText: 2026 Julian Y. Richard Corbet
# SPDX-License-Identifier: FSL-1.1-ALv2

# ccvl shortcuts. Every recipe delegates to the checked-in `bash ./ccvl`
# dispatcher (embedded engine only); just adds tab-completion, nothing more.

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

watch-cv locale pages="4" substyle="standard":
    bash ./ccvl watch-cv {{locale}} {{pages}} --substyle {{substyle}}

watch-cl locale substyle="left-rule":
    bash ./ccvl watch-cl {{locale}} --substyle {{substyle}}
