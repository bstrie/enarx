// SPDX-License-Identifier: Apache-2.0

mod util;

use util::{enarx, run};

/// Just a nice wrapper over `format!` for testing CLI invocations
macro_rules! cmd {
    ($($arg:tt)+) => (
        enarx(format!($($arg)+)).await
    )
}

#[async_std::test]
async fn user_register() {
    run(|oidc_addr, db_addr| async move {
        let cmd = cmd!(
            "enarx user register \
            --oidc-domain https://{oidc_addr} \
            --oidc-client-id test-client-id \
            {db_addr}+testuser"
        );
        println!("{}", cmd.err);
        assert_eq!(cmd.success, true);
    })
    .await;
}

#[async_std::test]
async fn full() {
    run(|_oidc_addr, db_addr| async move {
        // test for failure when looking up a user that does not exist
        let cmd = cmd!("enarx user info {db_addr}+testuser");
        assert_eq!(cmd.success, false);

        // test for success when creating a user with proper credentials
        // TODO: test for failure when creating a user without proper credentials
        let cmd = cmd!("enarx user register {db_addr}+testuser");
        assert_eq!(cmd.success, true);

        // test for failure when creating a user whose subject matches an existing user
        let cmd = cmd!("enarx user register {db_addr}+testuser2");
        assert_eq!(cmd.success, false);

        // test for success when looking up a user that exists
        let cmd = cmd!("enarx user info {db_addr}+testuser");
        assert_eq!(cmd.success, true);

        // test for failure when looking up a repo that does not exist
        let cmd = cmd!("enarx repo info {db_addr}+testuser/privaterepo");
        assert_eq!(cmd.success, false);

        // test for success when registering a private repo
        let cmd = cmd!("enarx repo register {db_addr}+testuser/privaterepo");
        assert_eq!(cmd.success, true);

        // test for success when registering a public repo
        let cmd = cmd!("enarx repo register --public {db_addr}+testuser/publicrepo");
        assert_eq!(cmd.success, true);

        // test for success when fetching tags from empty private repo
        let cmd = cmd!("enarx repo info {db_addr}+testuser/privaterepo");
        assert_eq!(cmd.success, true);
        assert_eq!(cmd.output, "");

        // test for success when fetching tags from empty public repo
        let cmd = cmd!("enarx repo info {db_addr}+testuser/publicrepo");
        assert_eq!(cmd.success, true);
        assert_eq!(cmd.output, "");

        // test for failure when looking up a package that does nto exist
        let cmd = cmd!("enarx package info {db_addr}+testuser/privaterepo:0.1.0");
        assert_eq!(cmd.success, false);

        // test for success when publishing a private package
        let cmd =
            cmd!("enarx package publish {db_addr}+testuser/privaterepo:0.1.0 tests/client/package");
        assert_eq!(cmd.success, true);

        // test for success when publishing a public package
        let cmd =
            cmd!("enarx package publish {db_addr}+testuser/publicrepo:0.1.0 tests/client/package");
        assert_eq!(cmd.success, true);

        // test for success when looking up a private package that exists
        let cmd = cmd!("enarx package info {db_addr}+testuser/privaterepo:0.1.0");
        assert_eq!(cmd.success, true);

        // test for success when looking up a public package that exists
        let cmd = cmd!("enarx package info {db_addr}+testuser/publicrepo:0.1.0");
        assert_eq!(cmd.success, true);

        // test for success when fetching tags from a non-empty private repo
        let cmd = cmd!("enarx repo info {db_addr}+testuser/privaterepo");
        assert_eq!(cmd.output, "0.1.0\n");

        // test for success when fetching tags from a non-empty public repo
        let cmd = cmd!("enarx repo info {db_addr}+testuser/publicrepo");
        assert_eq!(cmd.output, "0.1.0\n");

        // TODO: fetch package
    })
    .await;
}
