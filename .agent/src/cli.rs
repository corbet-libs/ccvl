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
    /// Print the compiled runtime source identity without opening a workspace.
    RuntimeId,
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
    PublicCheck {
        /// Retain freshly verified PDFs in a new directory.
        #[arg(long)]
        artifacts: Option<PathBuf>,
    },
    /// Verify that a private downstream differs only in explicitly owned paths.
    DownstreamCheck {
        #[arg(long, default_value = "ccvl-downstream.json")]
        policy: PathBuf,
        #[arg(long)]
        upstream_ref: Option<String>,
    },
    /// Build every CVL template and page preset.
    Build,
    /// List every registered document variant and its output as JSON.
    ListDocuments,
    /// Explain merged adapter settings and the source of each value as JSON.
    ExplainStyle {
        #[arg(value_parser = ["cv", "cl"])]
        document: String,
        locale: String,
        #[arg(long)]
        style: Option<String>,
        #[arg(long)]
        substyle: Option<String>,
    },
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
        pages: Option<usize>,
        /// CV substyle (one entry of cvl/cv/harvard/style.toml). Defaults to the
        /// record's selection, then to the family default.
        #[arg(long)]
        style: Option<String>,
        #[arg(long)]
        substyle: Option<String>,
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
        pages: Option<usize>,
        /// Cover-letter substyle (one entry of cvl/cl/harvard/style.toml). Defaults
        /// to the record's selection, then to the family default.
        #[arg(long)]
        style: Option<String>,
        #[arg(long)]
        substyle: Option<String>,
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
        pages: Option<usize>,
        #[arg(long)]
        style: Option<String>,
        #[arg(long)]
        substyle: Option<String>,
    },
    /// Rebuild one cover letter whenever its inputs change.
    WatchCl {
        locale: String,
        #[arg(long)]
        pages: Option<usize>,
        #[arg(long)]
        style: Option<String>,
        #[arg(long)]
        substyle: Option<String>,
    },
    /// Rebuild one keyed opportunity (PDFs plus resolved .typ copies)
    /// whenever its template, record, or generated typst outputs change.
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
    if matches!(args.command, Command::RuntimeId) {
        println!("{}", crate::runtime::ID);
        return Ok(ExitCode::SUCCESS);
    }
    let workspace = Workspace::discover(args.root.as_deref())?;
    crate::runtime::verify(workspace.root())?;
    let mut exit_code = ExitCode::SUCCESS;
    match args.command {
        Command::RuntimeId => unreachable!("handled before workspace discovery"),
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
        Command::PublicCheck { artifacts } => {
            check::run_with_artifacts(&workspace, artifacts.as_deref())?;
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
        Command::ListDocuments => println!(
            "{}",
            serde_json::to_string_pretty(&render::list_documents(&workspace)?)?
        ),
        Command::ExplainStyle {
            document,
            locale,
            style,
            substyle,
        } => {
            let document = if document == "cv" { "cv" } else { "cl" };
            let selected = crate::styles::selection(
                &workspace,
                document,
                style.as_deref(),
                substyle.as_deref(),
            )?;
            let leaf = crate::styles::leaf(&workspace, document, &locale, &selected)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&crate::settings::resolve(&workspace, &leaf)?)?
            );
        }
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
            style,
            substyle,
            application,
            profile,
            output,
        } => {
            let selection = cli_selection(
                &workspace,
                style.as_deref(),
                substyle.as_deref(),
                application.as_deref(),
                "cv",
            )?;
            let leaf = render::cv_leaf(&workspace, &locale, &selection)?;
            let application =
                workspace.existing_inside(application.unwrap_or_else(|| leaf.content()))?;
            let record = workspace.read_toml_value(workspace.relative(&application)?)?;
            let pages = pages
                .or(record
                    .pointer("/options/pages")
                    .and_then(serde_json::Value::as_u64)
                    .map(usize::try_from)
                    .transpose()?)
                .unwrap_or(leaf.default_pages);
            let profile = workspace
                .existing_inside(profile.unwrap_or_else(|| PathBuf::from("cvl/profile.toml")))?;
            let output = output.unwrap_or_else(|| leaf.output(pages));
            let spec = render::cv_spec(
                &workspace,
                &leaf.locale,
                pages,
                &application,
                &profile,
                &output,
                &selection,
            )?;
            print_outputs(vec![Compiler::new(&workspace)?.render(&workspace, &spec)?]);
        }
        Command::BuildCl {
            locale,
            pages,
            style,
            substyle,
            application,
            profile,
            output,
        } => {
            let selection = cli_selection(
                &workspace,
                style.as_deref(),
                substyle.as_deref(),
                application.as_deref(),
                "cl",
            )?;
            let leaf = render::cl_leaf(&workspace, &locale, &selection)?;
            let application =
                workspace.existing_inside(application.unwrap_or_else(|| leaf.content()))?;
            let record = workspace.read_toml_value(workspace.relative(&application)?)?;
            let pages = pages
                .or(record
                    .pointer("/options/cl_pages")
                    .and_then(serde_json::Value::as_u64)
                    .map(usize::try_from)
                    .transpose()?)
                .unwrap_or(leaf.default_pages);
            let profile = workspace
                .existing_inside(profile.unwrap_or_else(|| PathBuf::from("cvl/profile.toml")))?;
            let output = output.unwrap_or_else(|| leaf.output(pages));
            let spec = render::cl_spec(
                &workspace,
                &leaf.locale,
                pages,
                &application,
                &profile,
                &output,
                &selection,
            )?;
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
        Command::WatchCv {
            locale,
            pages,
            style,
            substyle,
        } => {
            watch_cv(
                &workspace,
                &locale,
                pages,
                style.as_deref(),
                substyle.as_deref(),
            )?;
        }
        Command::WatchCl {
            locale,
            pages,
            style,
            substyle,
        } => {
            watch_cl(
                &workspace,
                &locale,
                pages,
                style.as_deref(),
                substyle.as_deref(),
            )?;
        }
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

/// Explicit `--substyle` wins; otherwise the record at `application`
/// selects; otherwise the family default renders.
fn cli_selection(
    workspace: &Workspace,
    style: Option<&str>,
    substyle: Option<&str>,
    application: Option<&Path>,
    document: &str,
) -> Result<crate::styles::Selection> {
    let record = application
        .map(|path| {
            let path = workspace.existing_inside(path)?;
            let location = workspace.relative(&path)?.display().to_string();
            crate::styles::record_selection(
                workspace,
                document,
                &workspace.read_toml_value(&location)?,
                &location,
            )
        })
        .transpose()?;
    crate::styles::selection(
        workspace,
        document,
        style.or(record.as_ref().map(|selection| selection.style.as_str())),
        substyle.or(record.as_ref().map(|selection| selection.substyle.as_str())),
    )
}

fn watch_cv(
    workspace: &Workspace,
    locale: &str,
    pages: Option<usize>,
    style: Option<&str>,
    substyle: Option<&str>,
) -> Result<()> {
    let selection = crate::styles::selection(workspace, "cv", style, substyle)?;
    let leaf = render::cv_leaf(workspace, locale, &selection)?;
    let pages = pages.unwrap_or(leaf.default_pages);
    watch_loop(
        &format!(
            "ccvl sources for {}/{}/{locale} {pages}-page CV",
            selection.style, selection.substyle
        ),
        || cvl_digest(workspace),
        || {
            Ok(vec![Compiler::new(workspace)?.render(
                workspace,
                &render::cvl_cv_spec(workspace, locale, pages, Some(&selection))?,
            )?])
        },
    )
}

fn watch_cl(
    workspace: &Workspace,
    locale: &str,
    pages: Option<usize>,
    style: Option<&str>,
    substyle: Option<&str>,
) -> Result<()> {
    let selection = crate::styles::selection(workspace, "cl", style, substyle)?;
    watch_loop(
        &format!(
            "ccvl sources for {}/{}/{locale} cover letter",
            selection.style, selection.substyle
        ),
        || cvl_digest(workspace),
        || {
            Ok(vec![Compiler::new(workspace)?.render(
                workspace,
                &render::cvl_cl_spec(workspace, locale, pages, Some(&selection))?,
            )?])
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

/// General CVL sources: style leaves and records, the shared Typst
/// machinery and assets, plus the workspace contract. Built PDFs are
/// excluded so a render never retriggers itself.
fn cvl_digest(workspace: &Workspace) -> Result<Vec<u8>> {
    digest_roots(&[
        workspace.path("cvl"),
        workspace.path(".agent/typst"),
        workspace.path("ccvl.json"),
    ])
}

/// Opportunity sources: the record's leaf adapters and their inputs, the
/// shared renderers and knobs, the profile and workspace contract, plus the
/// keyed record directory including its generated typst/ copies. PDFs stay
/// out so a render never retriggers its own watcher.
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
        workspace.path("cvl/shared"),
        workspace.path("cvl/cv/harvard/contract.toml"),
        workspace.path("cvl/cl/harvard/contract.toml"),
        workspace.path("ccvl.json"),
        directory,
    ];
    match render::opportunity_selection(workspace, organisation, position) {
        Ok(selection) => {
            if let Ok(leaf) = render::cv_leaf(workspace, &selection.locale, &selection.cv) {
                roots.push(leaf.style_dir().to_path_buf());
                roots.push(leaf.adapter());
                roots.push(leaf.strings());
                roots.push(leaf.substyle_file());
            }
            if let Ok(leaf) = render::cl_leaf(workspace, &selection.locale, &selection.cl) {
                roots.push(leaf.style_dir().to_path_buf());
                roots.push(leaf.adapter());
                roots.push(leaf.strings());
                roots.push(leaf.substyle_file());
            }
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
        [
            "typ", "toml", "json", "png", "jpg", "jpeg", "webp", "svg", "ttf", "otf",
        ]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
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
mod tests;
