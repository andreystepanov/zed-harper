use std::fs;
use std::path::PathBuf;
use zed::Command;
use zed_extension_api::{self as zed, Result, serde_json::json, settings::LspSettings};

static NAME: &str = "harper-ls";

struct HarperExtension {
    binary_cache: Option<PathBuf>,
}

#[derive(Clone)]
struct HarperBinary {
    path: PathBuf,
    args: Option<Vec<String>>,
    env: Option<Vec<(String, String)>>,
}

impl HarperExtension {
    fn new() -> Self {
        Self { binary_cache: None }
    }

    fn get_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<HarperBinary> {
        let binary = LspSettings::for_worktree(NAME, worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary)
            .and_then(|binary| binary.path.map(|path| (path, binary.arguments.clone())));

        if let Some((path, args)) = binary {
            return Ok(HarperBinary {
                path: PathBuf::from(path),
                args,
                env: Some(worktree.shell_env()),
            });
        }

        if let Some(path) = worktree.which(NAME) {
            return Ok(HarperBinary {
                path: PathBuf::from(path),
                args: None,
                env: Some(worktree.shell_env()),
            });
        }

        if let Some(path) = &self.binary_cache
            && path.exists()
        {
            return Ok(HarperBinary {
                path: path.clone(),
                args: None,
                env: None,
            });
        }

        self.install_binary(language_server_id)
    }

    fn install_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
    ) -> Result<HarperBinary> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "elijah-potter/harper",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("Failed to fetch latest release: {e}"))?;

        let (platform, arch) = zed::current_platform();
        let arch_name = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            zed::Architecture::X86 => return Err("x86 architecture is not supported".into()),
        };

        let (os_str, file_ext) = match platform {
            zed::Os::Mac => ("apple-darwin", "tar.gz"),
            zed::Os::Linux => ("unknown-linux-gnu", "tar.gz"),
            zed::Os::Windows => ("pc-windows-msvc", "zip"),
        };

        let asset_name = format!("{NAME}-{arch_name}-{os_str}.{file_ext}");
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("No compatible Harper binary found for {arch_name}-{os_str}"))?;

        let version_dir = format!("{NAME}-{}", release.version);
        let mut binary_path = PathBuf::from(&version_dir).join(NAME);

        if platform == zed::Os::Windows {
            binary_path.set_extension("exe");
        }

        if !binary_path.exists() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            let download_result = (|| -> Result<()> {
                zed::download_file(
                    &asset.download_url,
                    &version_dir,
                    if platform == zed::Os::Windows {
                        zed::DownloadedFileType::Zip
                    } else {
                        zed::DownloadedFileType::GzipTar
                    },
                )
                .map_err(|e| format!("Failed to download Harper binary: {e}"))?;

                zed::make_file_executable(binary_path.to_str().ok_or("Invalid binary path")?)
                    .map_err(|e| format!("Failed to make binary executable: {e}"))?;

                Ok(())
            })();

            if let Err(e) = download_result {
                fs::remove_dir_all(&version_dir).ok();
                return Err(e);
            }

            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string()
                        && name != version_dir
                    {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        self.binary_cache = Some(binary_path.clone());

        Ok(HarperBinary {
            path: binary_path,
            args: None,
            env: None,
        })
    }
}

impl zed::Extension for HarperExtension {
    fn new() -> Self {
        Self::new()
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Command> {
        let HarperBinary { path, args, env } = self.get_binary(language_server_id, worktree)?;

        let command = path
            .to_str()
            .ok_or("Failed to convert binary path to string")?
            .to_string();
        let args = args.unwrap_or_else(|| vec!["--stdio".to_string()]);
        let env = env.unwrap_or_default();

        Ok(Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let options = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone());

        Ok(options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| {
                lsp_settings
                    .settings
                    .clone()
                    .or_else(|| Some(json!({ "harper-ls": { } })))
            });

        Ok(settings)
    }
}

zed::register_extension!(HarperExtension);
