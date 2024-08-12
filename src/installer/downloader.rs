use super::package::FhirPackage;
use crate::{registry::RegistryClient, storage};
use anyhow::Context;
use console::style;
use deno_npm::registry::NpmPackageVersionInfo;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use libflate::gzip;
use reqwest::Url;
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    time::Duration,
};

pub async fn download_package(
    registry_client: &RegistryClient,
    name: String,
    version_info: NpmPackageVersionInfo,
    bar: ProgressBar,
) -> anyhow::Result<FhirPackage> {
    bar.enable_steady_tick(Duration::from_millis(100));

    let registry_dir = match Url::parse(&registry_client.base_url) {
        Ok(url) => url.host_str().context("Invalid registry url")?.to_owned(),
        Err(_) => registry_client.base_url.to_string(),
    };
    let final_output_dir = storage::packages_dir()?
        .join(&registry_dir)
        .join(&name)
        .join(version_info.version.to_string());

    // We already have the package
    if final_output_dir.exists() {
        return Ok(FhirPackage::new(final_output_dir));
    }

    let tarball_url = &version_info.dist.tarball;

    let response = registry_client
        .client
        .get(tarball_url)
        .send()
        .await?
        .error_for_status()?;

    let content_length = response
        .content_length()
        .context("Server did not provide a content length")?;

    let archive_file_path =
        storage::downloads_dir()?.join(format!("{name}-{}.tar.gz", version_info.version));
    let mut output_writer = BufWriter::new(File::create(&archive_file_path)?);

    let styled_package_name = style(format!("{name}@{}", version_info.version)).bold();

    // let download_bar = ProgressBar::new(content_length)
    bar.set_length(content_length);
    bar.set_message(format!("Fetching {styled_package_name}"));
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner} {msg} [{wide_bar}] {decimal_bytes}/{decimal_total_bytes}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    bar.set_length(content_length);
    bar.enable_steady_tick(Duration::from_millis(100));

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        output_writer.write_all(&chunk)?;
        bar.inc(chunk.len() as u64);
    }

    output_writer.flush()?;

    bar.set_style(ProgressStyle::default_spinner());
    bar.set_message(format!("Extrating {styled_package_name}"));

    tokio::task::spawn_blocking(move || {
        let temp_output_dir = storage::packages_dir()?
            .join(&registry_dir)
            .join(name)
            .join(format!(".{}-temp", version_info.version));
        fs::create_dir_all(&temp_output_dir)?;

        let reader = gzip::Decoder::new(BufReader::new(File::open(&archive_file_path)?))?;
        let mut archive = tar::Archive::new(reader);

        for result in archive.entries().context("Extraction error")? {
            let mut entry = result?;
            let entry_path = entry.path()?.into_owned();

            if let Some(parent_dir) = entry_path.parent() {
                fs::create_dir_all(temp_output_dir.join(parent_dir))
                    .context("Could not create dir in package")?;
            }

            let entry_output_path = temp_output_dir.join(&entry_path);
            let mut entry_output_file = File::create(&entry_output_path).with_context(|| {
                format!("Could not create file {}", entry_output_path.display())
            })?;

            io::copy(&mut entry, &mut entry_output_file).context("Could not extract file")?;

            bar.set_message(format!(
                "Extracting {styled_package_name}: {}",
                entry_path.display()
            ));
        }

        fs::rename(temp_output_dir, &final_output_dir)?;

        // Delete the archive after it was extracted
        fs::remove_file(archive_file_path).context("Could not remove archive")?;

        Ok(FhirPackage::new(final_output_dir))
    })
    .await
    .unwrap()
}
