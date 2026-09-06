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

# Rebuild one tailored opportunity on every change to its locale template,
# record, or generated output typs.
watch org pos:
    bash ./ccvl watch-opportunity {{org}} {{pos}}

watch-cv locale pages="4":
    bash ./ccvl watch-cv {{locale}} {{pages}}

watch-cl locale:
    bash ./ccvl watch-cl {{locale}}
