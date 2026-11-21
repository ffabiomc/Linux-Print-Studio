//! Read-only CUPS printer discovery using the standard lpstat command.
use std::collections::BTreeMap;
use std::path::Path;
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

/// Extract the queue identifier returned by lp, without trusting arbitrary text.
pub fn submitted_job_id(output: &str, printer: &str) -> Option<String> {
    let prefix = format!("{printer}-");
    output.split_whitespace().find_map(|word| {
        let candidate = word.trim_end_matches(|ch: char| !ch.is_ascii_digit());
        let number = candidate.strip_prefix(&prefix)?;
        if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) {
            Some(candidate.to_owned())
        } else {
            None
        }
    })
}

/// Submit one manual-duplex pass, explicitly disabling driver-side duplex.
pub fn submit_simplex(printer: &str, pdf: &Path) -> Result<String, String> {
    if printer.is_empty() || printer.starts_with('-') || printer.chars().any(char::is_control) {
        return Err("Invalid printer name".to_owned());
    }
    if !pdf.is_file() || pdf.extension().and_then(|value| value.to_str()) != Some("pdf") {
        return Err("A valid PDF is required".to_owned());
    }
    let output = Command::new("lp")
        .arg("-d")
        .arg(printer)
        .arg("-n")
        .arg("1")
        .arg("-o")
        .arg("sides=one-sided")
        .arg("--")
        .arg(pdf)
        .output()
        .map_err(|error| format!("Cannot run lp: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "CUPS rejected job: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Submit a PDF to CUPS only after the user confirms the job.
pub fn submit(
    printer: &str,
    pdf: &Path,
    copies: u32,
    choices: &[PrinterOption],
    selected: &BTreeMap<String, String>,
) -> Result<String, String> {
    if printer.is_empty() || printer.starts_with('-') || printer.chars().any(char::is_control) {
        return Err("Invalid printer name".to_owned());
    }
    if !pdf.is_file() || pdf.extension().and_then(|value| value.to_str()) != Some("pdf") {
        return Err("A valid PDF is required".to_owned());
    }
    if !(1..=999).contains(&copies) {
        return Err("Invalid copy count".to_owned());
    }
    let mut command = Command::new("lp");
    command
        .arg("-d")
        .arg(printer)
        .arg("-n")
        .arg(copies.to_string());
    for (key, value) in selected {
        if !choices
            .iter()
            .any(|option| option.key == *key && option.values.contains(value))
        {
            return Err("Unsupported printer option".to_owned());
        }
        command.arg("-o").arg(format!("{key}={value}"));
    }
    let output = command
        .arg("--")
        .arg(pdf)
        .output()
        .map_err(|error| format!("Cannot run lp: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "CUPS rejected job: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintJob {
    pub id: String,
    pub description: String,
}

pub fn parse_jobs(output: &str, printer: &str) -> Vec<PrintJob> {
    let prefix = format!("{printer}-");
    output
        .lines()
        .filter_map(|line| {
            let id = line.split_whitespace().next()?;
            let number = id.strip_prefix(&prefix)?;
            if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            Some(PrintJob {
                id: id.to_owned(),
                description: line.to_owned(),
            })
        })
        .collect()
}

pub fn jobs(printer: &str) -> Result<Vec<PrintJob>, String> {
    if printer.is_empty() || printer.starts_with('-') || printer.chars().any(char::is_control) {
        return Err("Invalid printer name".to_owned());
    }
    let output = run_lpstat(&["-o", printer])?;
    Ok(parse_jobs(&output, printer))
}

pub fn cancel_job(printer: &str, job_id: &str) -> Result<(), String> {
    let prefix = format!("{printer}-");
    let number = job_id.strip_prefix(&prefix).ok_or("Invalid job ID")?;
    if printer.is_empty()
        || printer.starts_with('-')
        || printer.chars().any(char::is_control)
        || number.is_empty()
        || !number.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("Invalid job ID".to_owned());
    }
    let output = Command::new("cancel")
        .arg(job_id)
        .output()
        .map_err(|error| format!("Cannot run cancel: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "CUPS cancellation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
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
        let parsed = parse_options("PageSize/Media Size: *A4 Letter Legal\n");
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
    fn parses_queue_for_selected_printer() {
        let jobs = parse_jobs(
            "Office-12 user 1024 Sat\nOther-4 user 2048 Sat\nOffice-13 user 512 Sat\n",
            "Office",
        );
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id, "Office-12");
        assert_eq!(jobs[1].id, "Office-13");
    }

    #[test]
    fn ignores_invalid_job_identifiers() {
        assert!(parse_jobs("Office-abc user\nOffice-12x user\n", "Office").is_empty());
    }

    #[test]
    fn extracts_submitted_job_id() {
        assert_eq!(
            submitted_job_id("request id is Office-123 (1 file(s))", "Office"),
            Some("Office-123".to_owned())
        );
        assert_eq!(submitted_job_id("unknown response", "Office"), None);
        assert_eq!(submitted_job_id("Other-123", "Office"), None);
    }

    #[test]
    fn empty_list_is_valid() {
        assert!(parse_names("").is_empty());
    }
}
