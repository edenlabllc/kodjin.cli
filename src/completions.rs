use anyhow::{anyhow, bail, Context};
use clap::CommandFactory;
use clap_complete::Shell;
use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    process::Command,
    str::FromStr,
};

use crate::args::Args;

const BASH_COMPLETIONS_PATH: &str = "/etc/bash_completion.d";
const FISH_COMPLETIONS_PATH: &str = "/etc/fish/completions";
const ZSH_SOURCE_COMPLETIONS: &str = "source <(kodjin-cli generate-completions zsh)";
const ZSHRC: &str = ".zshrc";

pub fn detect_shell() -> anyhow::Result<Shell> {
    let value = env::var("SHELL").context("SHELL env variable not set")?;
    let value = value.split('/').next_back().unwrap_or(&value);

    Shell::from_str(value)
        .map_err(|_| anyhow!("Detected shell '{value}' is not a supported shell for completions"))
}

pub fn install_completions(shell: Shell) -> anyhow::Result<()> {
    match shell {
        Shell::Bash => {
            if env::var("USER").is_ok_and(|user| user != "root") {
                eprintln!("Elavating to root to install system-wide completions");
                let child = Command::new("sudo")
                    .args([
                        "kodjin-cli",
                        "generate-completions",
                        &Shell::Bash.to_string(),
                        "--install",
                    ])
                    .spawn()?;
                child.wait_with_output()?;
            } else {
                write_completions_file(BASH_COMPLETIONS_PATH, "kodjin-cli.bash", Shell::Bash)?
            }
        }
        Shell::Fish => {
            if env::var("USER").is_ok_and(|user| user != "root") {
                eprintln!("Elavating to root to install system-wide completions");
                let child = Command::new("sudo")
                    .args([
                        "kodjin-cli",
                        "generate-completions",
                        &Shell::Fish.to_string(),
                        "--install",
                    ])
                    .spawn()?;
                child.wait_with_output()?;
            } else {
                write_completions_file(FISH_COMPLETIONS_PATH, "kodjin-cli.fish", Shell::Fish)?
            }
        }
        Shell::Zsh => {
            let zshrc_path = Path::new(&env::var("HOME").context("HOME not set")?).join(ZSHRC);
            if fs::read_to_string(&zshrc_path)
                .is_ok_and(|contents| contents.contains(ZSH_SOURCE_COMPLETIONS))
            {
                eprintln!("Completions already installed")
            } else {
                let mut file = File::options()
                    .create(true)
                    .append(true)
                    .open(&zshrc_path)
                    .context("Could not open zshrc")?;

                file.write_all(b"\n")?;
                file.write_all(ZSH_SOURCE_COMPLETIONS.as_bytes())?;
                file.write_all(b"\n")?;

                eprintln!("zshrc modified to add completions");
            }
        }
        _ => bail!("Automatic completions installation is not supported for {shell}"),
    }

    Ok(())
}

fn write_completions_file(base_dir: &str, filename: &str, shell: Shell) -> anyhow::Result<()> {
    let path = Path::new(base_dir);
    if !path.exists() {
        fs::create_dir_all(path).context("Could not create completions path")?;
    }
    let file_path = path.join(filename);
    eprintln!("Installing completions to '{}'", file_path.display());
    let mut writer = BufWriter::new(File::create(file_path)?);

    clap_complete::generate(
        shell,
        &mut Args::command(),
        Args::command().get_name(),
        &mut writer,
    );

    writer.flush()?;

    Ok(())
}
