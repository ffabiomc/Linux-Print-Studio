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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterDetails {
    pub status: String,
    pub options: Vec<String>,
    pub choices: Vec<PrinterOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterOption {
    pub key: String,
    pub label: String,
    pub values: Vec<String>,
    pub default: Option<String>,
}

pub fn parse_options(output: &str) -> Vec<PrinterOption> {
    output
        .lines()
        .filter_map(|line| {
            let (header, choices) = line.split_once(':')?;
            let (key, label) = header.trim().split_once('/')?;
            if key.is_empty() {
                return None;
            }
            let mut values = Vec::new();
            let mut default = None;
            for token in choices.split_whitespace() {
                let value = token.trim_start_matches('*');
                if value.is_empty() {
                    continue;
                }
                if token.starts_with('*') {
                    default = Some(value.to_owned());
                }
                values.push(value.to_owned());
            }
            if values.is_empty() {
                return None;
            }
            Some(PrinterOption {
                key: key.to_owned(),
                label: label.to_owned(),
                values,
                default,
            })
        })
        .collect()
}

pub fn details(printer: &str) -> Result<PrinterDetails, String> {
    if printer.is_empty() || printer.starts_with('-') || printer.chars().any(char::is_control) {
        return Err("Invalid printer name".to_owned());
    }
    let status = run_lpstat(&["-p", printer])?;
    let output = Command::new("lpoptions")
        .args(["-p", printer, "-l"])
        .output()
        .map_err(|error| format!("Cannot run lpoptions: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "lpoptions failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let choices = parse_options(&text);
    let options = text.lines().map(str::to_owned).collect();
    Ok(PrinterDetails {
        status: status.trim().to_owned(),
        options,
        choices,
    })
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
    fn parses_driver_choices() {
        let parsed = parse_options("PageSize/Media Size: *A4 Letter Legal\\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].key, "PageSize");
        assert_eq!(parsed[0].default.as_deref(), Some("A4"));
        assert_eq!(parsed[0].values, vec!["A4", "Letter", "Legal"]);
    }

    #[test]
    fn ignores_malformed_choices() {
        assert!(parse_options("not a valid option").is_empty());
    }

    #[test]
    fn empty_list_is_valid() {
        assert!(parse_names("").is_empty());
    }
}
