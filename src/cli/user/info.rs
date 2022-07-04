// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::UserSpec;

use anyhow::Context;
use clap::Args;

/// Retrieve information about a user account on an Enarx package host.
#[derive(Args, Debug)]
pub struct Options {
    spec: UserSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let user = self.spec.user();
        let record = user
            .get()
            .with_context(|| format!("failed to get record for user: {}", self.spec.ctx.name))?;
        println!("asdf: {}", record.subject);
        Ok(())
    }
}
