use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};

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
    /// Export a neutral style variant for native or web consumers.
    ExportStyle {
        #[arg(value_parser = ["cv", "cl"])]
        document: String,
        locale: String,
        #[arg(long)]
        style: Option<String>,
        #[arg(long)]
        substyle: Option<String>,
        #[arg(long)]
        pages: Option<usize>,
        #[arg(long)]
        paper: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Render a versioned style bundle with explicit user inputs.
    RenderStyle {
        #[arg(long)]
        bundle: PathBuf,
        #[arg(long)]
        application: PathBuf,
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Prepare and validate an independently reviewed document package.
    Review {
        #[command(subcommand)]
        command: crate::review::Command,
    },
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
        /// Paper preset declared by the selected style.
        #[arg(long)]
        paper: Option<String>,
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
        /// Paper preset declared by the selected style.
        #[arg(long)]
        paper: Option<String>,
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
        /// Paper preset declared by the selected style.
        #[arg(long)]
        paper: Option<String>,
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
        /// Paper preset declared by the selected style.
        #[arg(long)]
        paper: Option<String>,
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
        /// Paper preset declared by the selected style.
        #[arg(long)]
        paper: Option<String>,
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
        /// Score complete option assessments without contacting a model provider.
        #[arg(long)]
        response_file: Option<PathBuf>,
        #[arg(long)]
        summary: Option<PathBuf>,
        /// Groq request model, or a caller-provided label for a saved response.
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
    if let Command::RenderStyle {
        bundle,
        application,
        profile,
        output,
    } = &args.command
    {
        crate::bundle::render(bundle, application, profile, output)?;
        return Ok(ExitCode::SUCCESS);
    }
    let workspace = Workspace::discover(args.root.as_deref())?;
    crate::runtime::verify(workspace.root())?;
    let mut exit_code = ExitCode::SUCCESS;
    match args.command {
        Command::ExportStyle {
            document,
            locale,
            style,
            substyle,
            pages,
            paper,
            output,
        } => {
            let bundle = crate::bundle::export(
                &workspace,
                &document,
                &locale,
                style.as_deref(),
                substyle.as_deref(),
                pages,
                paper.as_deref(),
            )?;
            std::fs::write(&output, serde_json::to_vec_pretty(&bundle)?)?;
            println!("{} {} {}", bundle.id, bundle.version, output.display());
        }
        Command::RenderStyle { .. } => unreachable!("handled without a workspace"),
        Command::Review { command } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&crate::review::run(&workspace, command)?)?
            );
        }
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
            paper,
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
                serde_json::to_string_pretty(&crate::settings::explain(
                    &workspace,
                    &leaf,
                    paper.as_deref()
                )?)?
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
            paper,
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
            let record = crate::content::read_record(&workspace, &application)?;
            let pages = pages
                .or(record
                    .pointer("/options/pages")
                    .and_then(serde_json::Value::as_u64)
                    .map(usize::try_from)
                    .transpose()?)
                .unwrap_or(leaf.default_pages);
            let profile = workspace
                .existing_inside(profile.unwrap_or_else(|| PathBuf::from("cvl/profile.toml")))?;
            let spec = render::document_spec(
                &workspace,
                &leaf,
                pages,
                &application,
                &profile,
                output.as_deref(),
                paper.as_deref(),
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
            paper,
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
            let record = crate::content::read_record(&workspace, &application)?;
            let pages = pages
                .or(record
                    .pointer("/options/cl_pages")
                    .and_then(serde_json::Value::as_u64)
                    .map(usize::try_from)
                    .transpose()?)
                .unwrap_or(leaf.default_pages);
            let profile = workspace
                .existing_inside(profile.unwrap_or_else(|| PathBuf::from("cvl/profile.toml")))?;
            let spec = render::document_spec(
                &workspace,
                &leaf,
                pages,
                &application,
                &profile,
                output.as_deref(),
                paper.as_deref(),
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
            paper,
        } => {
            watch_cv(
                &workspace,
                &locale,
                pages,
                style.as_deref(),
                substyle.as_deref(),
                paper.as_deref(),
            )?;
        }
        Command::WatchCl {
            locale,
            pages,
            style,
            substyle,
            paper,
        } => {
            watch_cl(
                &workspace,
                &locale,
                pages,
                style.as_deref(),
                substyle.as_deref(),
                paper.as_deref(),
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
            let model = model.or_else(|| {
                response_file
                    .is_none()
                    .then(|| std::env::var("GROQ_MODEL").ok())
                    .flatten()
            });
            let outcome = skills::run_hosted_evaluation(
                &workspace,
                &cases,
                &skills_root,
                &output,
                response_file.as_deref(),
                model.as_deref(),
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
                &crate::content::read_record(workspace, &path)?,
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
    paper: Option<&str>,
) -> Result<()> {
    crate::watch::run(workspace, &format!("{locale} CV"), |workspace| {
        let selection = crate::styles::selection(workspace, "cv", style, substyle)?;
        let leaf = render::cv_leaf(workspace, locale, &selection)?;
        let spec = render::cvl_spec_with_paper(
            workspace,
            &leaf,
            pages.unwrap_or(leaf.default_pages),
            paper,
        )?;
        Ok(vec![Compiler::new(workspace)?.render(workspace, &spec)?])
    })
}

fn watch_cl(
    workspace: &Workspace,
    locale: &str,
    pages: Option<usize>,
    style: Option<&str>,
    substyle: Option<&str>,
    paper: Option<&str>,
) -> Result<()> {
    crate::watch::run(workspace, &format!("{locale} cover letter"), |workspace| {
        let selection = crate::styles::selection(workspace, "cl", style, substyle)?;
        let leaf = render::cl_leaf(workspace, locale, &selection)?;
        let spec = render::cvl_spec_with_paper(
            workspace,
            &leaf,
            pages.unwrap_or(leaf.default_pages),
            paper,
        )?;
        Ok(vec![Compiler::new(workspace)?.render(workspace, &spec)?])
    })
}

fn watch_opportunity(workspace: &Workspace, organisation: &str, position: &str) -> Result<()> {
    // Reject malformed keys, while allowing a temporarily missing record to be
    // restored without restarting the watcher.
    opportunity::record_path(workspace, organisation, position, false)?;
    crate::watch::run(
        workspace,
        &format!("opportunity {organisation}/{position}"),
        |workspace| render::render_opportunity(workspace, organisation, position),
    )
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
