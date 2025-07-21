use crate::{
    args::{Args, InstallType, LogsOutput, PackageCommand, ServerCommand},
    client::FhirClient,
    config::{Config, ServerConfig},
    installer::{self, InstallContext},
    print_values_table,
    registry::RegistryClient,
};
use anyhow::bail;
use console::style;
use indicatif::{MultiProgress, ProgressBar};
use std::{collections::HashMap, path::PathBuf, sync::Mutex, time::Duration};
use tokio::sync::Semaphore;

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
        ServerCommand::Add { url, name } => {
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
                args.insecure_certificates,
                Duration::from_secs(args.request_timeout),
            );
            let metadata = client.get_metadata().await?;

            config.servers.insert(name.clone(), ServerConfig { url });
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
    let multi_progress = MultiProgress::new();
    let semaphore = Semaphore::new(5);

    let packages = Mutex::new(HashMap::new());
    let registry_client = RegistryClient::new(cmd.registry);

    let ctx = InstallContext {
        fhir_client: &client,
        action: installer::Action::Install,
        progress: &multi_progress,
        packages_progress: &packages,
        semaphore: &semaphore,
        registry_client: &registry_client,
        skip_strict_reference_versions: cmd.skip_strict_reference_versions,
        skip_dependencies: cmd.skip_dependencies,
        parallel_search_requests: cmd.parallel_search_requests,
        existing_resources_behaviour: cmd.existing_resources,
        errors_output: &errors_output,
        start_time: chrono::Local::now(),
    };

    match cmd.r#type {
        InstallType::Package => {
            installer::install_package_by_name(ctx, cmd.name.clone()).await?;
        }
        InstallType::Directory => {
            installer::process_directory(ctx, &PathBuf::from(cmd.name.clone())).await?;
        }
    }
    installer::print_report(ctx, &cmd.name);

    Ok(())
}

pub async fn uninstall(
    cmd: PackageCommand,
    client: FhirClient,
    errors_output: LogsOutput,
) -> anyhow::Result<()> {
    let multi_progress = MultiProgress::new();
    let semaphore = Semaphore::new(5);

    let packages = Mutex::new(HashMap::new());
    let registry_client = RegistryClient::new(cmd.registry);

    let ctx = InstallContext {
        fhir_client: &client,
        action: installer::Action::Uninstall,
        progress: &multi_progress,
        packages_progress: &packages,
        semaphore: &semaphore,
        registry_client: &registry_client,
        skip_strict_reference_versions: cmd.skip_strict_reference_versions,
        skip_dependencies: cmd.skip_dependencies,
        existing_resources_behaviour: cmd.existing_resources,
        parallel_search_requests: cmd.parallel_search_requests,
        errors_output: &errors_output,
        start_time: chrono::Local::now(),
    };

    match cmd.r#type {
        InstallType::Package => {
            installer::install_package_by_name(ctx, cmd.name.clone()).await?;
        }
        InstallType::Directory => {
            installer::process_directory(ctx, &PathBuf::from(cmd.name.clone())).await?;
        }
    }
    installer::print_report(ctx, &cmd.name);

    Ok(())
}

pub async fn check(cmd: PackageCommand, client: FhirClient) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry);

    match cmd.r#type {
        InstallType::Package => {
            installer::check_package_installed(
                &client,
                &registry_client,
                &cmd.name,
                cmd.parallel_search_requests,
            )
            .await?;
        }
        InstallType::Directory => {
            todo!()
        }
    }

    Ok(())
}

pub async fn tree(cmd: PackageCommand) -> anyhow::Result<()> {
    match cmd.r#type {
        InstallType::Package => {
            let registry_client = RegistryClient::new(cmd.registry);
            installer::print_tree(&registry_client, &cmd.name, 0).await?;
        }
        InstallType::Directory => {
            todo!()
        }
    }

    Ok(())
}

pub async fn info(cmd: PackageCommand) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry);
    installer::info(&registry_client, &cmd.name).await
}

pub async fn download(
    cmd: PackageCommand,
    client: FhirClient,
    preprocess: bool,
) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry);
    installer::download(
        &registry_client,
        &cmd.name,
        client,
        cmd.skip_strict_reference_versions,
        preprocess,
    )
    .await
}
