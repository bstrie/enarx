// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge;

use anyhow::Context;
use clap::Args;

/// Retrieve information about a user account on an Enarx package host.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    addr: String,
    user: String
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let client = drawbridge::client(self.addr);
        let user_name = self.user.parse().unwrap();
        let oidc_user = client.user(&user_name);
        let user = oidc_user.get().with_context(|| format!("failed to get record for user: {}", self.user))?;
        println!("asdf: {}", user.subject);
        Ok(())
    }
}
