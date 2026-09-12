use anyhow::{Context, Result, ensure};
use ctypst::{CompileRequest, Document, Engine, PageConstraint};

use crate::{StyleBundle, StyleDocument};

/// An in-memory renderer: no project root, database, shell, or host file access.
pub struct StyleRenderer {
    engine: Engine,
    bundle_sha256: String,
}

impl StyleRenderer {
    pub fn new(bundle: &StyleBundle) -> Result<Self> {
        bundle.validate()?;
        let mut builder = Engine::builder().fonts(ctypst::fonts::documents());
        for (path, bytes) in &bundle.files {
            builder = if path.ends_with(".typ") {
                builder.source(
                    path,
                    std::str::from_utf8(bytes).context("Typst source must be UTF-8")?,
                )?
            } else {
                builder.binary(path, bytes.clone())?
            };
        }
        for path in &bundle.fonts {
            builder = builder.fonts([bundle.files[path].clone()]);
        }
        Ok(Self {
            bundle_sha256: bundle.sha256()?,
            engine: builder
                .build()
                .context("cannot initialize bundled CCVL style")?,
        })
    }

    pub fn compile(&self, document: &StyleDocument) -> Result<Document> {
        document.validate()?;
        ensure!(
            document.bundle_sha256 == self.bundle_sha256,
            "renderer belongs to a different style bundle"
        );
        let mut inputs = document.bundle.inputs.clone();
        inputs.insert(
            "application".into(),
            "/__ccvl_inputs/application.toml".into(),
        );
        inputs.insert("profile".into(), "/__ccvl_inputs/profile.toml".into());
        Ok(self
            .engine
            .compile(
                CompileRequest::new(&document.bundle.entry)
                    .inputs(inputs)
                    .pages(PageConstraint::Exactly(document.bundle.pages))
                    .binary_file(
                        "__ccvl_inputs/application.toml",
                        toml::to_string(&document.record)?.into_bytes(),
                    )
                    .binary_file(
                        "__ccvl_inputs/profile.toml",
                        toml::to_string(&document.profile)?.into_bytes(),
                    ),
            )
            .context("CCVL style compilation failed")?
            .document)
    }

    pub fn pdf(&self, document: &Document) -> Result<Vec<u8>> {
        crate::pdf::lowercase_locales(
            &self
                .engine
                .pdf(document, 0)
                .context("CCVL PDF export failed")?,
        )
    }

    #[must_use]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}
