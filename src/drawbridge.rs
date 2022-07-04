// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use drawbridge_client::types::{RepositoryContext, TagContext, UserContext};
use drawbridge_client::{Client, Repository, Tag, User};
use rustls::{Certificate, RootCertStore};

const DEFAULT_HOST: &str = "example.com";

#[derive(Debug)]
pub struct UserSpec {
    pub client: Client,
    pub ctx: UserContext,
}

impl UserSpec {
    pub fn user(&self) -> User {
        self.client.user(&self.ctx)
    }
}

impl FromStr for UserSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (client, ctx) = parse_slug(s)?;
        Ok(Self { client, ctx })
    }
}

#[derive(Debug)]
pub struct RepoSpec {
    pub client: Client,
    pub ctx: RepositoryContext,
}

impl RepoSpec {
    pub fn repo(&self) -> Repository {
        self.client.repository(&self.ctx)
    }
}

impl FromStr for RepoSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (client, ctx) = parse_slug(s)?;
        Ok(Self { client, ctx })
    }
}

#[derive(Debug)]
pub struct TagSpec {
    pub client: Client,
    pub ctx: TagContext,
}

impl TagSpec {
    pub fn tag(&self) -> Tag {
        self.client.tag(&self.ctx)
    }
}

impl FromStr for TagSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (client, ctx) = parse_slug(s)?;
        Ok(Self { client, ctx })
    }
}

fn parse_slug<T: FromStr>(s: &str) -> Result<(Client, T), <T as FromStr>::Err> {
    let (host, name) = match s.split_once('+') {
        Some((host, name)) => (host, name),
        None => (DEFAULT_HOST, s),
    };

    let client = create_client(host);
    let ctx = name.parse()?;

    Ok((client, ctx))
}

fn create_client(addr: &str) -> Client {
    let cl = Client::builder(addr.parse().unwrap());

    // TODO: test only?
    let cl = cl.roots({
        let mut roots = RootCertStore::empty();
        rustls_pemfile::certs(&mut std::io::BufReader::new(
            include_bytes!("../tests/client/data/ca.crt").as_slice(),
        ))
        .unwrap()
        .into_iter()
        .map(Certificate)
        .try_for_each(|ref cert| roots.add(cert))
        .unwrap();
        roots
    });

    cl.token("test-token").build().unwrap()
}
