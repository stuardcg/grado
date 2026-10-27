//! Loads and validates the `content/` directory.
//!
//! Layout:
//!
//! ```text
//! content/
//!   coach.toml            tunable values, available as {{section.key}} everywhere
//!   core.md               the coach every level shares
//!   shared/<file>         sample data the agent can fetch by name
//!   levels/<NN-slug>/
//!     level.toml          metadata and criteria (checks stay server-side)
//!     brief.md            agent briefing for this level
//!     page.md             what the player reads on the website
//!     guides/<tier>-<name>.md   escalating help, tier 1 first
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::level::{Guide, Level, LevelSpec};
use crate::template::{self, Vars};

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("reading {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("parsing {path}: {source}")]
    Toml { path: PathBuf, source: toml::de::Error },
    #[error("{path}: unknown placeholders {names:?}")]
    Placeholder { path: PathBuf, names: Vec<String> },
    #[error("invalid content: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct Catalog {
    /// Short hash of every content file. Stored with progress so a run can be
    /// traced back to the content it was played on.
    pub version: String,
    pub core_md: String,
    pub vars: Vars,
    levels: Vec<Level>,
    shared: BTreeMap<String, String>,
}

impl Catalog {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, CatalogError> {
        Self::load_with(root, Vars::new())
    }

    /// Loads content, letting deployment settings (e.g. `site.url`) override
    /// values from `coach.toml`.
    pub fn load_with(root: impl AsRef<Path>, overrides: Vars) -> Result<Self, CatalogError> {
        let root = root.as_ref();
        let coach_path = root.join("coach.toml");
        let coach: toml::Table = parse_toml(&coach_path)?;
        let mut vars = template::flatten(&coach);
        vars.extend(overrides);

        let core_path = root.join("core.md");
        let core_md = render_file(&core_path, &vars)?;

        let mut levels = Vec::new();
        let levels_dir = root.join("levels");
        for entry in read_dir_sorted(&levels_dir)? {
            if entry.is_dir() {
                levels.push(load_level(&entry, &vars)?);
            }
        }
        levels.sort_by_key(Level::number);

        let mut shared = BTreeMap::new();
        let shared_dir = root.join("shared");
        if shared_dir.exists() {
            for path in read_dir_sorted(&shared_dir)? {
                if path.is_file() {
                    let name = file_name(&path);
                    shared.insert(name, read(&path)?);
                }
            }
        }

        let catalog = Self { version: content_hash(root)?, core_md, vars, levels, shared };
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn levels(&self) -> &[Level] {
        &self.levels
    }

    pub fn level(&self, id: &str) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == id)
    }

    pub fn first_level(&self) -> &Level {
        &self.levels[0]
    }

    pub fn next_level(&self, id: &str) -> Option<&Level> {
        let index = self.levels.iter().position(|l| l.id == id)?;
        self.levels.get(index + 1)
    }

    pub fn previous_level(&self, id: &str) -> Option<&Level> {
        let index = self.levels.iter().position(|l| l.id == id)?;
        index.checked_sub(1).and_then(|i| self.levels.get(i))
    }

    pub fn shared_file(&self, name: &str) -> Option<&str> {
        self.shared.get(name).map(String::as_str)
    }

    pub fn shared_file_names(&self) -> impl Iterator<Item = &str> {
        self.shared.keys().map(String::as_str)
    }

    fn validate(&self) -> Result<(), CatalogError> {
        if self.levels.is_empty() {
            return Err(CatalogError::Invalid("no levels found".into()));
        }
        for (expected, level) in self.levels.iter().enumerate() {
            if level.number() as usize != expected {
                return Err(CatalogError::Invalid(format!(
                    "level {} has number {}, expected {expected}; numbers must run 0, 1, 2, ...",
                    level.id,
                    level.number()
                )));
            }
            if level.spec.criteria.is_empty() {
                return Err(CatalogError::Invalid(format!("level {} has no criteria", level.id)));
            }
            let mut seen = BTreeSet::new();
            for criterion in &level.spec.criteria {
                if !seen.insert(&criterion.id) {
                    return Err(CatalogError::Invalid(format!(
                        "level {} repeats criterion id {}",
                        level.id, criterion.id
                    )));
                }
            }
            for (index, guide) in level.guides.iter().enumerate() {
                if guide.tier as usize != index + 1 {
                    return Err(CatalogError::Invalid(format!(
                        "level {} guides must be numbered 1, 2, 3, ... without gaps",
                        level.id
                    )));
                }
            }
        }
        Ok(())
    }
}

fn load_level(dir: &Path, vars: &Vars) -> Result<Level, CatalogError> {
    let id = file_name(dir);
    let spec_path = dir.join("level.toml");
    let spec: LevelSpec = toml::from_str(&render_file(&spec_path, vars)?)
        .map_err(|source| CatalogError::Toml { path: spec_path, source })?;
    let brief_md = render_file(&dir.join("brief.md"), vars)?;
    let page_md = render_file(&dir.join("page.md"), vars)?;

    let mut guides = Vec::new();
    let guides_dir = dir.join("guides");
    if guides_dir.exists() {
        for path in read_dir_sorted(&guides_dir)? {
            if path.extension().is_some_and(|ext| ext == "md") {
                guides.push(load_guide(&path, vars)?);
            }
        }
    }
    guides.sort_by_key(|g| g.tier);

    Ok(Level { id, spec, brief_md, page_md, guides })
}

fn load_guide(path: &Path, vars: &Vars) -> Result<Guide, CatalogError> {
    let name = file_name(path);
    let tier = name.split('-').next().and_then(|n| n.parse::<u8>().ok()).ok_or_else(|| {
        CatalogError::Invalid(format!("{}: guide names start with a tier number", path.display()))
    })?;
    let body = render_file(path, vars)?;
    let title = body.lines().find_map(|line| line.strip_prefix("# ")).unwrap_or(&name).trim().to_owned();
    Ok(Guide { tier, title, body_md: body })
}

fn render_file(path: &Path, vars: &Vars) -> Result<String, CatalogError> {
    let source = read(path)?;
    template::render(&source, vars)
        .map_err(|names| CatalogError::Placeholder { path: path.to_owned(), names })
}

fn parse_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CatalogError> {
    toml::from_str(&read(path)?).map_err(|source| CatalogError::Toml { path: path.to_owned(), source })
}

fn read(path: &Path) -> Result<String, CatalogError> {
    fs::read_to_string(path).map_err(|source| CatalogError::Io { path: path.to_owned(), source })
}

fn read_dir_sorted(dir: &Path) -> Result<Vec<PathBuf>, CatalogError> {
    let entries = fs::read_dir(dir).map_err(|source| CatalogError::Io { path: dir.to_owned(), source })?;
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    Ok(paths)
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

fn content_hash(root: &Path) -> Result<String, CatalogError> {
    let mut files: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();
    files.sort();
    let mut hasher = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(root).unwrap_or(&path);
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(fs::read(&path).map_err(|source| CatalogError::Io { path: path.clone(), source })?);
    }
    Ok(hex::encode(hasher.finalize())[..12].to_owned())
}
