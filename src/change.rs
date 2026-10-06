use std::{fs, path::PathBuf};

use thiserror::Error;

use crate::domain::NormalizedTarget;

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedChange {
    pub source: PathBuf,
    pub target: NormalizedTarget,
    pub old_text: String,
    pub new_text: String,
}

#[derive(Debug, Error)]
pub enum ChangeError {
    #[error("could not read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{0} changed after LazyConfig read it. Press [r] to reload.")]
    SourceChanged(PathBuf),
}

impl PreparedChange {
    pub fn write(&self) -> Result<(), ChangeError> {
        let current = fs::read_to_string(&self.source).map_err(|source| ChangeError::Read {
            path: self.source.clone(),
            source,
        })?;
        if current != self.old_text {
            return Err(ChangeError::SourceChanged(self.source.clone()));
        }

        fs::write(&self.source, &self.new_text).map_err(|source| ChangeError::Write {
            path: self.source.clone(),
            source,
        })
    }

    pub fn changed_lines(&self) -> Vec<String> {
        let old = self.old_text.lines().collect::<Vec<_>>();
        let new = self.new_text.lines().collect::<Vec<_>>();

        if old.len() == new.len() {
            return old
                .iter()
                .zip(&new)
                .enumerate()
                .filter(|(_, (old, new))| old != new)
                .flat_map(|(index, (old, new))| {
                    let line = index + 1;
                    [format!("-{line}: {old}"), format!("+{line}: {new}")]
                })
                .collect();
        }

        let prefix = old
            .iter()
            .zip(&new)
            .take_while(|(old, new)| old == new)
            .count();
        let suffix = old[prefix..]
            .iter()
            .rev()
            .zip(new[prefix..].iter().rev())
            .take_while(|(old, new)| old == new)
            .count();
        let removed = old[prefix..old.len() - suffix]
            .iter()
            .enumerate()
            .map(|(offset, line)| format!("-{}: {line}", prefix + offset + 1));
        let added = new[prefix..new.len() - suffix]
            .iter()
            .enumerate()
            .map(|(offset, line)| format!("+{}: {line}", prefix + offset + 1));

        removed.chain(added).collect()
    }
}
