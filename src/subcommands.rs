use crate::{
    args::{InstallType, PackageCommand, ServerCommand},
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

pub async fn server(
    cmd: ServerCommand,
    mut config: Config,
    insecure_certificates: bool,
) -> anyhow::Result<()> {
    match cmd {
        ServerCommand::List => {
            println!(
                "{}",
                style("List of currently configured servers:").underlined()
            );

            for (server_name, server_config) in &config.servers {
                print!("- ");
                if *server_name == server_config.url {
                    print!("{}", style(server_name).bold());
                } else {
                    print!("{} ({})", style(server_name).bold(), server_config.url);
                }

                if config.current_server.as_ref() == Some(server_name) {
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

            let bar = ProgressBar::new_spinner().with_message(format!("Checking server {url}"));
            bar.enable_steady_tick(Duration::from_millis(100));

            let client = FhirClient::new(url.clone(), insecure_certificates);
            let metadata = client.get_metadata().await?;

            config.servers.insert(name.clone(), ServerConfig { url });

            if config.servers.len() == 1 {
                config.current_server = Some(config.servers.keys().next().unwrap().clone());
            }

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

pub async fn install(cmd: PackageCommand, client: FhirClient) -> anyhow::Result<()> {
    let multi_progress = MultiProgress::new();
    let semaphore = Semaphore::new(5);

    let packages = Mutex::new(HashMap::new());
    let registry_client = RegistryClient::new(cmd.registry);

    let ctx = InstallContext {
        fhir_client: &client,
        action: installer::Action::Install,
        progress: &multi_progress,
        current_packages: &packages,
        semaphore: &semaphore,
        registry_client: &registry_client,
        skip_strict_reference_versions: cmd.skip_strict_reference_versions,
    };

    match cmd.r#type {
        InstallType::Package => {
            installer::install_package_by_name(ctx, cmd.name).await?;
        }
        InstallType::Directory => {
            installer::process_directory(ctx, &PathBuf::from(cmd.name)).await?;
        }
    }

    Ok(())
}

pub async fn uninstall(cmd: PackageCommand, client: FhirClient) -> anyhow::Result<()> {
    let multi_progress = MultiProgress::new();
    let semaphore = Semaphore::new(5);

    let packages = Mutex::new(HashMap::new());
    let registry_client = RegistryClient::new(cmd.registry);

    let ctx = InstallContext {
        fhir_client: &client,
        action: installer::Action::Uninstall,
        progress: &multi_progress,
        current_packages: &packages,
        semaphore: &semaphore,
        registry_client: &registry_client,
        skip_strict_reference_versions: cmd.skip_strict_reference_versions,
    };

    match cmd.r#type {
        InstallType::Package => {
            installer::install_package_by_name(ctx, cmd.name).await?;
        }
        InstallType::Directory => {
            installer::process_directory(ctx, &PathBuf::from(cmd.name)).await?;
        }
    }

    Ok(())
}

pub async fn check(cmd: PackageCommand, client: FhirClient) -> anyhow::Result<()> {
    let multi_progress = MultiProgress::new();
    let semaphore = Semaphore::new(5);

    let packages = Mutex::new(HashMap::new());
    let registry_client = RegistryClient::new(cmd.registry);

    let ctx = InstallContext {
        fhir_client: &client,
        action: installer::Action::Install,
        progress: &multi_progress,
        current_packages: &packages,
        semaphore: &semaphore,
        registry_client: &registry_client,
        skip_strict_reference_versions: cmd.skip_strict_reference_versions,
    };

    match cmd.r#type {
        InstallType::Package => {
            installer::check_package_installed(ctx, &registry_client, &cmd.name).await?;
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

pub async fn download(cmd: PackageCommand, client: FhirClient) -> anyhow::Result<()> {
    let registry_client = RegistryClient::new(cmd.registry);
    installer::download(
        &registry_client,
        &cmd.name,
        client,
        cmd.skip_strict_reference_versions,
    )
    .await
}
