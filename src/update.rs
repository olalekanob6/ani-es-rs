use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
};

const REPO: &str = "olalekanob6/ani-es-rs";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn asset_name_matches_host_os_and_architecture() {
        assert_eq!(
            asset_name("linux", "x86_64"),
            Some("ani-es-rs-linux-x86_64")
        );
        assert_eq!(
            asset_name("macos", "aarch64"),
            Some("ani-es-rs-macos-aarch64")
        );
        assert_eq!(
            asset_name("windows", "x86_64"),
            Some("ani-es-rs-windows-x86_64.exe")
        );
        assert_eq!(asset_name("linux", "aarch64"), None);
        assert_eq!(asset_name("macos", "x86_64"), None);
        assert_eq!(asset_name("plan9", "x86_64"), None);
        assert_eq!(asset_name("linux", "mips"), None);
    }
    #[test]
    fn version_comparison_never_offers_a_downgrade() {
        assert_eq!(
            compare_versions("0.1.3", "0.1.2").unwrap(),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.0.0", "0.99.99").unwrap(),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("0.1.2", "0.1.2").unwrap(),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_versions("0.1.1", "0.1.2").unwrap(),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_versions("v0.1.3", "0.1.2").unwrap(),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn version_comparison_rejects_malformed_versions() {
        assert!(compare_versions("not-a-version", "0.1.2").is_err());
    }

    #[test]
    fn checksum_manifest_requires_matching_sha256_asset_line() {
        let digest = "a".repeat(64);
        assert_eq!(
            checksum_for(
                &format!("{digest}  ani-es-rs-linux-x86_64\n"),
                "ani-es-rs-linux-x86_64"
            )
            .unwrap(),
            digest
        );
        assert!(checksum_for("bad  ani-es-rs-linux-x86_64\n", "ani-es-rs-linux-x86_64").is_err());
        assert!(checksum_for(&format!("{digest}  other\n"), "ani-es-rs-linux-x86_64").is_err());
    }
}

fn asset_name(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "x86_64") => Some("ani-es-rs-linux-x86_64"),
        ("macos", "aarch64") => Some("ani-es-rs-macos-aarch64"),
        ("windows", "x86_64") => Some("ani-es-rs-windows-x86_64.exe"),
        _ => None,
    }
}
fn compare_versions(left: &str, right: &str) -> Result<std::cmp::Ordering> {
    fn parts(version: &str) -> Result<[u64; 3]> {
        let version = version.strip_prefix('v').unwrap_or(version);
        let core = version.split(['-', '+']).next().unwrap_or_default();
        let mut segments = core.split('.');
        let parse = |segment: Option<&str>| -> Result<u64> {
            segment
                .context("Versión inválida en GitHub Releases")?
                .parse::<u64>()
                .context("Versión inválida en GitHub Releases")
        };
        let result = [
            parse(segments.next())?,
            parse(segments.next())?,
            parse(segments.next())?,
        ];
        if segments.next().is_some() {
            bail!("Versión inválida en GitHub Releases");
        }
        Ok(result)
    }

    Ok(parts(left)?.cmp(&parts(right)?))
}

fn checksum_for(manifest: &str, asset: &str) -> Result<String> {
    for line in manifest.lines() {
        let mut fields = line.split_whitespace();
        if fields
            .next()
            .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
            && fields.next() == Some(asset)
        {
            return Ok(line.split_whitespace().next().unwrap().to_ascii_lowercase());
        }
    }
    bail!("SHA256SUMS no contiene un checksum válido para {asset}")
}

#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}
#[derive(serde::Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn run(check: bool, yes: bool) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("ani-es-rs/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let release: Release = serde_json::from_str(
        &client
            .get(format!(
                "https://api.github.com/repos/{REPO}/releases/latest"
            ))
            .send()
            .context("No se pudo consultar GitHub Releases")?
            .error_for_status()
            .context("No hay una release publicada disponible o GitHub no está disponible")?
            .text()?,
    )?;
    let latest = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let current = env!("CARGO_PKG_VERSION");
    match compare_versions(latest, current)? {
        std::cmp::Ordering::Equal => {
            println!("ani-es-rs {current} está al día.");
            return Ok(());
        }
        std::cmp::Ordering::Less => {
            println!("La versión instalada ({current}) es más nueva que la última release publicada ({latest}); no se hará downgrade.");
            return Ok(());
        }
        std::cmp::Ordering::Greater => {}
    }
    println!("Nueva versión disponible: {latest} (actual: {current}).");
    if check {
        return Ok(());
    }
    if managed_by_pacman()? {
        println!("Instalación administrada por pacman/AUR. Actualiza con pacman -S ani-es-rs o tu helper AUR.");
        return Ok(());
    }
    if cfg!(windows) {
        println!("La actualización automática no está habilitada en Windows; descarga y reemplaza el ejecutable manualmente desde GitHub Releases.");
        return Ok(());
    }
    let Some(asset) = asset_name(std::env::consts::OS, std::env::consts::ARCH) else {
        println!("Actualización automática no disponible para esta plataforma; descarga manualmente desde GitHub Releases.");
        return Ok(());
    };
    if !yes {
        print!("¿Instalar la actualización? [s/N] ");
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(
            answer.trim().to_ascii_lowercase().as_str(),
            "s" | "si" | "sí" | "y" | "yes"
        ) {
            println!("Actualización cancelada.");
            return Ok(());
        }
    }
    let binary = release.assets.iter().find(|a| a.name == asset).context(
        "El release no incluye el binario compatible; descarga manualmente desde GitHub Releases",
    )?;
    let sums = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS")
        .context("El release no incluye SHA256SUMS")?;
    let manifest = client
        .get(&sums.browser_download_url)
        .send()?
        .error_for_status()?
        .text()?;
    let expected = checksum_for(&manifest, asset)?;
    const MAX_BINARY_SIZE: u64 = 128 * 1024 * 1024;
    let mut response = client
        .get(&binary.browser_download_url)
        .send()?
        .error_for_status()?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_BINARY_SIZE)
    {
        bail!("El binario descargado excede el tamaño máximo permitido");
    }
    let mut bytes = Vec::new();
    response
        .by_ref()
        .take(MAX_BINARY_SIZE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BINARY_SIZE {
        bail!("El binario descargado excede el tamaño máximo permitido");
    }
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        bail!("El SHA-256 descargado no coincide; no se instaló la actualización");
    }
    let current_exe = std::env::current_exe()?;
    let mut temp = current_exe.clone();
    temp.set_extension("ani-es-rs-update.tmp");
    fs::write(&temp, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o755))?;
        fs::rename(&temp, &current_exe)
            .context("No se pudo reemplazar el ejecutable; descarga manualmente")?;
    }
    if cfg!(windows) {
        let _ = fs::remove_file(&temp);
        bail!("La actualización automática no está habilitada en Windows: cierra el programa y descarga/reemplaza el ejecutable manualmente desde GitHub Releases");
    }
    println!("Actualizado a {latest}.");
    Ok(())
}
fn managed_by_pacman() -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        let path = std::env::current_exe()?.to_string_lossy().into_owned();
        if which::which("pacman").is_ok() {
            return Ok(std::process::Command::new("pacman")
                .args(["-Qo", &path])
                .output()?
                .status
                .success());
        }
    }
    Ok(false)
}
