use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::check;
use crate::downstream;
use crate::format;
use crate::measure;
use crate::opportunity;
use crate::public;
use crate::render::{self, Compiler};
use crate::skills;
use crate::stations;
use crate::workspace::Workspace;

#[derive(Debug, Parser)]
#[command(
    name = "ccvl",
    version,
    about = "Deterministic CV and cover-letter compiler"
)]
struct Args {
    #[arg(long, global = true, value_name = "DIRECTORY")]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Verify the self-contained binary and workspace.
    Setup,
    /// Show what the dependency-free setup requires.
    Bootstrap,
    /// Report the embedded runtime and workspace.
    Doctor,
    /// Run every deterministic workspace and document check.
    Check,
    /// Check station coverage and MECE ownership.
    ProfileStatus {
        #[arg(default_value = "interview/stations.toml")]
        plan: PathBuf,
        #[arg(long)]
        verify_sources: bool,
    },
    /// Measure the CVL templates.
    Measure {
        #[arg(long)]
        all: bool,
    },
    /// Measure one keyed opportunity.
    MeasureOpportunity {
        organisation_key: String,
        position_key: String,
        #[arg(long)]
        all: bool,
    },
    /// Run all checks required before publishing.
    PublicCheck,
    /// Verify that a private downstream differs only in explicitly owned paths.
    DownstreamCheck {
        #[arg(long, default_value = "ccvl-downstream.json")]
        policy: PathBuf,
        #[arg(long)]
        upstream_ref: Option<String>,
    },
    /// Build every CVL template and page preset.
    Build,
    /// Create one keyed opportunity without overwriting an existing record.
    NewOpportunity {
        organisation_key: String,
        position_key: String,
        /// Skip the cover letter: writes `generate_cl = false` and no `[cl]`
        /// table, so there is nothing to delete afterwards.
        #[arg(long)]
        no_cover_letter: bool,
    },
    /// Build one CV.
    BuildCv {
        locale: String,
        #[arg(default_value_t = 4)]
        pages: usize,
        #[arg(long)]
        application: Option<PathBuf>,
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Build one cover letter.
    BuildCl {
        locale: String,
        #[arg(long)]
        application: Option<PathBuf>,
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Build one keyed opportunity package.
    BuildOpportunity {
        organisation_key: String,
        position_key: String,
    },
    /// Rebuild one CV whenever its inputs change.
    WatchCv {
        locale: String,
        #[arg(default_value_t = 4)]
        pages: usize,
    },
    /// Rebuild one cover letter whenever its inputs change.
    WatchCl { locale: String },
    /// Rebuild one keyed opportunity (PDFs plus resolved .typ copies)
    /// whenever its template, record, or generated outputs change.
    WatchOpportunity {
        organisation_key: String,
        position_key: String,
    },
    /// Format Typst sources with the embedded formatter.
    Fmt {
        #[arg(long)]
        check: bool,
    },
    /// Run the strict small-model skill evaluation.
    SkillEval {
        #[arg(long, default_value = ".agent/tests/skill-cases.json")]
        cases: PathBuf,
        #[arg(long, default_value = ".agent/skills")]
        skills_root: PathBuf,
        #[arg(long, default_value = ".agent/cache/skill-eval/report.json")]
        output: PathBuf,
        #[arg(long)]
        response_file: Option<PathBuf>,
        #[arg(long)]
        summary: Option<PathBuf>,
        #[arg(long)]
        model: Option<String>,
    },
}

pub fn run() -> Result<ExitCode> {
    let args = Args::parse();
    let workspace = Workspace::discover(args.root.as_deref())?;
    let mut exit_code = ExitCode::SUCCESS;
    match args.command {
        Command::Setup => {
            doctor(&workspace)?;
            check::run(&workspace)?;
            println!("ccvl is ready. No external runtime or fonts are required.");
        }
        Command::Bootstrap => {
            println!("ccvl bootstrap plan");
            println!(
                "platform: {}-{}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            println!("runtime dependencies: none");
            println!("embedded: Typst 0.15.1 | Typstyle 0.15.1 | 16 font files");
        }
        Command::Doctor => doctor(&workspace)?,
        Command::Check => {
            check::run(&workspace)?;
            println!(
                "All data, station, source, skill, font, reproducibility, CV, and cover-letter checks passed."
            );
        }
        Command::ProfileStatus {
            plan,
            verify_sources,
        } => {
            let path = workspace.existing_inside(plan)?;
            let document = stations::load_plan(&workspace, &path)?;
            let assessment = stations::assess(
                &workspace,
                &document,
                &workspace.relative(&path)?.display().to_string(),
            )?;
            if verify_sources && assessment.ready() {
                stations::validate_interview(&workspace, true)?;
            }
            println!("{}", stations::format_report(&workspace, &assessment)?);
            ensure!(assessment.ready(), "station plan is not ready");
        }
        Command::Measure { all } => {
            let failures =
                measure::measure(&workspace, &measure::cvl_specs(&workspace)?, all, true)?;
            ensure!(
                failures.is_empty(),
                "{} line-contract failure(s)",
                failures.len()
            );
        }
        Command::MeasureOpportunity {
            organisation_key,
            position_key,
            all,
        } => {
            let specs = measure::keyed_specs(&workspace, &organisation_key, &position_key)?;
            let failures = measure::measure(&workspace, &specs, all, true)?;
            ensure!(
                failures.is_empty(),
                "{} line-contract failure(s)",
                failures.len()
            );
        }
        Command::PublicCheck => {
            check::run(&workspace)?;
            public::validate_boundary(&workspace)?;
            println!(
                "Public-boundary checks passed. Review .agent/docs/public-identifiers.md before publishing."
            );
        }
        Command::DownstreamCheck {
            policy,
            upstream_ref,
        } => downstream::validate(&workspace, &policy, upstream_ref.as_deref())?,
        Command::Build => print_outputs(render::render_cvl(&workspace)?),
        Command::NewOpportunity {
            organisation_key,
            position_key,
            no_cover_letter,
        } => {
            let path = opportunity::create_record(
                &workspace,
                &organisation_key,
                &position_key,
                !no_cover_letter,
            )?;
            println!("Created {}", workspace.relative(&path)?.display());
        }
        Command::BuildCv {
            locale,
            pages,
            application,
            profile,
            output,
        } => {
            stations::validate_interview(&workspace, true)?;
            let locale = render::normalize_locale(&locale)?;
            let application =
                workspace
                    .existing_inside(application.unwrap_or_else(|| {
                        PathBuf::from(format!("cvl/{locale}/application.toml"))
                    }))?;
            let profile = workspace
                .existing_inside(profile.unwrap_or_else(|| PathBuf::from("cvl/profile.toml")))?;
            let output = if let Some(output) = output {
                output
            } else {
                workspace.path(format!("cvl/{locale}/output/cv-{pages}.pdf"))
            };
            let spec = render::cv_spec(&workspace, locale, pages, &application, &profile, &output)?;
            print_outputs(vec![Compiler::new(&workspace)?.render(&workspace, &spec)?]);
        }
        Command::BuildCl {
            locale,
            application,
            profile,
            output,
        } => {
            let locale = render::normalize_locale(&locale)?;
            let application =
                workspace
                    .existing_inside(application.unwrap_or_else(|| {
                        PathBuf::from(format!("cvl/{locale}/application.toml"))
                    }))?;
            let profile = workspace
                .existing_inside(profile.unwrap_or_else(|| PathBuf::from("cvl/profile.toml")))?;
            let output =
                output.unwrap_or_else(|| workspace.path(format!("cvl/{locale}/output/cl.pdf")));
            let spec = render::cl_spec(&workspace, locale, &application, &profile, &output)?;
            print_outputs(vec![Compiler::new(&workspace)?.render(&workspace, &spec)?]);
        }
        Command::BuildOpportunity {
            organisation_key,
            position_key,
        } => print_outputs(render::render_opportunity(
            &workspace,
            &organisation_key,
            &position_key,
        )?),
        Command::WatchCv { locale, pages } => watch_cv(&workspace, &locale, pages)?,
        Command::WatchCl { locale } => watch_cl(&workspace, &locale)?,
        Command::WatchOpportunity {
            organisation_key,
            position_key,
        } => watch_opportunity(&workspace, &organisation_key, &position_key)?,
        Command::Fmt { check } => format::format_typst(&workspace, check)?,
        Command::SkillEval {
            cases,
            skills_root,
            output,
            response_file,
            summary,
            model,
        } => {
            let cases = resolve(&workspace, &cases);
            let skills_root = resolve(&workspace, &skills_root);
            let output = resolve(&workspace, &output);
            let response_file = response_file
                .as_deref()
                .map(|path| resolve(&workspace, path));
            let summary = summary.as_deref().map(|path| resolve(&workspace, path));
            let model = model
                .or_else(|| std::env::var("GROQ_MODEL").ok())
                .unwrap_or_else(|| skills::DEFAULT_MODEL.to_owned());
            let outcome = skills::run_hosted_evaluation(
                &workspace,
                &cases,
                &skills_root,
                &output,
                response_file.as_deref(),
                &model,
                summary.as_deref(),
            )?;
            println!(
                "Skill evaluation report: {} ({})",
                output.display(),
                outcome.status()
            );
            exit_code = ExitCode::from(outcome.exit_code());
        }
    }
    Ok(exit_code)
}

fn doctor(workspace: &Workspace) -> Result<()> {
    ensure!(
        ctypst::fonts::documents().len() == 16,
        "embedded font set is incomplete"
    );
    println!("ccvl {}", env!("CARGO_PKG_VERSION"));
    println!(
        "platform: {}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("workspace: {}", workspace.root().display());
    println!("Typst engine: embedded 0.15.1");
    println!("Typstyle formatter: embedded 0.15.1");
    println!("fonts: 16 embedded files; system fonts disabled");
    println!("external runtime dependencies: none");
    Ok(())
}

fn watch_cv(workspace: &Workspace, locale: &str, pages: usize) -> Result<()> {
    let locale = render::normalize_locale(locale)?;
    watch_loop(
        &format!("ccvl sources for {locale} {pages}-page CV"),
        || cvl_digest(workspace),
        || {
            let spec = render::cvl_cv_spec(workspace, locale, pages)?;
            Ok(vec![Compiler::new(workspace)?.render(workspace, &spec)?])
        },
    )
}

fn watch_cl(workspace: &Workspace, locale: &str) -> Result<()> {
    let locale = render::normalize_locale(locale)?;
    watch_loop(
        &format!("ccvl sources for {locale} cover letter"),
        || cvl_digest(workspace),
        || {
            let spec = render::cvl_cl_spec(workspace, locale)?;
            Ok(vec![Compiler::new(workspace)?.render(workspace, &spec)?])
        },
    )
}

fn watch_opportunity(workspace: &Workspace, organisation: &str, position: &str) -> Result<()> {
    // Fail fast on an unknown record instead of looping on the error.
    opportunity::record_path(workspace, organisation, position, true)?;
    watch_loop(
        &format!("opportunity sources for {organisation}/{position}"),
        || opportunity_digest(workspace, organisation, position),
        || render::render_opportunity(workspace, organisation, position),
    )
}

fn watch_loop(
    label: &str,
    digest: impl Fn() -> Result<Vec<u8>>,
    render: impl Fn() -> Result<Vec<PathBuf>>,
) -> Result<()> {
    println!("Watching {label}. Press Ctrl-C to stop.");
    let mut previous = Vec::new();
    loop {
        let current = digest()?;
        if current != previous {
            print_outputs(render()?);
            // Re-hash after rendering: built PDFs are excluded from the
            // digest and resolved .typ copies are content-deterministic, so
            // a quiet tree settles instead of rebuilding twice per change.
            previous = digest()?;
        }
        thread::sleep(Duration::from_millis(500));
    }
}

/// General CVL sources: locale templates and records, the shared Typst
/// machinery and assets, plus the workspace contract. Built PDFs are
/// excluded so a render never retriggers itself.
fn cvl_digest(workspace: &Workspace) -> Result<Vec<u8>> {
    digest_roots(&[
        workspace.path("cvl"),
        workspace.path(".agent/typst"),
        workspace.path("ccvl.json"),
    ])
}

/// Opportunity sources: the record's locale templates, the shared Typst
/// machinery and assets, the profile and workspace contract, plus the keyed
/// record directory including its generated output .typ copies.
fn opportunity_digest(
    workspace: &Workspace,
    organisation: &str,
    position: &str,
) -> Result<Vec<u8>> {
    let record = opportunity::record_path(workspace, organisation, position, true)?;
    let directory = record
        .parent()
        .context("opportunity record has no parent")?
        .to_path_buf();
    let mut roots = vec![
        workspace.path(".agent/typst"),
        workspace.path("cvl/assets"),
        workspace.path("cvl/profile.toml"),
        workspace.path("ccvl.json"),
        directory,
    ];
    match render::opportunity_locale(workspace, organisation, position) {
        Ok(locale) => {
            roots.push(workspace.path(format!("cvl/{locale}/cv.typ")));
            roots.push(workspace.path(format!("cvl/{locale}/cl.typ")));
        }
        Err(_) => roots.push(workspace.path("cvl")),
    }
    digest_roots(&roots)
}

fn digest_roots(roots: &[PathBuf]) -> Result<Vec<u8>> {
    let mut paths = Vec::new();
    for root in roots {
        if root.is_file() {
            if is_watched(root) {
                paths.push(root.clone());
            }
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        paths.extend(
            WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| is_watched(entry.path()))
                .map(walkdir::DirEntry::into_path),
        );
    }
    paths.sort();
    let mut digest = Sha256::new();
    for path in paths {
        digest.update(path.to_string_lossy().as_bytes());
        digest.update(fs::read(&path)?);
    }
    Ok(digest.finalize().to_vec())
}

/// Typst-relevant source extensions: templates, TOML records, JSON contracts,
/// and generated customization copies plus raster assets. PDFs stay out so a
/// render never retriggers its own watcher.
fn is_watched(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        extension == "typ" || extension == "toml" || extension == "json" || extension == "png"
    })
}

fn print_outputs(outputs: Vec<PathBuf>) {
    for output in outputs {
        println!("Rendered {}", output.display());
    }
}

fn resolve(workspace: &Workspace, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn watched_workspace() -> (tempfile::TempDir, Workspace) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("cvl/de-ch")).unwrap();
        fs::create_dir_all(root.join(".agent/typst")).unwrap();
        fs::create_dir_all(root.join("opportunities/acme/lead/output")).unwrap();
        fs::write(root.join("ccvl.json"), "{}\n").unwrap();
        fs::write(root.join("cvl/de-ch/cv.typ"), "#let x = 1\n").unwrap();
        fs::write(
            root.join("cvl/de-ch/application.toml"),
            "language = \"en-CH\"\n",
        )
        .unwrap();
        fs::write(root.join(".agent/typst/shared.typ"), "#let y = 2\n").unwrap();
        fs::write(
            root.join("opportunities/acme/lead/application.toml"),
            "language = \"en-CH\"\n",
        )
        .unwrap();
        fs::write(
            root.join("opportunities/acme/lead/output/cv.typ"),
            "#let x = 1\n",
        )
        .unwrap();
        fs::write(root.join("opportunities/acme/lead/output/cv.pdf"), b"%PDF-").unwrap();
        let workspace = Workspace::at(root).unwrap();
        (directory, workspace)
    }

    fn fixture_digest(workspace: &Workspace) -> Vec<u8> {
        digest_roots(&[
            workspace.path("cvl"),
            workspace.path(".agent/typst"),
            workspace.path("ccvl.json"),
            workspace.path("opportunities/acme/lead"),
        ])
        .unwrap()
    }

    #[test]
    fn watched_extensions_cover_templates_records_and_generated_typs() {
        for watched in ["cv.typ", "application.toml", "contract.json", "asset.png"] {
            assert!(is_watched(Path::new(watched)), "{watched} was ignored");
        }
        for ignored in ["cv.pdf", "notes.md", "no-extension"] {
            assert!(!is_watched(Path::new(ignored)), "{ignored} was watched");
        }
    }

    #[test]
    fn digest_reacts_to_templates_records_contracts_and_generated_typs() {
        let (_directory, workspace) = watched_workspace();
        let baseline = fixture_digest(&workspace);
        // A rebuilt PDF alone must not retrigger the watcher.
        fs::write(
            workspace.path("opportunities/acme/lead/output/cv.pdf"),
            b"%PDF-changed",
        )
        .unwrap();
        assert_eq!(fixture_digest(&workspace), baseline);
        // Untouched content hashes stably.
        assert_eq!(fixture_digest(&workspace), baseline);
        for relative in [
            "cvl/de-ch/cv.typ",
            "cvl/de-ch/application.toml",
            ".agent/typst/shared.typ",
            "ccvl.json",
            "opportunities/acme/lead/application.toml",
            "opportunities/acme/lead/output/cv.typ",
        ] {
            let path = workspace.path(relative);
            let before = fs::read(&path).unwrap();
            fs::write(&path, [before.clone(), b"changed\n".to_vec()].concat()).unwrap();
            assert_ne!(
                fixture_digest(&workspace),
                baseline,
                "{relative} was ignored"
            );
            fs::write(&path, before).unwrap();
            assert_eq!(fixture_digest(&workspace), baseline);
        }
    }

    #[test]
    fn invalid_record_keys_are_rejected_before_watching() {
        let (_directory, workspace) = watched_workspace();
        assert!(opportunity::record_path(&workspace, "../acme", "lead", true).is_err());
        assert!(opportunity::record_path(&workspace, "acme", "lead", true).is_ok());
    }
}
