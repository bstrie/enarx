// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::{client, RepoSpec};

use anyhow::Context;
use clap::Args;

/// List all tags associated with a repository.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    insecure_auth_token_file: Option<String>,
    spec: RepoSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let cl = client(&self.spec.host, &self.insecure_auth_token_file)
            .context("Failed to build client")?;
        let repo = cl.repository(&self.spec.ctx);
        let record = repo
            .get()
            .context("Failed to retrieve repository information")?;
        let visibility = if record.public { "public" } else { "private" };
        println!("Visibility: {visibility}");
        let tags = repo.tags().context("Failed to retrieve repository tags")?;
        println!("Tags:");
        for tag in tags {
            println!("\t{tag}");
        }
        Ok(())
    }
}
