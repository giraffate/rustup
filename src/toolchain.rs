use crate::config::Cfg;
use crate::dist::dist::TargetTriple;
use crate::dist::dist::ToolchainDesc;
use crate::dist::download::DownloadCfg;
use crate::dist::manifest::Component;
use crate::dist::manifest::Manifest;
use crate::dist::manifestation::{Changes, Manifestation};
use crate::dist::prefix::InstallPrefix;
use crate::env_var;
use crate::errors::*;
use crate::install::{self, InstallMethod};
use crate::notifications::*;
use crate::utils::utils;

use std::env;
use std::env::consts::EXE_SUFFIX;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use url::Url;

/// A fully resolved reference to a toolchain which may or may not exist
pub struct Toolchain<'a> {
    cfg: &'a Cfg,
    name: String,
    path: PathBuf,
    dist_handler: Box<dyn Fn(crate::dist::Notification<'_>) + 'a>,
}

/// Used by the `list_component` function
pub struct ComponentStatus {
    pub component: Component,
    pub name: String,
    pub installed: bool,
    pub available: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum UpdateStatus {
    Installed,
    Updated,
    Unchanged,
}

impl<'a> Toolchain<'a> {
    pub fn from(cfg: &'a Cfg, name: &str) -> Result<Self> {
        let resolved_name = cfg.resolve_toolchain(name)?;
        let path = cfg.toolchains_dir.join(&resolved_name);
        Ok(Toolchain {
            cfg,
            name: resolved_name,
            path,
            dist_handler: Box::new(move |n| (cfg.notify_handler)(n.into())),
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn desc(&self) -> Result<ToolchainDesc> {
        Ok(ToolchainDesc::from_str(&self.name)?)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn is_symlink(&self) -> bool {
        use std::fs;
        fs::symlink_metadata(&self.path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
    pub fn exists(&self) -> bool {
        // HACK: linked toolchains are symlinks, and, contrary to what std docs
        // lead me to believe `fs::metadata`, used by `is_directory` does not
        // seem to follow symlinks on windows.
        let is_symlink = if cfg!(windows) {
            self.is_symlink()
        } else {
            false
        };
        utils::is_directory(&self.path) || is_symlink
    }
    pub fn verify(&self) -> Result<()> {
        utils::assert_is_directory(&self.path)
    }
    pub fn remove(&self) -> Result<()> {
        if self.exists() || self.is_symlink() {
            (self.cfg.notify_handler)(Notification::UninstallingToolchain(&self.name));
        } else {
            (self.cfg.notify_handler)(Notification::ToolchainNotInstalled(&self.name));
            return Ok(());
        }
        if let Some(update_hash) = self.update_hash()? {
            utils::ensure_file_removed("update hash", &update_hash)?;
        }
        let result = install::uninstall(&self.path, &|n| (self.cfg.notify_handler)(n.into()));
        if !self.exists() {
            (self.cfg.notify_handler)(Notification::UninstalledToolchain(&self.name));
        }
        result
    }
    fn install(&self, install_method: InstallMethod<'_>) -> Result<UpdateStatus> {
        assert!(self.is_valid_install_method(install_method));
        let exists = self.exists();
        if exists {
            (self.cfg.notify_handler)(Notification::UpdatingToolchain(&self.name));
        } else {
            (self.cfg.notify_handler)(Notification::InstallingToolchain(&self.name));
        }
        (self.cfg.notify_handler)(Notification::ToolchainDirectory(&self.path, &self.name));
        let updated = install_method.run(&self.path, &|n| (self.cfg.notify_handler)(n.into()))?;

        if !updated {
            (self.cfg.notify_handler)(Notification::UpdateHashMatches);
        } else {
            (self.cfg.notify_handler)(Notification::InstalledToolchain(&self.name));
        }

        let status = match (updated, exists) {
            (true, false) => UpdateStatus::Installed,
            (true, true) => UpdateStatus::Updated,
            (false, true) => UpdateStatus::Unchanged,
            (false, false) => UpdateStatus::Unchanged,
        };

        Ok(status)
    }
    fn install_if_not_installed(&self, install_method: InstallMethod<'_>) -> Result<UpdateStatus> {
        assert!(self.is_valid_install_method(install_method));
        (self.cfg.notify_handler)(Notification::LookingForToolchain(&self.name));
        if !self.exists() {
            Ok(self.install(install_method)?)
        } else {
            (self.cfg.notify_handler)(Notification::UsingExistingToolchain(&self.name));
            Ok(UpdateStatus::Unchanged)
        }
    }
    fn is_valid_install_method(&self, install_method: InstallMethod<'_>) -> bool {
        match install_method {
            InstallMethod::Copy(_) | InstallMethod::Link(_) | InstallMethod::Installer(..) => {
                self.is_custom()
            }
            InstallMethod::Dist(..) => !self.is_custom(),
        }
    }
    fn update_hash(&self) -> Result<Option<PathBuf>> {
        if self.is_custom() {
            Ok(None)
        } else {
            Ok(Some(self.cfg.get_hash_file(&self.name, true)?))
        }
    }

    fn download_cfg(&self) -> DownloadCfg<'_> {
        DownloadCfg {
            dist_root: &self.cfg.dist_root_url,
            temp_cfg: &self.cfg.temp_cfg,
            download_dir: &self.cfg.download_dir,
            notify_handler: &*self.dist_handler,
            pgp_keys: self.cfg.get_pgp_keys(),
        }
    }

    pub fn install_from_dist(
        &self,
        force_update: bool,
        components: &[&str],
        targets: &[&str],
    ) -> Result<UpdateStatus> {
        let update_hash = self.update_hash()?;
        let old_date = self.get_manifest().ok().and_then(|m| m.map(|m| m.date));
        self.install(InstallMethod::Dist(
            &self.desc()?,
            self.cfg.get_profile()?,
            update_hash.as_ref().map(|p| &**p),
            self.download_cfg(),
            force_update,
            self.exists(),
            old_date.as_ref().map(|s| &**s),
            components,
            targets,
        ))
    }

    pub fn install_from_dist_if_not_installed(&self) -> Result<UpdateStatus> {
        let update_hash = self.update_hash()?;
        self.install_if_not_installed(InstallMethod::Dist(
            &self.desc()?,
            self.cfg.get_profile()?,
            update_hash.as_ref().map(|p| &**p),
            self.download_cfg(),
            false,
            false,
            None,
            &[],
            &[],
        ))
    }
    pub fn is_custom(&self) -> bool {
        ToolchainDesc::from_str(&self.name).is_err()
    }
    pub fn is_tracking(&self) -> bool {
        ToolchainDesc::from_str(&self.name)
            .ok()
            .map(|d| d.is_tracking())
            == Some(true)
    }

    fn ensure_custom(&self) -> Result<()> {
        if !self.is_custom() {
            Err(crate::ErrorKind::InvalidCustomToolchainName(self.name.to_string()).into())
        } else {
            Ok(())
        }
    }

    pub fn install_from_installers(&self, installers: &[&OsStr]) -> Result<()> {
        self.ensure_custom()?;

        self.remove()?;

        // FIXME: This should do all downloads first, then do
        // installs, and do it all in a single transaction.
        for installer in installers {
            let installer_str = installer.to_str().unwrap_or("bogus");
            match installer_str.rfind('.') {
                Some(i) => {
                    let extension = &installer_str[i + 1..];
                    if extension != "gz" {
                        return Err(ErrorKind::BadInstallerType(extension.to_string()).into());
                    }
                }
                None => return Err(ErrorKind::BadInstallerType(String::from("(none)")).into()),
            }

            // FIXME: Pretty hacky
            let is_url = installer_str.starts_with("file://")
                || installer_str.starts_with("http://")
                || installer_str.starts_with("https://");
            let url = Url::parse(installer_str).ok();
            let url = if is_url { url } else { None };
            if let Some(url) = url {
                // Download to a local file
                let local_installer = self.cfg.temp_cfg.new_file_with_ext("", ".tar.gz")?;
                utils::download_file(&url, &local_installer, None, &|n| {
                    (self.cfg.notify_handler)(n.into())
                })?;
                self.install(InstallMethod::Installer(
                    &local_installer,
                    &self.cfg.temp_cfg,
                ))?;
            } else {
                // If installer is a filename

                // No need to download
                let local_installer = Path::new(installer);

                // Install from file
                self.install(InstallMethod::Installer(
                    &local_installer,
                    &self.cfg.temp_cfg,
                ))?;
            }
        }

        Ok(())
    }

    pub fn install_from_dir(&self, src: &Path, link: bool) -> Result<()> {
        self.ensure_custom()?;

        let mut pathbuf = PathBuf::from(src);

        pathbuf.push("lib");
        utils::assert_is_directory(&pathbuf)?;
        pathbuf.pop();
        pathbuf.push("bin");
        utils::assert_is_directory(&pathbuf)?;
        pathbuf.push(format!("rustc{}", EXE_SUFFIX));
        utils::assert_is_file(&pathbuf)?;

        if link {
            self.install(InstallMethod::Link(&utils::to_absolute(src)?))?;
        } else {
            self.install(InstallMethod::Copy(src))?;
        }

        Ok(())
    }

    pub fn create_command<T: AsRef<OsStr>>(&self, binary: T) -> Result<Command> {
        if !self.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(self.name.to_owned()).into());
        }

        // Create the path to this binary within the current toolchain sysroot
        let binary = if let Some(binary_str) = binary.as_ref().to_str() {
            if binary_str.to_lowercase().ends_with(EXE_SUFFIX) {
                binary.as_ref().to_owned()
            } else {
                OsString::from(format!("{}{}", binary_str, EXE_SUFFIX))
            }
        } else {
            // Very weird case. Non-unicode command.
            binary.as_ref().to_owned()
        };

        let bin_path = self.path.join("bin").join(&binary);
        let path = if utils::is_file(&bin_path) {
            &bin_path
        } else {
            let recursion_count = env::var("RUST_RECURSION_COUNT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            if recursion_count > env_var::RUST_RECURSION_COUNT_MAX - 1 {
                let defaults = self.cfg.get_default()?;
                return Err(ErrorKind::BinaryNotFound(
                    binary.to_string_lossy().into(),
                    self.name.clone(),
                    Some(&self.name) == defaults.as_ref(),
                )
                .into());
            }
            Path::new(&binary)
        };
        let mut cmd = Command::new(&path);
        self.set_env(&mut cmd);
        Ok(cmd)
    }

    // Create a command as a fallback for another toolchain. This is used
    // to give custom toolchains access to cargo
    pub fn create_fallback_command<T: AsRef<OsStr>>(
        &self,
        binary: T,
        primary_toolchain: &Toolchain<'_>,
    ) -> Result<Command> {
        // With the hacks below this only works for cargo atm
        assert!(binary.as_ref() == "cargo" || binary.as_ref() == "cargo.exe");

        if !self.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(self.name.to_owned()).into());
        }
        if !primary_toolchain.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(primary_toolchain.name.to_owned()).into());
        }

        let src_file = self.path.join("bin").join(format!("cargo{}", EXE_SUFFIX));

        // MAJOR HACKS: Copy cargo.exe to its own directory on windows before
        // running it. This is so that the fallback cargo, when it in turn runs
        // rustc.exe, will run the rustc.exe out of the PATH environment
        // variable, _not_ the rustc.exe sitting in the same directory as the
        // fallback. See the `fallback_cargo_calls_correct_rustc` test case and
        // PR 812.
        //
        // On Windows, spawning a process will search the running application's
        // directory for the exe to spawn before searching PATH, and we don't want
        // it to do that, because cargo's directory contains the _wrong_ rustc. See
        // the documentation for the lpCommandLine argument of CreateProcess.
        let exe_path = if cfg!(windows) {
            use std::fs;
            let fallback_dir = self.cfg.rustup_dir.join("fallback");
            fs::create_dir_all(&fallback_dir)
                .chain_err(|| "unable to create dir to hold fallback exe")?;
            let fallback_file = fallback_dir.join("cargo.exe");
            if fallback_file.exists() {
                fs::remove_file(&fallback_file)
                    .chain_err(|| "unable to unlink old fallback exe")?;
            }
            fs::hard_link(&src_file, &fallback_file)
                .chain_err(|| "unable to hard link fallback exe")?;
            fallback_file
        } else {
            src_file
        };
        let mut cmd = Command::new(exe_path);
        primary_toolchain.set_env(&mut cmd); // set up the environment to match rustc, not cargo
        cmd.env("RUSTUP_TOOLCHAIN", &primary_toolchain.name);
        Ok(cmd)
    }

    fn set_env(&self, cmd: &mut Command) {
        self.set_ldpath(cmd);

        // Because rustup and cargo use slightly different
        // definitions of cargo home (rustup doesn't read HOME on
        // windows), we must set it here to ensure cargo and
        // rustup agree.
        if let Ok(cargo_home) = utils::cargo_home() {
            cmd.env("CARGO_HOME", &cargo_home);
        }

        env_var::inc("RUST_RECURSION_COUNT", cmd);

        cmd.env("RUSTUP_TOOLCHAIN", &self.name);
        cmd.env("RUSTUP_HOME", &self.cfg.rustup_dir);
    }

    pub fn set_ldpath(&self, cmd: &mut Command) {
        let mut new_path = vec![self.path.join("lib")];

        #[cfg(not(target_os = "macos"))]
        mod sysenv {
            pub const LOADER_PATH: &str = "LD_LIBRARY_PATH";
        }
        #[cfg(target_os = "macos")]
        mod sysenv {
            // When loading and linking a dynamic library or bundle, dlopen
            // searches in LD_LIBRARY_PATH, DYLD_LIBRARY_PATH, PWD, and
            // DYLD_FALLBACK_LIBRARY_PATH.
            // In the Mach-O format, a dynamic library has an "install path."
            // Clients linking against the library record this path, and the
            // dynamic linker, dyld, uses it to locate the library.
            // dyld searches DYLD_LIBRARY_PATH *before* the install path.
            // dyld searches DYLD_FALLBACK_LIBRARY_PATH only if it cannot
            // find the library in the install path.
            // Setting DYLD_LIBRARY_PATH can easily have unintended
            // consequences.
            pub const LOADER_PATH: &str = "DYLD_FALLBACK_LIBRARY_PATH";
        }
        if cfg!(target_os = "macos")
            && env::var_os(sysenv::LOADER_PATH)
                .filter(|x| x.len() > 0)
                .is_none()
        {
            // These are the defaults when DYLD_FALLBACK_LIBRARY_PATH isn't
            // set or set to an empty string. Since we are explicitly setting
            // the value, make sure the defaults still work.
            if let Some(home) = env::var_os("HOME") {
                new_path.push(PathBuf::from(home).join("lib"));
            }
            new_path.push(PathBuf::from("/usr/local/lib"));
            new_path.push(PathBuf::from("/usr/lib"));
        }

        env_var::prepend_path(sysenv::LOADER_PATH, new_path, cmd);

        // Prepend CARGO_HOME/bin to the PATH variable so that we're sure to run
        // cargo/rustc via the proxy bins. There is no fallback case for if the
        // proxy bins don't exist. We'll just be running whatever happens to
        // be on the PATH.
        let mut path_entries = vec![];
        if let Ok(cargo_home) = utils::cargo_home() {
            path_entries.push(cargo_home.join("bin"));
        }

        if cfg!(target_os = "windows") {
            path_entries.push(self.path.join("bin"));
        }

        env_var::prepend_path("PATH", path_entries, cmd);
    }

    pub fn doc_path(&self, relative: &str) -> Result<PathBuf> {
        self.verify()?;

        let parts = vec!["share", "doc", "rust", "html"];
        let mut doc_dir = self.path.clone();
        for part in parts {
            doc_dir.push(part);
        }
        doc_dir.push(relative);

        Ok(doc_dir)
    }
    pub fn open_docs(&self, relative: &str) -> Result<()> {
        self.verify()?;

        utils::open_browser(&self.doc_path(relative)?)
    }

    pub fn make_default(&self) -> Result<()> {
        self.cfg.set_default(&self.name)
    }
    pub fn make_override(&self, path: &Path) -> Result<()> {
        self.cfg.settings_file.with_mut(|s| {
            s.add_override(path, self.name.clone(), self.cfg.notify_handler.as_ref());
            Ok(())
        })
    }

    pub fn get_manifest(&self) -> Result<Option<Manifest>> {
        if !self.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(self.name.to_owned()).into());
        }

        let toolchain = &self.name;
        let toolchain = ToolchainDesc::from_str(toolchain)?;

        let prefix = InstallPrefix::from(self.path.to_owned());
        let manifestation = Manifestation::open(prefix, toolchain.target)?;

        manifestation.load_manifest()
    }

    pub fn show_version(&self) -> Result<Option<String>> {
        match self.get_manifest()? {
            Some(manifest) => Ok(Some(manifest.get_rust_version()?.to_string())),
            None => Ok(None),
        }
    }

    pub fn show_dist_version(&self) -> Result<Option<String>> {
        let update_hash = self.update_hash()?;

        match crate::dist::dist::dl_v2_manifest(
            self.download_cfg(),
            update_hash.as_ref().map(|p| &**p),
            &self.desc()?,
        )? {
            Some((manifest, _)) => Ok(Some(manifest.get_rust_version()?.to_string())),
            None => Ok(None),
        }
    }

    pub fn list_components(&self) -> Result<Vec<ComponentStatus>> {
        if !self.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(self.name.to_owned()).into());
        }

        let toolchain = &self.name;
        let toolchain = ToolchainDesc::from_str(toolchain)
            .chain_err(|| ErrorKind::ComponentsUnsupported(self.name.to_string()))?;

        let prefix = InstallPrefix::from(self.path.to_owned());
        let manifestation = Manifestation::open(prefix, toolchain.target.clone())?;

        if let Some(manifest) = manifestation.load_manifest()? {
            let config = manifestation.read_config()?;

            // Return all optional components of the "rust" package for the
            // toolchain's target triple.
            let mut res = Vec::new();

            let rust_pkg = manifest
                .packages
                .get("rust")
                .expect("manifest should contain a rust package");
            let targ_pkg = rust_pkg
                .targets
                .get(&toolchain.target)
                .expect("installed manifest should have a known target");

            for component in &targ_pkg.components {
                let installed = config
                    .as_ref()
                    .map(|c| component.contained_within(&c.components))
                    .unwrap_or(false);

                let component_target = TargetTriple::new(&component.target());

                // Get the component so we can check if it is available
                let component_pkg = manifest
                    .get_package(&component.short_name_in_manifest())
                    .unwrap_or_else(|_| {
                        panic!(
                            "manifest should contain component {}",
                            &component.short_name(&manifest)
                        )
                    });
                let component_target_pkg = component_pkg
                    .targets
                    .get(&component_target)
                    .expect("component should have target toolchain");

                res.push(ComponentStatus {
                    component: component.clone(),
                    name: component.name(&manifest),
                    installed,
                    available: component_target_pkg.available(),
                });
            }

            res.sort_by(|a, b| a.component.cmp(&b.component));

            Ok(res)
        } else {
            Err(ErrorKind::ComponentsUnsupported(self.name.to_string()).into())
        }
    }

    fn get_component_suggestion(
        &self,
        component: &Component,
        manifest: &Manifest,
        only_installed: bool,
    ) -> Option<String> {
        use strsim::damerau_levenshtein;

        // Suggest only for very small differences
        // High number can result in inaccurate suggestions for short queries e.g. `rls`
        const MAX_DISTANCE: usize = 3;

        let components = self.list_components();
        if let Ok(components) = components {
            let short_name_distance = components
                .iter()
                .filter(|c| !only_installed || c.installed)
                .map(|c| {
                    (
                        damerau_levenshtein(
                            &c.component.name(manifest)[..],
                            &component.name(manifest)[..],
                        ),
                        c,
                    )
                })
                .min_by_key(|t| t.0)
                .expect("There should be always at least one component");

            let long_name_distance = components
                .iter()
                .filter(|c| !only_installed || c.installed)
                .map(|c| {
                    (
                        damerau_levenshtein(
                            &c.component.name_in_manifest()[..],
                            &component.name(manifest)[..],
                        ),
                        c,
                    )
                })
                .min_by_key(|t| t.0)
                .expect("There should be always at least one component");

            let mut closest_distance = short_name_distance;
            let mut closest_match = short_name_distance.1.component.short_name(manifest);

            // Find closer suggestion
            if short_name_distance.0 > long_name_distance.0 {
                closest_distance = long_name_distance;

                // Check if only targets differ
                if closest_distance.1.component.short_name_in_manifest()
                    == component.short_name_in_manifest()
                {
                    closest_match = long_name_distance.1.component.target();
                } else {
                    closest_match = long_name_distance
                        .1
                        .component
                        .short_name_in_manifest()
                        .to_string();
                }
            } else {
                // Check if only targets differ
                if closest_distance.1.component.short_name(manifest)
                    == component.short_name(manifest)
                {
                    closest_match = short_name_distance.1.component.target();
                }
            }

            // If suggestion is too different don't suggest anything
            if closest_distance.0 > MAX_DISTANCE {
                None
            } else {
                Some(closest_match)
            }
        } else {
            None
        }
    }

    pub fn add_component(&self, mut component: Component) -> Result<()> {
        if !self.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(self.name.to_owned()).into());
        }

        let toolchain = &self.name;
        let toolchain = ToolchainDesc::from_str(toolchain)
            .chain_err(|| ErrorKind::ComponentsUnsupported(self.name.to_string()))?;

        let prefix = InstallPrefix::from(self.path.to_owned());
        let manifestation = Manifestation::open(prefix, toolchain.target.clone())?;

        if let Some(manifest) = manifestation.load_manifest()? {
            // Rename the component if necessary.
            if let Some(c) = manifest.rename_component(&component) {
                component = c;
            }

            // Validate the component name
            let rust_pkg = manifest
                .packages
                .get("rust")
                .expect("manifest should contain a rust package");
            let targ_pkg = rust_pkg
                .targets
                .get(&toolchain.target)
                .expect("installed manifest should have a known target");

            if !targ_pkg.components.contains(&component) {
                let wildcard_component = component.wildcard();
                if targ_pkg.components.contains(&wildcard_component) {
                    component = wildcard_component;
                } else {
                    return Err(ErrorKind::UnknownComponent(
                        self.name.to_string(),
                        component.description(&manifest),
                        self.get_component_suggestion(&component, &manifest, false),
                    )
                    .into());
                }
            }

            let changes = Changes {
                explicit_add_components: vec![component],
                remove_components: vec![],
            };

            manifestation.update(
                &manifest,
                changes,
                false,
                &self.download_cfg(),
                &self.download_cfg().notify_handler,
                &toolchain.manifest_name(),
                false,
            )?;

            Ok(())
        } else {
            Err(ErrorKind::ComponentsUnsupported(self.name.to_string()).into())
        }
    }

    pub fn remove_component(&self, mut component: Component) -> Result<()> {
        if !self.exists() {
            return Err(ErrorKind::ToolchainNotInstalled(self.name.to_owned()).into());
        }

        let toolchain = &self.name;
        let toolchain = ToolchainDesc::from_str(toolchain)
            .chain_err(|| ErrorKind::ComponentsUnsupported(self.name.to_string()))?;

        let prefix = InstallPrefix::from(self.path.to_owned());
        let manifestation = Manifestation::open(prefix, toolchain.target.clone())?;

        if let Some(manifest) = manifestation.load_manifest()? {
            // Rename the component if necessary.
            if let Some(c) = manifest.rename_component(&component) {
                component = c;
            }

            let dist_config = manifestation.read_config()?.unwrap();
            if !dist_config.components.contains(&component) {
                let wildcard_component = component.wildcard();
                if dist_config.components.contains(&wildcard_component) {
                    component = wildcard_component;
                } else {
                    return Err(ErrorKind::UnknownComponent(
                        self.name.to_string(),
                        component.description(&manifest),
                        self.get_component_suggestion(&component, &manifest, true),
                    )
                    .into());
                }
            }

            let changes = Changes {
                explicit_add_components: vec![],
                remove_components: vec![component],
            };

            manifestation.update(
                &manifest,
                changes,
                false,
                &self.download_cfg(),
                &self.download_cfg().notify_handler,
                &toolchain.manifest_name(),
                false,
            )?;

            Ok(())
        } else {
            Err(ErrorKind::ComponentsUnsupported(self.name.to_string()).into())
        }
    }

    pub fn binary_file(&self, name: &str) -> PathBuf {
        let mut path = self.path.clone();
        path.push("bin");
        path.push(name.to_owned() + env::consts::EXE_SUFFIX);
        path
    }
}
