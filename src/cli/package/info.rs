// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::TagSpec;

use clap::Args;

/// Retrieve information about a published package.
#[derive(Args, Debug)]
pub struct Options {
    spec: TagSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let tag = self.spec.tag();
        let tag_entry = tag.get()?;
        dbg!(tag_entry);

        Ok(())
    }
}
