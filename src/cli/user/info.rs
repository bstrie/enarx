// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::{client, UserSpec};

use anyhow::Context;
use clap::Args;

/// Retrieve information about a user account on an Enarx package host.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    insecure_auth_token_file: Option<String>,
    spec: UserSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let cl = client(&self.spec.host, &self.insecure_auth_token_file)
            .context("Failed to build client")?;
        let user = cl.user(&self.spec.ctx);
        let record = user
            .get()
            .with_context(|| format!("Failed to get record for user: {}", self.spec.ctx.name))?;
        // TODO: make this useful
        println!("Subject: {}", record.subject);
        Ok(())
    }
}
