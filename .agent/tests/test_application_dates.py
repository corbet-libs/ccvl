#!/usr/bin/env python3
"""Focused source-bound Rust and real Typst dateline regressions, run on CI."""

import argparse
from datetime import datetime, timezone
import hashlib
from importlib.metadata import version
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

import typst


ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / ".agent/tests/fixtures/application-dates.json").read_text())


def run(*command, cwd=None):
    print("+ " + " ".join(map(str, command)), flush=True)
    result = subprocess.run(command, cwd=cwd, text=True, capture_output=True)
    if result.returncode:
        print(result.stdout, flush=True)
        print(result.stderr, flush=True)
        result.check_returncode()
    return result.stdout


def compile_source(source, root, **options):
    return typst.compile(
        source, root=str(root), font_paths=[str(root / ".agent/typst/fonts")],
        timestamp=datetime.fromtimestamp(0, timezone.utc), **options,
    )


def check_rust(output):
    # Compile the actual production module, without pulling in ccvl's unrelated
    # document compiler. This is focused module evidence, not a full runtime build.
    with tempfile.TemporaryDirectory(prefix="ccvl-date-rust-") as directory:
        scratch = Path(directory)
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        dependency = manifest["dependencies"]["cletter"]
        if dependency != "=0.2.0":
            raise ValueError("Review the focused harness when the cletter dependency changes")
        source = ROOT / ".agent/src/application/date.rs"
        (scratch / "Cargo.toml").write_text(
            '[package]\nname="ccvl-date-regression"\nversion="0.0.0"\nedition="2024"\n'
            '[lib]\npath=' + json.dumps(str(source)) + '\n'
            '[dependencies]\nanyhow="1.0"\nregex="1.12"\ncletter="=0.2.0"\n'
            '[dev-dependencies]\nserde_json="1.0"\n'
            '[lints.rust]\nunsafe_code="forbid"\n'
            '[lints.clippy]\nall={level="warn",priority=-1}\n'
            'pedantic={level="warn",priority=-1}\n'
            'missing_errors_doc="allow"\nmissing_panics_doc="allow"\n'
        )
        shutil.copyfile(ROOT / "Cargo.lock", scratch / "Cargo.lock")
        # Resolve once in scratch using already available registry data; retain
        # the result. Cached resolution is not a latest-dependency claim.
        run("cargo", "generate-lockfile", "--offline", cwd=scratch)
        shutil.copyfile(scratch / "Cargo.lock", output / "focused-Cargo.lock")
        print(run("cargo", "test", "--offline", "--locked", cwd=scratch))
        print(run("cargo", "clippy", "--offline", "--locked", "--all-targets", "--", "-D", "warnings", cwd=scratch))
        formatted = output / "date.formatted.rs"
        shutil.copyfile(source, formatted)
        run("rustfmt", "--edition", "2024", str(formatted))
    return {"vectors": len(CASES), "scope": "production date module; cached registry resolution",
            "formatting_matches": source.read_bytes() == formatted.read_bytes()}


def check_typst(output):
    valid = [case for case in CASES if case["expected"] is not None]
    source = b'#import "/.agent/typst/application-date.typ": format-application-date\n'
    source += b'#let cases = json(bytes(sys.inputs.cases))\n'
    source += b'#for case in cases { assert.eq(format-application-date(case.locale, case.input), case.expected) }\n'
    compile_source(source, ROOT, sys_inputs={"cases": json.dumps(valid)})
    rejected = 0
    for case in CASES:
        if case["expected"] is not None:
            continue
        source = b'#import "/.agent/typst/application-date.typ": format-application-date\n'
        source += b'#format-application-date(sys.inputs.locale, sys.inputs.value)\n'
        try:
            compile_source(source, ROOT, sys_inputs={"locale": case["locale"], "value": case["input"]})
        except typst.TypstError as error:
            if "options.application_date" not in str(error) and "options.language" not in str(error):
                raise
            rejected += 1
        else:
            raise AssertionError(f"Typst accepted invalid date: {case!r}")
    return {"accepted": len(valid), "rejected": rejected}


def check_letters(output):
    results = []
    with tempfile.TemporaryDirectory(prefix="ccvl-date-letters-") as directory:
        scratch = Path(directory) / "source"
        shutil.copytree(ROOT, scratch, ignore=shutil.ignore_patterns(".git", "target", "cache", "pdf", "png"))
        for language, expected, other in [
            ("de", "7. Oktober 2026", "October 7, 2026"),
            ("en", "October 7, 2026", "7. Oktober 2026"),
        ]:
            for substyle in ("left-rule", "frame"):
                leaf = scratch / f"cvl/cl/harvard/{substyle}/{language}/ch"
                record = leaf / "content.toml"
                original = record.read_text()
                if original.count('application_date = "2026-09"') != 1:
                    raise ValueError("Unexpected showcase date; review fixture preparation")
                record.write_text(original.replace('application_date = "2026-09"', 'application_date = "2026-10-07"'))
                name = f"{substyle}-{language}-ch"
                pdf = output / f"{name}.pdf"
                pdf.write_bytes(compile_source(str(leaf / "typst/cl.typ"), scratch))
                text = run("pdftotext", "-layout", str(pdf), "-")
                (output / f"{name}.txt").write_text(text)
                if expected not in text or other in text:
                    raise AssertionError(f"Wrong dateline in {name}")
                info = run("pdfinfo", str(pdf))
                pages = next(line.split(":", 1)[1].strip() for line in info.splitlines() if line.startswith("Pages:"))
                if pages != "1":
                    raise AssertionError(f"Unexpected page count in {name}: {pages}")
                run("qpdf", "--check", str(pdf))
                run("pdftoppm", "-singlefile", "-scale-to", "1400", "-png", str(pdf), str(output / name))
                results.append({"name": name, "date": expected, "pages": 1})
                # Direct Typst users receive the same field-specific rejection
                # as CLI validation instead of silently printing English prose.
                record.write_text(original.replace('application_date = "2026-09"', 'application_date = "October 7, 2026"'))
                try:
                    compile_source(str(leaf / "typst/cl.typ"), scratch)
                except typst.TypstError as error:
                    if "options.application_date" not in str(error):
                        raise
                else:
                    raise AssertionError(f"Raw prose bypassed the formatter in {name}")
                record.write_text(original)
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", choices=("all", "rust", "typst", "letters"), default="all")
    arguments = parser.parse_args()
    output = Path(os.environ["CCVL_DATE_ARTIFACT_DIR"])
    if not output.is_absolute():
        raise ValueError("CCVL_DATE_ARTIFACT_DIR must be absolute")
    output.mkdir(parents=True, exist_ok=True)
    receipt = {"schema": 1, "check": arguments.check, "tools": {}}
    for tool in ("rustc", "cargo", "python3", "uv"):
        receipt["tools"][tool] = run(tool, "--version").strip()
    receipt["tools"]["typst"] = version("typst")
    for name, check in (("rust", check_rust), ("typst", check_typst), ("letters", check_letters)):
        if arguments.check in ("all", name):
            receipt[name] = check(output)
    receipt["artifacts"] = {
        path.name: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(output.iterdir()) if path.is_file()
    }
    receipt["status"] = "passed"
    (output / "date-regression.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
