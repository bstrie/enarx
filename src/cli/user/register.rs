// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::UserSpec;

use anyhow::Context;
use clap::Args;
use drawbridge_client::types::UserRecord;

/// Register a new user account with a package host.
#[derive(Args, Debug)]
pub struct Options {
    spec: UserSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let user = self.spec.user();

        // TODO: but actually
        let record = UserRecord {
            subject: "test|subject".into(),
        };

        user.create(&record)
            .context("failed to register new user")?;

        Ok(())
    }
}
