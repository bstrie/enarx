// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::RepoSpec;

use anyhow::Context;
use clap::Args;

/// List all tags associated with a repository.
#[derive(Args, Debug)]
pub struct Options {
    spec: RepoSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let repo = self.spec.repo();
        //let record = repo.get().context("failed to get repo")?;
        let tags = repo.tags().context("failed to get tags")?;
        //println!("repo: {}", res.public);
        for tag in tags {
            println!("{tag}");
        }
        Ok(())
    }
}
