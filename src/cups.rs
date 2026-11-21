//! Read-only CUPS printer discovery using the standard lpstat command.
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printers {
    pub names: Vec<String>,
    pub default: Option<String>,
}

fn run_lpstat(args: &[&str]) -> Result<String, String> {
    let output = Command::new("lpstat")
        .args(args)
        .output()
        .map_err(|error| format!("Cannot run lpstat: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "lpstat {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

pub fn discover() -> Result<Printers, String> {
    let output = run_lpstat(&["-e"])?;
    let names = parse_names(&output);
    let default = run_lpstat(&["-d"]).ok().and_then(|text| {
        text.lines()
            .next()
            .and_then(|line| line.rsplit_once(':'))
            .map(|(_, name)| name.trim().to_owned())
            .filter(|name| names.contains(name))
    });
    Ok(Printers { names, default })
}

fn parse_names(output: &str) -> Vec<String> {
    let mut names = Vec::new();
    for name in output.lines().map(str::trim) {
        if !name.is_empty() && !names.iter().any(|existing| existing == name) {
            names.push(name.to_owned());
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_printer_names() {
        assert_eq!(
            parse_names("Office_Laser\nHome_Printer\nOffice_Laser\n\n"),
            vec!["Office_Laser", "Home_Printer"]
        );
    }

    #[test]
    fn empty_list_is_valid() {
        assert!(parse_names("").is_empty());
    }
}
