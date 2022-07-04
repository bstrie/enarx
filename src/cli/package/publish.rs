// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::{client, TagSpec};

use std::path::Path;

use anyhow::Context;
use clap::Args;

/// Publish a new package.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    insecure_auth_token_file: Option<String>,
    spec: TagSpec,
    path: String,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let cl = client(&self.spec.host, &self.insecure_auth_token_file)
            .context("Failed to build client")?;
        let tag = cl.tag(&self.spec.ctx);
        // why does create_from_path_unsigned return a bool? what is this useful for?
        let (_tag_created, _tree_created) = tag
            .create_from_path_unsigned(&Path::new(&self.path))
            .context("Failed to create tag and upload tree")?;

        Ok(())
    }
}
