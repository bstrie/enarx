// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::net::SocketAddr;
use std::process::Command;
use std::str::from_utf8;

use drawbridge_app::{App, OidcConfig, TlsConfig};

use async_std::fs::{create_dir, write};
use async_std::net::{Ipv4Addr, TcpListener};
use async_std::task::{spawn, JoinHandle};
use futures::channel::oneshot::{channel, Sender};
use futures::{join, try_join, StreamExt};
use http_types::convert::{json, Serialize};
use http_types::{Body, Response, StatusCode};
use openidconnect::core::{
    CoreJsonWebKey, CoreJsonWebKeySet, CoreJwsSigningAlgorithm, CoreProviderMetadata,
    CoreResponseType, CoreSubjectIdentifierType, CoreUserInfoClaims,
};
use openidconnect::{
    AuthUrl, EmptyAdditionalClaims, EmptyAdditionalProviderMetadata, IssuerUrl, JsonWebKeySetUrl,
    ResponseTypes, StandardClaims, SubjectIdentifier, UserInfoUrl,
};
use tempfile::{tempdir, TempDir};

pub fn enarx(args: String) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_enarx"));

    for arg in args.split(' ').skip(1) {
        cmd.arg(arg);
    }

    let out = cmd.output().expect("failed to execute `enarx`");

    Output {
        success: out.status.success(),
        output: from_utf8(&out.stdout).unwrap().to_string(),
        err: from_utf8(&out.stderr).unwrap().to_string(),
    }
}

#[allow(dead_code)]
pub struct Output {
    pub success: bool,
    pub output: String,
    pub err: String,
}

pub async fn run<F, R>(commands: F)
where
    F: FnOnce(String) -> R,
    R: Future<Output = ()>,
{
    env_logger::builder().is_test(true).init();
    let (oidc_addr, oidc_tx, oidc_handle) = init_oidc().await;
    let (srv_port, srv_tx, srv_handle) = init_drawbridge(oidc_addr).await;
    let srv_addr = format!("https://localhost:{srv_port}");

    commands(srv_addr).await;

    // Gracefully stop servers
    assert_eq!(oidc_tx.send(()), Ok(()));
    assert_eq!(srv_tx.send(()), Ok(()));
    assert!(matches!(join!(oidc_handle, srv_handle), ((), ())));
}

async fn init_oidc() -> (SocketAddr, Sender<()>, JoinHandle<()>) {
    let oidc_lis = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("failed to bind to address");

    let oidc_addr = oidc_lis.local_addr().unwrap();

    let (oidc_tx, oidc_rx) = channel::<()>();

    let oidc_handle = spawn(async move {
        oidc_lis
            .incoming()
            .take_until(oidc_rx)
            .for_each_concurrent(None, |stream| async {
                async_h1::accept(
                    stream.expect("failed to initialize stream"),
                    |req| async move {
                        fn json_response(
                            body: &impl Serialize,
                        ) -> Result<Response, http_types::Error> {
                            let mut res = Response::new(StatusCode::Ok);
                            res.insert_header("Content-Type", "application/json");
                            let body = Body::from_json(&json!(body))?;
                            res.set_body(body);
                            Ok(res)
                        }

                        let oidc_url = format!("http://{oidc_addr}/");
                        match req.url().path() {
                            "/.well-known/openid-configuration" => json_response(
                                &CoreProviderMetadata::new(
                                    // Parameters required by the OpenID Connect Discovery spec.
                                    IssuerUrl::new(oidc_url.to_string()).unwrap(),
                                    AuthUrl::new(format!("{oidc_url}authorize")).unwrap(),
                                    // Use the JsonWebKeySet struct to serve the JWK Set at this URL.
                                    JsonWebKeySetUrl::new(format!("{oidc_url}jwks")).unwrap(),
                                    vec![ResponseTypes::new(vec![CoreResponseType::Code])],
                                    vec![CoreSubjectIdentifierType::Pairwise],
                                    vec![CoreJwsSigningAlgorithm::RsaSsaPssSha256],
                                    EmptyAdditionalProviderMetadata {},
                                )
                                .set_userinfo_endpoint(Some(
                                    UserInfoUrl::new(format!("{oidc_url}userinfo")).unwrap(),
                                )),
                            ),
                            "/jwks" => json_response(&CoreJsonWebKeySet::new(vec![
                                CoreJsonWebKey::new_rsa(b"ntest".to_vec(), b"etest".to_vec(), None),
                            ])),
                            "/userinfo" => {
                                let auth = req
                                    .header("Authorization")
                                    .expect("Authorization header missing");
                                assert_eq!(
                                    auth.as_str().split_once(' '),
                                    Some(("Bearer", "test-token")),
                                );
                                json_response(&CoreUserInfoClaims::new(
                                    StandardClaims::new(SubjectIdentifier::new(
                                        "test|subject".into(),
                                    )),
                                    EmptyAdditionalClaims {},
                                ))
                            }
                            p => panic!("Unsupported path requested: `{p}`"),
                        }
                    },
                )
                .await
                .expect("failed to handle OIDC connection");
            })
            .await
    });

    (oidc_addr, oidc_tx, oidc_handle)
}

async fn init_drawbridge(oidc_addr: SocketAddr) -> (u16, Sender<()>, JoinHandle<()>) {
    let srv_lis = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("failed to bind to address");

    let store = tempdir().expect("failed to create temporary store directory");

    let (srv_tx, srv_rx) = channel::<()>();

    let srv_port = srv_lis.local_addr().unwrap().port();

    let srv_handle = spawn(async move {
        let tls = TlsConfig::read(
            include_bytes!("data/server.crt").as_slice(),
            include_bytes!("data/server.key").as_slice(),
            include_bytes!("data/ca.crt").as_slice(),
        )
        .unwrap();
        let app = App::new(
            store.path(),
            tls,
            OidcConfig {
                label: "test-label".into(),
                issuer: format!("http://{oidc_addr}").parse().unwrap(),
                client_id: "test-client_id".into(),
                client_secret: None,
            },
        )
        .await
        .unwrap();
        srv_lis
            .incoming()
            .take_until(srv_rx)
            .for_each_concurrent(None, |stream| async {
                app.handle(stream.expect("failed to initialize stream"))
                    .await
                    .expect("failed to handle stream")
            })
            .await
    });

    (srv_port, srv_tx, srv_handle)
}

pub async fn write_files() -> TempDir {
    let pkg = tempdir().expect("failed to create temporary package directory");

    try_join!(
        write(pkg.path().join("test-file"), "no extension"),
        write(pkg.path().join("test-file.txt"), "text"),
        write(pkg.path().join("test-file.json"), "not valid json"),
        write(pkg.path().join("tEst-file..__.foo.42."), "invalidext"),
        create_dir(pkg.path().join("test-dir-1")),
    )
    .unwrap();

    try_join!(
        write(pkg.path().join("test-dir-1").join("test-file.txt"), "text"),
        write(pkg.path().join("test-dir-1").join("test-file"), "test"),
        create_dir(pkg.path().join("test-dir-1").join("test-subdir-1")),
        create_dir(pkg.path().join("test-dir-1").join("test-subdir-2")),
    )
    .unwrap();

    write(
        pkg.path()
            .join("test-dir-1")
            .join("test-subdir-2")
            .join("test-file"),
        "test",
    )
    .await
    .unwrap();

    pkg
}
