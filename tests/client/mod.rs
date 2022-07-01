// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::net::SocketAddr;
use std::process::Command;
use std::str::from_utf8;

use drawbridge_app::{App, OidcConfig, TlsConfig};
use drawbridge_client::mime::APPLICATION_OCTET_STREAM;
use drawbridge_client::types::{RepositoryConfig, TreePath, UserRecord};
use drawbridge_client::Client;

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
use rustls::{Certificate, RootCertStore};
use tempfile::tempdir;

const TOKEN: &str = "test-token";
const SUBJECT: &str = "test|subject";

/// Just a nice wrapper over `format!` for testing CLI invocations
macro_rules! cmd {
    ($($arg:tt)+) => (
        enarx(format!($($arg)+))
    )
}

#[async_std::test]
async fn full() {
    run(|srv_addr| async move {
        let oidc_cl = init_clients(srv_addr.clone());

let user_name = "testuser".parse().unwrap();
let oidc_user = oidc_cl.user(&user_name);

        // test for failure to get a user that does not exist
        // TODO: make address part of slug
        let out = cmd!("enarx user info --addr {srv_addr} testuser");
        assert_eq!(out.success, false);

let user_record = UserRecord {
    subject: SUBJECT.into(),
};

        // test for success when creating a user with proper credentials
        // enarx user register testuser
        // TODO: test for failure when creating a user withour proper credentials
        assert_eq!(
            oidc_user
                .create(&user_record)
                .expect("failed to create user"),
            true
        );

        // test for failure when creating a user whose subject matches an existing user?
        // enarx user register testuser2
        assert!(oidc_cl
            .user(&format!("{user_name}other").parse().unwrap())
            .create(&user_record)
            .is_err());

        // test for success to get a user that exists
        // enarx user info testuser
        assert_eq!(oidc_user.get().expect("failed to get user"), user_record);

let prv_repo_name = "test-repo-private".parse().unwrap();
let prv_repo_conf = RepositoryConfig { public: false };

let pub_repo_name = "test-repo-public".parse().unwrap();
let pub_repo_conf = RepositoryConfig { public: true };

let oidc_prv_repo = oidc_user.repository(&prv_repo_name);

let oidc_pub_repo = oidc_user.repository(&pub_repo_name);

        // test for failure to get private repo that does not exist
        // enarx repo info testuser/prvrepo
        assert!(oidc_prv_repo.tags().is_err());

        // test for failure to get public repo that does not exist
        // enarx repo info testuser/pubrepo
        assert!(oidc_pub_repo.tags().is_err());

        // test for success to register private repo
        // enarx repo register --private testuser/prvrepo
        assert_eq!(
            oidc_prv_repo
                .create(&prv_repo_conf)
                .expect("failed to create repository"),
            true
        );

        // test for success to register public repo
        // enarx repo register --public testuser/pubrepo
        assert_eq!(
            oidc_pub_repo
                .create(&pub_repo_conf)
                .expect("failed to create repository"),
            true
        );

        // test for tags associated with an empty private repo
        // enarx repo info testuser/prvrepo
        assert_eq!(oidc_prv_repo.tags().expect("failed to get tags"), vec![]);

        // test for tags asoociated with an empty public repo
        // enarx repo info testuser/pubrepo
        assert_eq!(oidc_pub_repo.tags().expect("failed to get tags"), vec![]);

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

        let tag_name = "0.1.0".parse().unwrap();

        let oidc_prv_tag = oidc_prv_repo.tag(&tag_name);

        let oidc_pub_tag = oidc_pub_repo.tag(&tag_name);

        // test for failure to get a private tag that doesn't exist
        // enarx package info testuser/prvrepo/0.1.0
        assert!(oidc_prv_tag.get().is_err());

        // test for failure to get a public tag that doesn't exist
        // enarx package info testuser/pubrepo/0.1.0
        assert!(oidc_pub_tag.get().is_err());

        let (prv_tag_created, prv_tree_created) = oidc_prv_tag
            .create_from_path_unsigned(pkg.path())
            .expect("failed to create a tag and upload the tree");

        // test for success to publish a private package
        // enarx package publish testuser/pubrepo/0.1.0 some_path
        assert!(prv_tag_created);

        // test the contents of a published private package
        // enarx package fetch testuser/pubrepo/0.1.0
        assert_eq!(
            prv_tree_created.clone().into_iter().collect::<Vec<_>>(),
            vec![
                (TreePath::ROOT, true),
                ("tEst-file..__.foo.42.".parse().unwrap(), true),
                ("test-dir-1".parse().unwrap(), true),
                ("test-dir-1/test-file".parse().unwrap(), true),
                ("test-dir-1/test-file.txt".parse().unwrap(), true),
                ("test-dir-1/test-subdir-1".parse().unwrap(), true),
                ("test-dir-1/test-subdir-2".parse().unwrap(), true),
                ("test-dir-1/test-subdir-2/test-file".parse().unwrap(), true),
                ("test-file".parse().unwrap(), true),
                ("test-file.json".parse().unwrap(), true),
                ("test-file.txt".parse().unwrap(), true),
            ]
        );

        // test for success to publish a public package
        assert_eq!(
            oidc_pub_tag
                .create_from_path_unsigned(pkg.path())
                .expect("failed to create a tag and upload the tree"),
            (prv_tag_created, prv_tree_created)
        );

        // test for tags associated with a non-empty private repo
        assert_eq!(
            oidc_prv_repo.tags().expect("failed to get tags"),
            vec![tag_name.clone()]
        );

        // test for tags associated with a non-empty public repo
        assert_eq!(
            oidc_pub_repo.tags().expect("failed to get tags"),
            vec![tag_name.clone()]
        );

        let file_name = "test-file.txt".parse().unwrap();

        let oidc_prv_file = oidc_prv_tag.path(&file_name);

        let oidc_pub_file = oidc_pub_tag.path(&file_name);

        // test for retrival of a single file from a tree in a private package
        assert_eq!(
            oidc_prv_file.get_string().expect("failed to get file"),
            (APPLICATION_OCTET_STREAM, "text".into())
        );

        // test for retrieval of a single file from a tree in a public package
        assert_eq!(
            oidc_pub_file.get_string().expect("failed to get file"),
            (APPLICATION_OCTET_STREAM, "text".into())
        );
    }).await;
}

fn enarx(args: String) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_enarx"));

    for arg in args.split(' ').skip(1) {
        cmd.arg(arg);
    }

    let out = cmd.output().expect("failed to execute `enarx`");

    Output {
        success: out.status.success(),
        output: from_utf8(&out.stdout).unwrap().to_string(),
    }
}

struct Output {
    success: bool,
    output: String,
}


async fn run<F, R>(commands: F)
where F: FnOnce(String)-> R, R: Future<Output=()> {
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
                                assert_eq!(auth.as_str().split_once(' '), Some(("Bearer", TOKEN)),);
                                json_response(&CoreUserInfoClaims::new(
                                    StandardClaims::new(SubjectIdentifier::new(SUBJECT.into())),
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

fn init_clients(srv_addr: String) -> Client {
    let cl = Client::builder(srv_addr.parse().unwrap())
        .roots({
            let mut roots = RootCertStore::empty();
            rustls_pemfile::certs(&mut std::io::BufReader::new(
                include_bytes!("data/ca.crt").as_slice(),
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
