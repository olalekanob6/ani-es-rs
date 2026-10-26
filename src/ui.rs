use anyhow::{anyhow, Result};
use std::io::Write;
use std::process::Command;

pub fn ensure_fzf() -> Result<()> {
    if which::which("fzf").is_err() {
        return Err(anyhow!(
            "fzf no está instalado. Instálalo para usar la interfaz interactiva."
        ));
    }
    Ok(())
}

pub fn prompt_search(query: &str) -> Result<String> {
    let input = if query.is_empty() {
        print!("Buscar Anime: ");
        std::io::stdout().flush()?;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        buf.trim().to_string()
    } else {
        query.to_string()
    };
    Ok(input)
}

pub fn select_from_list(items: &[String], prompt: &str, height: &str) -> Result<Option<String>> {
    ensure_fzf()?;

    let mut cmd = Command::new("fzf");
    cmd.arg("--prompt")
        .arg(prompt)
        .arg("--height")
        .arg(height)
        .arg("--border")
        .arg("--info=inline")
        .arg("--margin=1,1,0,0")
        .arg("--reverse");

    let input = items.join("\n");
    let mut child = cmd
        .arg("--tac")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No se pudo abrir stdin de fzf"))?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None);
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

pub fn select_episode(total: u32, prompt: &str) -> Result<Option<u32>> {
    ensure_fzf()?;

    let items: Vec<String> = (1..=total).map(|n| n.to_string()).collect();
    let mut cmd = Command::new("fzf");
    cmd.arg("--prompt")
        .arg(prompt)
        .arg("--height")
        .arg("~30")
        .arg("--border")
        .arg("--info=inline")
        .arg("--margin=1,1,0,0")
        .arg("--reverse");

    let input = items.join("\n");
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No se pudo abrir stdin de fzf"))?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None);
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        Ok(None)
    } else {
        selected
            .parse::<u32>()
            .map(Some)
            .map_err(|e| anyhow!("{:?}", e))
    }
}

pub fn select_post_episode_action() -> Result<Option<String>> {
    ensure_fzf()?;

    let options = vec![
        "Siguiente".to_string(),
        "Retroceder".to_string(),
        "Atras".to_string(),
    ];

    let mut cmd = Command::new("fzf");
    cmd.arg("--prompt")
        .arg("Selecciona una opción: ")
        .arg("--height")
        .arg("7")
        .arg("--border")
        .arg("--info=inline")
        .arg("--margin=1,1,0,0")
        .arg("--reverse")
        .arg("--tac");

    let input = options.join("\n");
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No se pudo abrir stdin de fzf"))?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None);
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}
