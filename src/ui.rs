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

fn run_fzf(items: &[String], prompt: &str, height: &str, tac: bool) -> Result<Option<String>> {
    ensure_fzf()?;

    let mut cmd = Command::new("fzf");
    cmd.arg("--prompt")
        .arg(strip_ansi_codes(prompt))
        .arg("--height")
        .arg(height)
        .arg("--border")
        .arg("--info=inline")
        .arg("--margin=1,1,0,0")
        .arg("--reverse");

    if tac {
        cmd.arg("--tac");
    }

    let input = items
        .iter()
        .map(|item| strip_ansi_codes(item))
        .collect::<Vec<_>>()
        .join("\n");
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

pub fn select_from_list(items: &[String], prompt: &str, height: &str) -> Result<Option<String>> {
    run_fzf(items, prompt, height, true)
}

pub fn select_episode(total: u32, prompt: &str) -> Result<Option<u32>> {
    let Some(items) = episode_items(total) else {
        return prompt_manual_episode(prompt);
    };
    let selected = run_fzf(&items, prompt, "~30", false)?;
    match selected {
        None => Ok(None),
        Some(selected) => selected
            .parse::<u32>()
            .map(Some)
            .map_err(|e| anyhow!("{:?}", e)),
    }
}

pub fn select_post_episode_action() -> Result<Option<String>> {
    let options = [
        "Siguiente".to_string(),
        "Retroceder".to_string(),
        "Atras".to_string(),
    ];
    run_fzf(&options, "Selecciona una opción: ", "7", true)
}

const MAX_LIST_EPISODES: u32 = 5000;

fn episode_items(total: u32) -> Option<Vec<String>> {
    if total == 0 || total > MAX_LIST_EPISODES {
        return None;
    }
    Some((1..=total).map(|n| n.to_string()).collect())
}

fn prompt_manual_episode(prompt: &str) -> Result<Option<u32>> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<u32>()
        .map(Some)
        .map_err(|e| anyhow!("{:?}", e))
}

fn strip_ansi_codes(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episode_items_lists_reasonable_totals_and_rejects_huge_ones() {
        assert_eq!(
            episode_items(3),
            Some(vec!["1".to_string(), "2".to_string(), "3".to_string()])
        );
        assert_eq!(episode_items(0), None);
        assert_eq!(episode_items(5001), None);
        assert_eq!(episode_items(u32::MAX), None);
        assert!(episode_items(5000).is_some());
    }

    #[test]
    fn strips_ansi_sequences_from_fzf_values() {
        assert_eq!(strip_ansi_codes("\x1b[36mBleach\x1b[0m"), "Bleach");
        assert_eq!(strip_ansi_codes("plain"), "plain");
    }
}
