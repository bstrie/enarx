// SPDX-License-Identifier: Apache-2.0

use drawbridge_client::Client;
use rustls::{Certificate, RootCertStore};

const TOKEN: &str = "test-token";

pub fn client(srv_addr: String) -> Client {
    let cl = Client::builder(srv_addr.parse().unwrap())
        .roots({
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

    cl.token(TOKEN).build().unwrap()
}
