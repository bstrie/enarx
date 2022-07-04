// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::UserSpec;

use anyhow::{anyhow, Context};
use clap::Args;
use drawbridge_client::types::UserRecord;
use drawbridge_client::Url;
use oauth2::AccessToken;
use openidconnect::{IssuerUrl, ClientId};
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreUserInfoClaims};
use openidconnect::ureq::http_client;

/// Register a new user account with a package host.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long, default_value = "https://auth.profian.com/")]
    oidc_domain: Url,
    #[clap(long, default_value = "4NuaJxkQv8EZBeJKE56R57gKJbxrTLG2")]
    oidc_client_id: String,
    spec: UserSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let user = self.spec.user();
/*
		let provider_metadata = CoreProviderMetadata::discover(
			&IssuerUrl::new(self.oidc_domain.to_string())?,
			http_client,
		)?;

        panic!("block here????");

		let client =
			CoreClient::from_provider_metadata(
				provider_metadata,
				ClientId::new(self.oidc_client_id),
				None
			);

        let userinfo: CoreUserInfoClaims = client
            .user_info(AccessToken::new("test-token".to_string()), None)
            .map_err(|err| anyhow!("No user info endpoint: {:?}", err))?
            .request(http_client)
            .map_err(|err| anyhow!("Failed requesting user info: {:?}", err))?;

        let subject = userinfo.subject();

        println!("{:?}", subject);
        return Ok(());
*/
        // TODO: but actually
        let record = UserRecord {
            subject: "test|subject".into(),
        };

        user.create(&record)
            .context("failed to register new user")?;

        Ok(())
    }
}
