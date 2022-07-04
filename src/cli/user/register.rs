// SPDX-License-Identifier: Apache-2.0

use crate::drawbridge::{client, UserSpec};

use anyhow::Context;
use clap::Args;
use drawbridge_client::types::UserRecord;
use drawbridge_client::Url;
use oauth2::AccessToken;
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreUserInfoClaims};
use openidconnect::ureq::http_client;
use openidconnect::{ClientId, IssuerUrl};

/// Register a new user account with a package host.
#[derive(Args, Debug)]
pub struct Options {
    #[clap(long)]
    insecure_auth_token_file: Option<String>,
    #[clap(long, default_value = "https://auth.profian.com/")]
    oidc_domain: Url,
    #[clap(long, default_value = "4NuaJxkQv8EZBeJKE56R57gKJbxrTLG2")]
    oidc_client_id: String,
    spec: UserSpec,
}

impl Options {
    pub fn execute(self) -> anyhow::Result<()> {
        let cl = client(&self.spec.host, &self.insecure_auth_token_file)
            .context("Failed to build client")?;
        let user = cl.user(&self.spec.ctx);

        let provider_metadata = CoreProviderMetadata::discover(
            &IssuerUrl::new(self.oidc_domain.to_string())
                .context("Failed to construct issuer URL")?,
            http_client,
        )
        .context("Failed to discover provider metadata")?;

        let oidc_client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(self.oidc_client_id),
            None,
        );

        let userinfo: CoreUserInfoClaims = oidc_client
            .user_info(AccessToken::new("test-token".to_string()), None)
            .context("Failed to find user info endpoint")?
            .request(http_client)
            .context("Failed to make user info request")?;

        let subject = userinfo.subject().to_string();

        let record = UserRecord { subject };

        user.create(&record)
            .context("Failed to register new user")?;

        Ok(())
    }
}
