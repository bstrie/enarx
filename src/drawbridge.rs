// SPDX-License-Identifier: Apache-2.0

use std::fs::read_to_string;
use std::str::FromStr;

use anyhow::Context;
use drawbridge_client::types::{RepositoryContext, TagContext, UserContext};
use drawbridge_client::Client;
use rustls::{Certificate, RootCertStore};

const DEFAULT_HOST: &str = "example.com";

#[derive(Debug)]
pub struct UserSpec {
    pub host: String,
    pub ctx: UserContext,
}

impl FromStr for UserSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (host, ctx) = parse_slug(s).with_context(|| format!("Failed to parse slug: {s}"))?;
        Ok(Self { host, ctx })
    }
}

#[derive(Debug)]
pub struct RepoSpec {
    pub host: String,
    pub ctx: RepositoryContext,
}

impl FromStr for RepoSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (host, ctx) = parse_slug(s).with_context(|| format!("Failed to parse slug: {s}"))?;
        Ok(Self { host, ctx })
    }
}

#[derive(Debug)]
pub struct TagSpec {
    pub host: String,
    pub ctx: TagContext,
}

impl FromStr for TagSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (host, ctx) = parse_slug(s).with_context(|| format!("Failed to parse slug: {s}"))?;
        Ok(Self { host, ctx })
    }
}

fn parse_slug<T: FromStr>(s: &str) -> Result<(String, T), <T as FromStr>::Err> {
    let (host, name) = match s.split_once('+') {
        Some((host, name)) => (host, name),
        None => (DEFAULT_HOST, s),
    };

    let host = host.to_string();
    let ctx = name.parse()?;

    Ok((host, ctx))
}

pub fn client(host: &str, token_file: &Option<String>) -> anyhow::Result<Client> {
    let token = match token_file {
        Some(path) => {
            read_to_string(path).with_context(|| format!("Failed to read path: {path}"))?
        }
        None => keyring::Entry::new("enarx", "drawbridge")
            .get_password()
            .context("Failed to read credentials from keyring")?,
    };

    let url = format!("https://{host}");

    let mut cl = Client::builder(
        url.parse()
            .with_context(|| format!("Failed to parse URL: {url}"))?,
    );

    // For local development and integration testing
    if host.starts_with("localhost:") {
        cl = cl.roots({
            let mut roots = RootCertStore::empty();

            rustls_pemfile::certs(&mut std::io::BufReader::new(
                include_bytes!("../tests/client/certs/ca.crt").as_slice(),
            ))
            .unwrap()
            .into_iter()
            .map(Certificate)
            .try_for_each(|ref cert| {
                roots
                    .add(cert)
                    .with_context(|| format!("Failed to add root certificate: {cert:?}"))
            })?;

            roots
        });
    }

    let cl = cl
        .token(token.trim())
        .build()
        .context("Failed to build client")?;

    Ok(cl)
}
