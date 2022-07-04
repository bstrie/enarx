// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::RepoSpec;

use anyhow::Context;
use clap::Args;
use drawbridge_client::types::RepositoryConfig;

/// Register a new repository.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    public: bool,
    spec: RepoSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let repo = self.spec.repo();
        let repo_config = RepositoryConfig {
            public: self.public,
        };
        repo.create(&repo_config)
            .context("failed to register repo")?;
        Ok(())
    }
}
