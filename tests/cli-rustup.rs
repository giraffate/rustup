//! Test cases for new rustup UI

pub mod mock;

use crate::mock::clitools::{
    self, expect_err, expect_ok, expect_ok_ex, expect_stderr_ok, expect_stdout_ok, run,
    set_current_dist_date, this_host_triple, Config, Scenario,
};
use rustup::utils::raw;
use std::env::consts::EXE_SUFFIX;
use std::fs;
use std::path::MAIN_SEPARATOR;

macro_rules! for_host {
    ($s: expr) => {
        &format!($s, this_host_triple())
    };
}

macro_rules! for_host_and_home {
    ($config:ident, $s: expr) => {
        &format!($s, this_host_triple(), $config.rustupdir.display())
    };
}

pub fn setup(f: &dyn Fn(&Config)) {
    clitools::setup(Scenario::ArchivesV2, &|config| {
        f(config);
    });
}

#[test]
fn rustup_stable() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable", "--no-self-update"]);
        set_current_dist_date(config, "2015-01-02");
        expect_ok_ex(
            config,
            &["rustup", "update", "--no-self-update"],
            for_host!(
                r"
  stable-{0} updated - 1.1.0 (hash-stable-1.1.0)

"
            ),
            for_host!(
                r"info: syncing channel updates for 'stable-{0}'
info: latest update on 2015-01-02, rust version 1.1.0 (hash-stable-1.1.0)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: cleaning up downloads & tmp directories
"
            ),
        );
    });
}

#[test]
fn rustup_stable_quiet() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(
            config,
            &["rustup", "--quiet", "update", "stable", "--no-self-update"],
        );
        set_current_dist_date(config, "2015-01-02");
        expect_ok_ex(
            config,
            &["rustup", "--quiet", "update", "--no-self-update"],
            for_host!(
                r"
  stable-{0} updated - 1.1.0 (hash-stable-1.1.0)

"
            ),
            for_host!(
                r"info: syncing channel updates for 'stable-{0}'
info: latest update on 2015-01-02, rust version 1.1.0 (hash-stable-1.1.0)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: cleaning up downloads & tmp directories
"
            ),
        );
    });
}

#[test]
fn rustup_stable_no_change() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable", "--no-self-update"]);
        expect_ok_ex(
            config,
            &["rustup", "update", "--no-self-update"],
            for_host!(
                r"
  stable-{0} unchanged - 1.0.0 (hash-stable-1.0.0)

"
            ),
            for_host!(
                r"info: syncing channel updates for 'stable-{0}'
info: cleaning up downloads & tmp directories
"
            ),
        );
    });
}

#[test]
fn rustup_all_channels() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable", "--no-self-update"]);
        expect_ok(config, &["rustup", "update", "beta", "--no-self-update"]);
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        set_current_dist_date(config, "2015-01-02");
        expect_ok_ex(
            config,
            &["rustup", "update", "--no-self-update"],
            for_host!(
                r"
   stable-{0} updated - 1.1.0 (hash-stable-1.1.0)
     beta-{0} updated - 1.2.0 (hash-beta-1.2.0)
  nightly-{0} updated - 1.3.0 (hash-nightly-2)

"
            ),
            for_host!(
                r"info: syncing channel updates for 'stable-{0}'
info: latest update on 2015-01-02, rust version 1.1.0 (hash-stable-1.1.0)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: syncing channel updates for 'beta-{0}'
info: latest update on 2015-01-02, rust version 1.2.0 (hash-beta-1.2.0)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: syncing channel updates for 'nightly-{0}'
info: latest update on 2015-01-02, rust version 1.3.0 (hash-nightly-2)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: cleaning up downloads & tmp directories
"
            ),
        );
    })
}

#[test]
fn rustup_some_channels_up_to_date() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable", "--no-self-update"]);
        expect_ok(config, &["rustup", "update", "beta", "--no-self-update"]);
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        set_current_dist_date(config, "2015-01-02");
        expect_ok(config, &["rustup", "update", "beta", "--no-self-update"]);
        expect_ok_ex(
            config,
            &["rustup", "update", "--no-self-update"],
            for_host!(
                r"
   stable-{0} updated - 1.1.0 (hash-stable-1.1.0)
   beta-{0} unchanged - 1.2.0 (hash-beta-1.2.0)
  nightly-{0} updated - 1.3.0 (hash-nightly-2)

"
            ),
            for_host!(
                r"info: syncing channel updates for 'stable-{0}'
info: latest update on 2015-01-02, rust version 1.1.0 (hash-stable-1.1.0)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: syncing channel updates for 'beta-{0}'
info: syncing channel updates for 'nightly-{0}'
info: latest update on 2015-01-02, rust version 1.3.0 (hash-nightly-2)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: removing previous version of component 'cargo'
info: removing previous version of component 'rust-docs'
info: removing previous version of component 'rust-std'
info: removing previous version of component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: cleaning up downloads & tmp directories
"
            ),
        );
    })
}

#[test]
fn rustup_no_channels() {
    setup(&|config| {
        expect_ok(config, &["rustup", "update", "stable", "--no-self-update"]);
        expect_ok(config, &["rustup", "toolchain", "remove", "stable"]);
        expect_ok_ex(
            config,
            &["rustup", "update", "--no-self-update"],
            r"",
            r"info: no updatable toolchains installed
info: cleaning up downloads & tmp directories
",
        );
    })
}

#[test]
fn default() {
    setup(&|config| {
        expect_ok_ex(
            config,
            &["rustup", "default", "nightly"],
            for_host!(
                r"
  nightly-{0} installed - 1.3.0 (hash-nightly-2)

"
            ),
            for_host!(
                r"info: syncing channel updates for 'nightly-{0}'
info: latest update on 2015-01-02, rust version 1.3.0 (hash-nightly-2)
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: default toolchain set to 'nightly-{0}'
"
            ),
        );
    });
}

#[test]
fn default_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(config, &["rustup", "override", "set", "nightly"]);
        expect_stderr_ok(
            config,
            &["rustup", "default", "stable"],
            for_host!(
                r"info: using existing install for 'stable-{0}'
info: default toolchain set to 'stable-{0}'
info: note that the toolchain 'nightly-{0}' is currently in use (directory override for"
            ),
        );
    });
}

#[test]
fn rustup_xz() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_stderr_ok(
            config,
            &[
                "rustup",
                "--verbose",
                "update",
                "nightly",
                "--no-self-update",
            ],
            for_host!(r"dist/2015-01-01/rust-std-nightly-{0}.tar.xz"),
        );
    });
}

#[test]
fn add_target() {
    setup(&|config| {
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH1
        );
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "target", "add", clitools::CROSS_ARCH1]);
        assert!(config.rustupdir.join(path).exists());
    });
}

#[test]
fn remove_target() {
    setup(&|config| {
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH1
        );
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "target", "add", clitools::CROSS_ARCH1]);
        assert!(config.rustupdir.join(&path).exists());
        expect_ok(
            config,
            &["rustup", "target", "remove", clitools::CROSS_ARCH1],
        );
        assert!(!config.rustupdir.join(&path).exists());
    });
}

#[test]
fn add_remove_multiple_targets() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(
            config,
            &[
                "rustup",
                "target",
                "add",
                clitools::CROSS_ARCH1,
                clitools::CROSS_ARCH2,
            ],
        );
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH1
        );
        assert!(config.rustupdir.join(path).exists());
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH2
        );
        assert!(config.rustupdir.join(path).exists());

        expect_ok(
            config,
            &[
                "rustup",
                "target",
                "remove",
                clitools::CROSS_ARCH1,
                clitools::CROSS_ARCH2,
            ],
        );
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH1
        );
        assert!(!config.rustupdir.join(path).exists());
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH2
        );
        assert!(!config.rustupdir.join(path).exists());
    });
}

#[test]
fn list_targets() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_stdout_ok(config, &["rustup", "target", "list"], clitools::CROSS_ARCH1);
    });
}

#[test]
fn list_installed_targets() {
    setup(&|config| {
        let trip = this_host_triple();

        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_stdout_ok(config, &["rustup", "target", "list", "--installed"], &trip);
    });
}

#[test]
fn add_target_explicit() {
    setup(&|config| {
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH1
        );
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        expect_ok(
            config,
            &[
                "rustup",
                "target",
                "add",
                "--toolchain",
                "nightly",
                clitools::CROSS_ARCH1,
            ],
        );
        assert!(config.rustupdir.join(path).exists());
    });
}

#[test]
fn remove_target_explicit() {
    setup(&|config| {
        let path = format!(
            "toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
            &this_host_triple(),
            clitools::CROSS_ARCH1
        );
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        expect_ok(
            config,
            &[
                "rustup",
                "target",
                "add",
                "--toolchain",
                "nightly",
                clitools::CROSS_ARCH1,
            ],
        );
        assert!(config.rustupdir.join(&path).exists());
        expect_ok(
            config,
            &[
                "rustup",
                "target",
                "remove",
                "--toolchain",
                "nightly",
                clitools::CROSS_ARCH1,
            ],
        );
        assert!(!config.rustupdir.join(&path).exists());
    });
}

#[test]
fn list_targets_explicit() {
    setup(&|config| {
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        expect_stdout_ok(
            config,
            &["rustup", "target", "list", "--toolchain", "nightly"],
            clitools::CROSS_ARCH1,
        );
    });
}

#[test]
fn link() {
    setup(&|config| {
        let path = config.customdir.join("custom-1");
        let path = path.to_string_lossy();
        expect_ok(config, &["rustup", "toolchain", "link", "custom", &path]);
        expect_ok(config, &["rustup", "default", "custom"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-c-1");
        expect_stdout_ok(config, &["rustup", "show"], "custom (default)");
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_stdout_ok(config, &["rustup", "show"], "custom");
    });
}

// Issue #809. When we call the fallback cargo, when it in turn invokes
// "rustc", that rustc should actually be the rustup proxy, not the toolchain rustc.
// That way the proxy can pick the correct toolchain.
#[test]
fn fallback_cargo_calls_correct_rustc() {
    setup(&|config| {
        // Hm, this is the _only_ test that assumes that toolchain proxies
        // exist in CARGO_HOME. Adding that proxy here.
        let rustup_path = config.exedir.join(format!("rustup{}", EXE_SUFFIX));
        let cargo_bin_path = config.cargodir.join("bin");
        fs::create_dir_all(&cargo_bin_path).unwrap();
        let rustc_path = cargo_bin_path.join(format!("rustc{}", EXE_SUFFIX));
        fs::hard_link(&rustup_path, &rustc_path).unwrap();

        // Install a custom toolchain and a nightly toolchain for the cargo fallback
        let path = config.customdir.join("custom-1");
        let path = path.to_string_lossy();
        expect_ok(config, &["rustup", "toolchain", "link", "custom", &path]);
        expect_ok(config, &["rustup", "default", "custom"]);
        expect_ok(config, &["rustup", "update", "nightly", "--no-self-update"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-c-1");
        expect_stdout_ok(config, &["cargo", "--version"], "hash-nightly-2");

        assert!(rustc_path.exists());

        // Here --call-rustc tells the mock cargo bin to exec `rustc --version`.
        // We should be ultimately calling the custom rustc, according to the
        // RUSTUP_TOOLCHAIN variable set by the original "cargo" proxy, and
        // interpreted by the nested "rustc" proxy.
        expect_stdout_ok(config, &["cargo", "--call-rustc"], "hash-c-1");
    });
}

#[test]
fn show_home() {
    setup(&|config| {
        expect_ok_ex(
            config,
            &["rustup", "show", "home"],
            &format!(
                r"{}
",
                config.rustupdir.display()
            ),
            r"",
        );
    });
}

#[test]
fn show_toolchain_none() {
    setup(&|config| {
        expect_ok_ex(
            config,
            &["rustup", "show"],
            &for_host_and_home!(
                config,
                r"Default host: {0}
rustup home:  {1}

no active toolchain
"
            ),
            r"",
        );
    });
}

#[test]
fn show_toolchain_default() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok_ex(
            config,
            &["rustup", "show"],
            for_host_and_home!(
                config,
                r"Default host: {0}
rustup home:  {1}

nightly-{0} (default)
1.3.0 (hash-nightly-2)
"
            ),
            r"",
        );
    });
}

#[test]
fn show_multiple_toolchains() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "update", "stable", "--no-self-update"]);
        expect_ok_ex(
            config,
            &["rustup", "show"],
            for_host_and_home!(
                config,
                r"Default host: {0}
rustup home:  {1}

installed toolchains
--------------------

stable-{0}
nightly-{0} (default)

active toolchain
----------------

nightly-{0} (default)
1.3.0 (hash-nightly-2)

"
            ),
            r"",
        );
    });
}

#[test]
fn show_multiple_targets() {
    // Using the MULTI_ARCH1 target doesn't work on i686 linux
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86") {
        return;
    }

    clitools::setup(Scenario::MultiHost, &|config| {
        expect_ok(
            config,
            &[
                "rustup",
                "default",
                &format!("nightly-{}", clitools::MULTI_ARCH1),
            ],
        );
        expect_ok(config, &["rustup", "target", "add", clitools::CROSS_ARCH2]);
        expect_ok_ex(
            config,
            &["rustup", "show"],
            &format!(
                r"Default host: {2}
rustup home:  {3}

installed targets for active toolchain
--------------------------------------

{1}
{0}

active toolchain
----------------

nightly-{0} (default)
1.3.0 (xxxx-nightly-2)

",
                clitools::MULTI_ARCH1,
                clitools::CROSS_ARCH2,
                this_host_triple(),
                config.rustupdir.display()
            ),
            r"",
        );
    });
}

#[test]
fn show_multiple_toolchains_and_targets() {
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86") {
        return;
    }

    clitools::setup(Scenario::MultiHost, &|config| {
        expect_ok(
            config,
            &[
                "rustup",
                "default",
                &format!("nightly-{}", clitools::MULTI_ARCH1),
            ],
        );
        expect_ok(config, &["rustup", "target", "add", clitools::CROSS_ARCH2]);
        expect_ok(
            config,
            &[
                "rustup",
                "update",
                &format!("stable-{}", clitools::MULTI_ARCH1),
                "--no-self-update",
            ],
        );
        expect_ok_ex(
            config,
            &["rustup", "show"],
            &format!(
                r"Default host: {2}
rustup home:  {3}

installed toolchains
--------------------

stable-{0}
nightly-{0} (default)

installed targets for active toolchain
--------------------------------------

{1}
{0}

active toolchain
----------------

nightly-{0} (default)
1.3.0 (xxxx-nightly-2)

",
                clitools::MULTI_ARCH1,
                clitools::CROSS_ARCH2,
                this_host_triple(),
                config.rustupdir.display()
            ),
            r"",
        );
    });
}

#[test]
fn list_default_toolchain() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok_ex(
            config,
            &["rustup", "toolchain", "list"],
            for_host!(
                r"nightly-{0} (default)
"
            ),
            r"",
        );
    });
}

#[test]
#[ignore = "FIXME: Windows shows UNC paths"]
fn show_toolchain_override() {
    setup(&|config| {
        let cwd = config.current_dir();
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok_ex(
            config,
            &["rustup", "show"],
            &format!(
                r"Default host: {0}
rustup home:  {1}

nightly-{0} (directory override for '{2}')
1.3.0 (hash-nightly-2)
",
                this_host_triple(),
                config.rustupdir.display(),
                cwd.display(),
            ),
            r"",
        );
    });
}

#[test]
#[ignore = "FIXME: Windows shows UNC paths"]
fn show_toolchain_toolchain_file_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(config, &["rustup", "toolchain", "install", "nightly"]);

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");

        raw::write_file(&toolchain_file, "nightly").unwrap();

        expect_ok_ex(
            config,
            &["rustup", "show"],
            &format!(
                r"Default host: {0}
rustup home:  {1}

installed toolchains
--------------------

stable-{0} (default)
nightly-{0}

active toolchain
----------------

nightly-{0} (overridden by '{2}')
1.3.0 (hash-nightly-2)

",
                this_host_triple(),
                config.rustupdir.display(),
                toolchain_file.display()
            ),
            r"",
        );
    });
}

#[test]
#[ignore = "FIXME: Windows shows UNC paths"]
fn show_toolchain_version_nested_file_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(config, &["rustup", "toolchain", "install", "nightly"]);

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");

        raw::write_file(&toolchain_file, "nightly").unwrap();

        let subdir = cwd.join("foo");

        fs::create_dir_all(&subdir).unwrap();
        config.change_dir(&subdir, &|| {
            expect_ok_ex(
                config,
                &["rustup", "show"],
                &format!(
                    r"Default host: {0}

installed toolchains
--------------------

stable-{0} (default)
nightly-{0}

active toolchain
----------------

nightly-{0} (overridden by '{1}')
1.3.0 (hash-nightly-2)

",
                    this_host_triple(),
                    toolchain_file.display()
                ),
                r"",
            );
        });
    });
}

#[test]
#[ignore = "FIXME: Windows shows UNC paths"]
fn show_toolchain_toolchain_file_override_not_installed() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");

        raw::write_file(&toolchain_file, "nightly").unwrap();

        // I'm not sure this should really be erroring when the toolchain
        // is not installed; just capturing the behavior.
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(!out.status.success());
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.starts_with("error: override toolchain 'nightly' is not installed"));
        assert!(stderr.contains(&format!(
            "the toolchain file at '{}' specifies an uninstalled toolchain",
            toolchain_file.display()
        )));
    });
}

#[test]
fn show_toolchain_override_not_installed() {
    setup(&|config| {
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok(config, &["rustup", "toolchain", "remove", "nightly"]);
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(!stdout.contains("not a directory"));
        assert!(!stdout.contains("is not installed"));
        assert!(stderr.contains("info: installing component 'rustc'"));
    });
}

#[test]
fn override_set_unset_with_path() {
    setup(&|config| {
        let cwd = fs::canonicalize(config.current_dir()).unwrap();
        let mut cwd_str = cwd.to_str().unwrap();

        if cfg!(windows) {
            cwd_str = &cwd_str[4..];
        }

        config.change_dir(&config.emptydir, &|| {
            expect_ok(
                config,
                &["rustup", "override", "set", "nightly", "--path", cwd_str],
            );
        });
        expect_ok_ex(
            config,
            &["rustup", "override", "list"],
            &format!("{}\tnightly-{}\n", cwd_str, this_host_triple()),
            r"",
        );
        config.change_dir(&config.emptydir, &|| {
            expect_ok(config, &["rustup", "override", "unset", "--path", cwd_str]);
        });
        expect_ok_ex(
            config,
            &["rustup", "override", "list"],
            &"no overrides\n",
            r"",
        );
    });
}

#[test]
fn show_toolchain_env() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        cmd.env("RUSTUP_TOOLCHAIN", "nightly");
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert_eq!(
            &stdout,
            for_host_and_home!(
                config,
                r"Default host: {0}
rustup home:  {1}

nightly-{0} (environment override by RUSTUP_TOOLCHAIN)
1.3.0 (hash-nightly-2)
"
            )
        );
    });
}

#[test]
fn show_toolchain_env_not_installed() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        cmd.env("RUSTUP_TOOLCHAIN", "nightly");
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(!stdout.contains("is not installed"));
        assert!(stderr.contains("info: installing component 'rustc'"));
    });
}

#[test]
fn show_active_toolchain() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok_ex(
            config,
            &["rustup", "show", "active-toolchain"],
            for_host!(
                r"nightly-{0} (default)
"
            ),
            r"",
        );
    });
}

#[test]
fn show_active_toolchain_with_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "override", "set", "stable"]);
        expect_stdout_ok(
            config,
            &["rustup", "show", "active-toolchain"],
            for_host!("stable-{0} (directory override for"),
        );
    });
}

#[test]
fn show_active_toolchain_none() {
    setup(&|config| {
        expect_ok_ex(config, &["rustup", "show", "active-toolchain"], r"", r"");
    });
}

#[test]
fn show_profile() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_stdout_ok(config, &["rustup", "show", "profile"], "default");

        // Check we get the same thing after we add or remove a component.
        expect_ok(config, &["rustup", "component", "add", "rust-src"]);
        expect_stdout_ok(config, &["rustup", "show", "profile"], "default");
        expect_ok(config, &["rustup", "component", "remove", "rustc"]);
        expect_stdout_ok(config, &["rustup", "show", "profile"], "default");
    });
}

// #846
#[test]
fn set_default_host() {
    setup(&|config| {
        expect_ok(
            config,
            &["rustup", "set", "default-host", &this_host_triple()],
        );
        expect_stdout_ok(config, &["rustup", "show"], for_host!("Default host: {0}"));
    });
}

// #846
#[test]
fn set_default_host_invalid_triple() {
    setup(&|config| {
        expect_err(
            config,
            &["rustup", "set", "default-host", "foo"],
            "error: Provided host 'foo' couldn't be converted to partial triple",
        );
    });
}

// #745
#[test]
fn set_default_host_invalid_triple_valid_partial() {
    setup(&|config| {
        expect_err(
            config,
            &["rustup", "set", "default-host", "x86_64-msvc"],
            "error: Provided host 'x86_64-msvc' did not specify an operating system",
        );
    });
}

// #422
#[test]
fn update_doesnt_update_non_tracking_channels() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(
            config,
            &["rustup", "update", "nightly-2015-01-01", "--no-self-update"],
        );
        let mut cmd = clitools::cmd(config, "rustup", &["update"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(!stderr.contains(for_host!(
            "syncing channel updates for 'nightly-2015-01-01-{}'"
        )));
    });
}

#[test]
fn toolchain_install_is_like_update() {
    setup(&|config| {
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );
        expect_stdout_ok(
            config,
            &["rustup", "run", "nightly", "rustc", "--version"],
            "hash-nightly-2",
        );
    });
}

#[test]
fn toolchain_install_is_like_update_quiet() {
    setup(&|config| {
        expect_ok(
            config,
            &[
                "rustup",
                "--quiet",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );
        expect_stdout_ok(
            config,
            &["rustup", "run", "nightly", "rustc", "--version"],
            "hash-nightly-2",
        );
    });
}

#[test]
fn toolchain_install_is_like_update_except_that_bare_install_is_an_error() {
    setup(&|config| {
        expect_err(
            config,
            &["rustup", "toolchain", "install", "--no-self-update"],
            "arguments were not provided",
        );
    });
}

#[test]
fn toolchain_update_is_like_update() {
    setup(&|config| {
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "update",
                "nightly",
                "--no-self-update",
            ],
        );
        expect_stdout_ok(
            config,
            &["rustup", "run", "nightly", "rustc", "--version"],
            "hash-nightly-2",
        );
    });
}

#[test]
fn toolchain_uninstall_is_like_uninstall() {
    setup(&|config| {
        expect_ok(config, &["rustup", "uninstall", "nightly"]);
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(!stdout.contains(for_host!("'nightly-2015-01-01-{}'")));
    });
}

#[test]
fn toolchain_update_is_like_update_except_that_bare_install_is_an_error() {
    setup(&|config| {
        expect_err(
            config,
            &["rustup", "toolchain", "update"],
            "arguments were not provided",
        );
    });
}

#[test]
fn proxy_toolchain_shorthand() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "update",
                "nightly",
                "--no-self-update",
            ],
        );
        expect_stdout_ok(config, &["rustc", "--version"], "hash-stable-1.1.0");
        expect_stdout_ok(
            config,
            &["rustc", "+stable", "--version"],
            "hash-stable-1.1.0",
        );
        expect_stdout_ok(
            config,
            &["rustc", "+nightly", "--version"],
            "hash-nightly-2",
        );
    });
}

#[test]
fn add_component() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(config, &["rustup", "component", "add", "rust-src"]);
        let path = format!(
            "toolchains/stable-{}/lib/rustlib/src/rust-src/foo.rs",
            this_host_triple()
        );
        let path = config.rustupdir.join(path);
        assert!(path.exists());
    });
}

#[test]
fn remove_component() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(config, &["rustup", "component", "add", "rust-src"]);
        let path = format!(
            "toolchains/stable-{}/lib/rustlib/src/rust-src/foo.rs",
            this_host_triple()
        );
        let path = config.rustupdir.join(path);
        assert!(path.exists());
        expect_ok(config, &["rustup", "component", "remove", "rust-src"]);
        assert!(!path.parent().unwrap().exists());
    });
}

#[test]
fn add_remove_multiple_components() {
    let files = [
        "lib/rustlib/src/rust-src/foo.rs".to_owned(),
        format!("lib/rustlib/{}/analysis/libfoo.json", this_host_triple()),
    ];

    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(
            config,
            &["rustup", "component", "add", "rust-src", "rust-analysis"],
        );
        for file in &files {
            let path = format!("toolchains/nightly-{}/{}", this_host_triple(), file);
            let path = config.rustupdir.join(path);
            assert!(path.exists());
        }
        expect_ok(
            config,
            &["rustup", "component", "remove", "rust-src", "rust-analysis"],
        );
        for file in &files {
            let path = format!("toolchains/nightly-{}/{}", this_host_triple(), file);
            let path = config.rustupdir.join(path);
            assert!(!path.parent().unwrap().exists());
        }
    });
}

#[test]
fn file_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        expect_stdout_ok(config, &["rustc", "--version"], "hash-stable-1.1.0");

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        expect_stdout_ok(config, &["rustc", "--version"], "hash-nightly-2");
    });
}

#[test]
fn file_override_subdir() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        expect_stdout_ok(config, &["rustc", "--version"], "hash-stable-1.1.0");

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        let subdir = cwd.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        config.change_dir(&subdir, &|| {
            expect_stdout_ok(config, &["rustc", "--version"], "hash-nightly-2");
        });
    });
}

#[test]
fn file_override_with_archive() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly-2015-01-01",
                "--no-self-update",
            ],
        );

        expect_stdout_ok(config, &["rustc", "--version"], "hash-stable-1.1.0");

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly-2015-01-01").unwrap();

        expect_stdout_ok(config, &["rustc", "--version"], "hash-nightly-1");
    });
}

#[test]
fn directory_override_beats_file_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &["rustup", "toolchain", "install", "beta", "--no-self-update"],
        );
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        expect_ok(config, &["rustup", "override", "set", "beta"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-beta-1.2.0");

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        expect_stdout_ok(config, &["rustc", "--version"], "hash-beta-1.2.0");
    });
}

#[test]
fn close_file_override_beats_far_directory_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &["rustup", "toolchain", "install", "beta", "--no-self-update"],
        );
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        expect_ok(config, &["rustup", "override", "set", "beta"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-beta-1.2.0");

        let cwd = config.current_dir();

        let subdir = cwd.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        let toolchain_file = subdir.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        config.change_dir(&subdir, &|| {
            expect_stdout_ok(config, &["rustc", "--version"], "hash-nightly-2");
        });
    });
}

#[test]
fn directory_override_doesnt_need_to_exist_unless_it_is_selected() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &["rustup", "toolchain", "install", "beta", "--no-self-update"],
        );
        // not installing nightly

        expect_ok(config, &["rustup", "override", "set", "beta"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-beta-1.2.0");

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        expect_stdout_ok(config, &["rustc", "--version"], "hash-beta-1.2.0");
    });
}

#[test]
fn env_override_beats_file_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &["rustup", "toolchain", "install", "beta", "--no-self-update"],
        );
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        let mut cmd = clitools::cmd(config, "rustc", &["--version"]);
        clitools::env(config, &mut cmd);
        cmd.env("RUSTUP_TOOLCHAIN", "beta");

        let out = cmd.output().unwrap();
        assert!(String::from_utf8(out.stdout)
            .unwrap()
            .contains("hash-beta-1.2.0"));
    });
}

#[test]
fn plus_override_beats_file_override() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &["rustup", "toolchain", "install", "beta", "--no-self-update"],
        );
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly").unwrap();

        expect_stdout_ok(config, &["rustc", "+beta", "--version"], "hash-beta-1.2.0");
    });
}

#[test]
fn bad_file_override() {
    setup(&|config| {
        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "gumbo").unwrap();

        expect_err(
            config,
            &["rustc", "--version"],
            "invalid channel name 'gumbo' in",
        );
    });
}

#[test]
fn file_override_with_target_info() {
    setup(&|config| {
        let cwd = config.current_dir();
        let toolchain_file = cwd.join("rust-toolchain");
        raw::write_file(&toolchain_file, "nightly-x86_64-unknown-linux-gnu").unwrap();

        expect_err(
            config,
            &["rustc", "--version"],
            "target triple in channel name 'nightly-x86_64-unknown-linux-gnu'",
        );
    });
}

#[test]
fn docs_with_path() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        let mut cmd = clitools::cmd(config, "rustup", &["doc", "--path"]);
        clitools::env(config, &mut cmd);

        let out = cmd.output().unwrap();
        let path = format!("share{0}doc{0}rust{0}html", MAIN_SEPARATOR);
        assert!(String::from_utf8(out.stdout).unwrap().contains(&path));

        let mut cmd = clitools::cmd(
            config,
            "rustup",
            &["doc", "--path", "--toolchain", "nightly"],
        );
        clitools::env(config, &mut cmd);

        let out = cmd.output().unwrap();
        assert!(String::from_utf8(out.stdout).unwrap().contains("nightly"));
    });
}

#[test]
fn docs_topical_with_path() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "stable"]);
        expect_ok(
            config,
            &[
                "rustup",
                "toolchain",
                "install",
                "nightly",
                "--no-self-update",
            ],
        );

        for (topic, path) in mock::topical_doc_data::test_cases() {
            let mut cmd = clitools::cmd(config, "rustup", &["doc", "--path", topic]);
            clitools::env(config, &mut cmd);

            let out = cmd.output().unwrap();
            eprintln!("{:?}", String::from_utf8(out.stderr).unwrap());
            let out_str = String::from_utf8(out.stdout).unwrap();
            assert!(
                out_str.contains(&path),
                "comparing path\ntopic: '{}'\nexpected path: '{}'\noutput: {}\n\n\n",
                topic,
                path,
                out_str,
            );
        }
    });
}

#[test]
fn docs_missing() {
    setup(&|config| {
        expect_ok(config, &["rustup", "set", "profile", "minimal"]);
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_err(
            config,
            &["rustup", "doc"],
            "error: unable to view documentation which is not installed",
        );
    });
}

#[cfg(unix)]
#[test]
fn non_utf8_arg() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        let out = run(
            config,
            "rustc",
            &[
                OsStr::new("--echo-args"),
                OsStr::new("echoed non-utf8 arg:"),
                OsStr::from_bytes(b"\xc3\x28"),
            ],
            &[("RUST_BACKTRACE", "1")],
        );
        assert!(out.stderr.contains("echoed non-utf8 arg"));
    });
}

#[cfg(windows)]
#[test]
fn non_utf8_arg() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        let out = run(
            config,
            "rustc",
            &[
                OsString::from("--echo-args".to_string()),
                OsString::from("echoed non-utf8 arg:".to_string()),
                OsString::from_wide(&[0xd801, 0xd801]),
            ],
            &[("RUST_BACKTRACE", "1")],
        );
        assert!(out.stderr.contains("echoed non-utf8 arg"));
    });
}

#[cfg(unix)]
#[test]
fn non_utf8_toolchain() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        let out = run(
            config,
            "rustc",
            &[OsStr::from_bytes(b"+\xc3\x28")],
            &[("RUST_BACKTRACE", "1")],
        );
        assert!(out.stderr.contains("toolchain '�(' is not installed"));
    });
}

#[cfg(windows)]
#[test]
fn non_utf8_toolchain() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        let out = run(
            config,
            "rustc",
            &[OsString::from_wide(&[u16::from('+' as u8), 0xd801, 0xd801])],
            &[("RUST_BACKTRACE", "1")],
        );
        assert!(out.stderr.contains("toolchain '��' is not installed"));
    });
}
