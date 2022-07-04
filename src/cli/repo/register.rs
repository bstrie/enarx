// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::{client, RepoSpec};

use anyhow::Context;
use clap::Args;
use drawbridge_client::types::RepositoryConfig;

/// Register a new repository.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    insecure_auth_token_file: Option<String>,
    #[clap(long)]
    public: bool,
    spec: RepoSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let cl = client(&self.spec.host, &self.insecure_auth_token_file)
            .context("Failed to build client")?;
        let repo = cl.repository(&self.spec.ctx);
        let repo_config = RepositoryConfig {
            public: self.public,
        };
        repo.create(&repo_config)
            .context("Failed to register repository")?;
        Ok(())
    }
}
