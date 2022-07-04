// SPDX-License-Identifier: Apache-2.0

mod util;

use util::{enarx, run, write_files};

/// Just a nice wrapper over `format!` for testing CLI invocations
macro_rules! cmd {
    ($($arg:tt)+) => (
        enarx(format!($($arg)+))
    )
}

#[async_std::test]
async fn full() {
    run(|srv_addr| async move {
        // test for failure to get a user that does not exist
        let cmd = cmd!("enarx user info {srv_addr}+testuser");
        assert_eq!(cmd.success, false);

        // test for success when creating a user with proper credentials
        // TODO: test for failure when creating a user withour proper credentials
        let cmd = cmd!("enarx user register {srv_addr}+testuser");
        assert_eq!(cmd.success, true);

        // test for failure when creating a user whose subject matches an existing user
        let cmd = cmd!("enarx user register {srv_addr}+testuser2");
        assert_eq!(cmd.success, false);

        // test for success to get a user that exists
        let cmd = cmd!("enarx user info {srv_addr}+testuser");
        assert_eq!(cmd.success, true);

        // test for failure to get repo that does not exist
        let cmd = cmd!("enarx repo info {srv_addr}+testuser/test-repo-private");
        assert_eq!(cmd.success, false);

        // test for success to register private repo
        let cmd = cmd!("enarx repo register {srv_addr}+testuser/test-repo-private");
        assert_eq!(cmd.success, true);

        // test for success to register public repo
        let cmd = cmd!("enarx repo register {srv_addr}+testuser/test-repo-public");
        assert_eq!(cmd.success, true);

        // test for tags associated with an empty private repo
        let cmd = cmd!("enarx repo info {srv_addr}+testuser/test-repo-private");
        assert_eq!(cmd.output, "");

        // test for tags asoociated with an empty public repo
        let cmd = cmd!("enarx repo info {srv_addr}+testuser/test-repo-public");
        assert_eq!(cmd.output, "");

        // TODO: just ship test files?
        let pkg = write_files().await;

        // test for failure to get a private package that doesn't exist
        let cmd = cmd!("enarx package info {srv_addr}+testuser/test-repo-private:0.1.0");
        assert_eq!(cmd.success, false);

        // TODO: success for above

        // test for success to publish a private package
        let cmd = cmd!(
            "enarx package publish {srv_addr}+testuser/test-repo-private:0.1.0 {}",
            pkg.path().display()
        );
        assert_eq!(cmd.success, true);

        // test for success to publish a public package
        let cmd = cmd!(
            "enarx package publish {srv_addr}+testuser/test-repo-public:0.1.0 {}",
            pkg.path().display()
        );
        assert_eq!(cmd.success, true);

        // test for tags associated with a non-empty private repo
        let cmd = cmd!("enarx repo info {srv_addr}+testuser/test-repo-private");
        assert_eq!(cmd.output, "0.1.0\n");

        // test for tags associated with a non-empty public repo
        let cmd = cmd!("enarx repo info {srv_addr}+testuser/test-repo-public");
        assert_eq!(cmd.output, "0.1.0\n");
    })
    .await;
}
