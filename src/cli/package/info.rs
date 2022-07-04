// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::{client, TagSpec};

use anyhow::Context;
use clap::Args;

/// Retrieve information about a published package.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    insecure_auth_token_file: Option<String>,
    spec: TagSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let cl = client(&self.spec.host, &self.insecure_auth_token_file)
            .context("Failed to build client")?;
        let tag = cl.tag(&self.spec.ctx);
        let _tag_entry = tag
            .get()
            .context("Failed to retrieve package information")?;
        // TODO: make this actually useful
        println!("Package exists");

        Ok(())
    }
}
