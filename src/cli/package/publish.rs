// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::TagSpec;

use std::path::Path;

use anyhow::Context;
use clap::Args;

/// Publish a new package.
#[derive(Args, Debug)]
pub struct Options {
    spec: TagSpec,
    path: String,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let tag = self.spec.tag();
        // why does create_from_path_unsigned return a bool?
        let (_tag_created, _tree_created) =
            tag.create_from_path_unsigned(&Path::new(&self.path))
                .context("failed to create a tag and upload the tree")?;

        Ok(())
    }
}
