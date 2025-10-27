use crate::{
    args::{Args, GenerateCompletions, InstallType, LogsOutput, PackageCommand, ServerCommand},
    client::FhirClient,
    completions,
    config::{Config, ServerConfig},
    installer::{self, Action, PackageContext, PLACEHOLDER_PACKAGE_NAME},
    print_values_table,
    registry::RegistryClient,
};
use anyhow::{bail, Context};
use clap::CommandFactory;
use console::style;
use indicatif::{MultiProgress, ProgressBar};
use std::{
    collections::HashMap,
    fs, io,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Semaphore;

const INSTALL_SCRIPT_URL: &str =
    "https://edenlabllc-kodjin-cli.s3.eu-north-1.amazonaws.com/kodjin-cli/installer.sh";
const INSTALLER_TEMPFILE: &str = "/tmp/kodjin-installer.sh";

pub async fn server(cmd: ServerCommand, mut config: Config, args: &Args) -> anyhow::Result<()> {
    match cmd {
        ServerCommand::List => {
            println!(
                "{}",
                style("List of currently configured servers:").underlined()
            );

            let mut servers = config
                .servers
                .iter()
                .map(|(name, server)| (name, server, config.current_server.as_ref() == Some(name)))
                .collect::<Vec<_>>();

            servers.sort_by_key(|(name, server, default)| (!*default, *name, &server.url));

            for (server_name, server_config, is_default) in servers {
                print!("- ");
                if *server_name == server_config.url {
                    print!("{}", style(server_name).bold());
                } else {
                    print!("{} ({})", style(server_name).bold(), server_config.url);
                }

                if is_default {
                    print!(" {}", style("(default)").bold());
                }
                println!();
            }

            Ok(())
        }
        ServerCommand::Add {
            url,
            name,
            search_url,
        } => {
            let name = name.unwrap_or_else(|| url.clone());

            if config.servers.contains_key(&name) {
                bail!("Server {name} already exists");
            }

            if let Some((name, _)) = config.servers.iter().find(|(_, server)| server.url == url) {
                if *name == url {
                    bail!("Server wth url {url} already exists");
                } else {
                    bail!("Server wth url {url} already exists ({name})");
                }
            }

            let bar = ProgressBar::new_spinner().with_message(format!("Checking server {url}"));
            bar.enable_steady_tick(Duration::from_millis(100));

            let client = FhirClient::new(
                url.clone(),
                search_url.clone(),
                args,
                Duration::from_secs(args.request_timeout),
            )
            .await?;
            let metadata = client.get_metadata().await?;

            config
                .servers
                .insert(name.clone(), ServerConfig { url, search_url });
            config.current_server = Some(name.clone());

            config.save()?;

            bar.finish_and_clear();

            print!("Added server {}", style(name).bold());

            if let Some(software) = &metadata.software {
                print!(" running {}", software.name);
                if let Some(version) = &software.version {
                    print!(" {version}");
                }
            }
            println!();

            Ok(())
        }
        ServerCommand::Remove { name } => {
            if config.servers.shift_remove(&name).is_some() {
                config.save()?;

                println!("Removed server {}", style(name).bold());

                Ok(())
            } else {
                bail!("Server {name} not in configuration");
            }
        }
        ServerCommand::Default { name } => {
            if config.servers.contains_key(&name) {
                println!("Setting {} as the default server", style(&name).bold());

                config.current_server = Some(name);
                config.save()?;
                Ok(())
            } else {
                bail!("Server {name} not found");
            }
        }
    }
}

pub async fn metadata(client: FhirClient) -> anyhow::Result<()> {
    let bar = ProgressBar::new_spinner().with_message("Fetching metadata");
    bar.enable_steady_tick(Duration::from_millis(100));

    let metadata = client.get_metadata().await?;

    bar.finish_and_clear();

    let values = [
        ("Name", metadata.name),
        ("Publisher", metadata.publisher),
        ("URL", metadata.url),
        ("Date", Some(metadata.date)),
        (
            "Software",
            metadata
                .software
                .as_ref()
                .map(|software| software.name.clone()),
        ),
        (
            "Software Version",
            metadata.software.and_then(|software| software.version),
        ),
        ("FHIR Version", Some(metadata.fhir_version)),
    ];

    print_values_table(&values);

    Ok(())
}

pub async fn install(
    cmd: PackageCommand,
    client: FhirClient,
    errors_output: LogsOutput,
) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry.clone());
    let ctx = install_ctx(
        &cmd,
        Action::Install,
        &client,
        &registry_client,
        &errors_output,
    );

    let name = match cmd.r#type {
        InstallType::Package => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one package may be supplied at a time");
            };
            installer::process_package_by_name(&ctx, name).await?;
            name
        }
        InstallType::Directory => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one directory may be supplied at a time");
            };
            installer::process_directory(&ctx, &PathBuf::from(&name)).await?;
            name
        }
        InstallType::File => {
            installer::process_file(&ctx, &cmd.name).await?;
            PLACEHOLDER_PACKAGE_NAME
        }
    };
    installer::print_report(&ctx, name);

    Ok(())
}

pub async fn uninstall(
    cmd: PackageCommand,
    client: FhirClient,
    errors_output: LogsOutput,
) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry.clone());
    let ctx = install_ctx(
        &cmd,
        Action::Uninstall,
        &client,
        &registry_client,
        &errors_output,
    );

    let name = match cmd.r#type {
        InstallType::Package => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one package may be supplied at a time");
            };
            installer::process_package_by_name(&ctx, name).await?;
            name
        }
        InstallType::Directory => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one directory may be supplied at a time");
            };
            installer::process_directory(&ctx, &PathBuf::from(name)).await?;
            name
        }
        InstallType::File => {
            installer::process_file(&ctx, &cmd.name).await?;
            PLACEHOLDER_PACKAGE_NAME
        }
    };
    installer::print_report(&ctx, name);

    Ok(())
}

pub async fn check(
    cmd: PackageCommand,
    client: FhirClient,
    errors_output: LogsOutput,
) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry.clone());
    let ctx = install_ctx(
        &cmd,
        Action::Check,
        &client,
        &registry_client,
        &errors_output,
    );

    match cmd.r#type {
        InstallType::Package => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one package may be supplied at a time");
            };
            installer::process_package_by_name(&ctx, name).await?;
        }
        InstallType::Directory => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one directory may be supplied at a time");
            };
            installer::process_directory(&ctx, &PathBuf::from(name)).await?;
        }
        InstallType::File => {
            installer::process_file(&ctx, &cmd.name).await?;
        }
    }

    Ok(())
}

fn install_ctx<'a>(
    cmd: &PackageCommand,
    action: Action,
    fhir_client: &'a FhirClient,
    registry_client: &'a RegistryClient,
    errors_output: &'a LogsOutput,
) -> PackageContext<'a> {
    let multi_progress = MultiProgress::new();
    let semaphore = Semaphore::new(5);

    let packages = Mutex::new(HashMap::new());

    PackageContext {
        fhir_client,
        action,
        progress: multi_progress,
        packages_progress: Arc::new(packages),
        semaphore: Arc::new(semaphore),
        registry_client,
        skip_preprocessing: cmd.skip_preprocessing,
        skip_strict_reference_versions: cmd.skip_strict_reference_versions,
        skip_dependencies: cmd.skip_dependencies,
        existing_resources_behaviour: cmd.existing_resources,
        parallel_search_requests: cmd.parallel_search_requests,
        errors_output,
        start_time: chrono::Local::now(),
    }
}

pub async fn tree(cmd: PackageCommand) -> anyhow::Result<()> {
    match cmd.r#type {
        InstallType::Package => {
            let [name] = cmd.name.as_slice() else {
                bail!("Only one package may be supplied at a time");
            };
            let registry_client = RegistryClient::new(cmd.registry);
            installer::print_tree(&registry_client, name, 0).await?;
        }
        InstallType::Directory => {
            todo!()
        }
        InstallType::File => bail!("Doesn't make sense with a single file"),
    }

    Ok(())
}

pub async fn info(cmd: PackageCommand) -> anyhow::Result<()> {
    let [name] = cmd.name.as_slice() else {
        bail!("Only one package may be supplied at a time");
    };
    let registry_client = RegistryClient::new(cmd.registry);
    installer::info(&registry_client, name).await
}

pub async fn download(
    cmd: PackageCommand,
    client: FhirClient,
    preprocess: bool,
) -> anyhow::Result<()> {
    let [name] = cmd.name.as_slice() else {
        bail!("Only one package may be supplied at a time");
    };

    let registry_client = RegistryClient::new(cmd.registry);
    installer::download(
        &registry_client,
        name,
        client,
        cmd.skip_strict_reference_versions,
        preprocess,
    )
    .await
}

pub fn generate_completions(cmd: GenerateCompletions) -> anyhow::Result<()> {
    let shell = match cmd.shell {
        Some(shell) => shell,
        None => {
            let shell = completions::detect_shell()?;
            eprintln!("Detected {shell} as the current shell");
            shell
        }
    };

    if cmd.install {
        completions::install_completions(shell).context("Could not install completions")?;
    } else {
        clap_complete::generate(
            shell,
            &mut Args::command(),
            Args::command().get_name(),
            &mut io::stdout(),
        );
    }

    Ok(())
}

pub async fn update(version: Option<String>) -> anyhow::Result<()> {
    let response = reqwest::get(INSTALL_SCRIPT_URL).await?;
    let script = response
        .bytes()
        .await
        .context("Could not download installer")?;

    fs::write(INSTALLER_TEMPFILE, script).context("Could not save installer")?;

    let mut command = Command::new("sh");
    command.arg(INSTALLER_TEMPFILE);

    if let Some(version) = version {
        command.arg(version);
    }

    let child = command.spawn()?;
    let output = child.wait_with_output()?;

    if !output.status.success() {
        eprintln!("Update failed, see previous logs");
    }

    fs::remove_file(INSTALLER_TEMPFILE).context("Could not clean up installer")?;

    Ok(())
}
