//! Local named manual-duplex profiles. No printer discovery is implied.
use crate::duplex::FeedOrder;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterProfile {
    pub name: String,
    pub back_feed: FeedOrder,
}

pub fn config_path() -> Result<PathBuf, String> {
    let root = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or("Cannot find user configuration directory")?;
    Ok(root.join("linux-print-studio").join("printer-profiles.txt"))
}

pub fn load(path: &Path) -> Result<Vec<PrinterProfile>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse(&contents)
}

fn parse(contents: &str) -> Result<Vec<PrinterProfile>, String> {
    let mut profiles = Vec::new();
    for line in contents.lines() {
        let (order, name) = line.split_once('\t').ok_or("Invalid profile entry")?;
        let back_feed = match order {
            "same" => FeedOrder::Same,
            "reversed" => FeedOrder::Reversed,
            _ => return Err("Invalid back-side order".to_owned()),
        };
        validate_name(name)?;
        if profiles.iter().any(|profile: &PrinterProfile| profile.name == name) {
            return Err("Duplicate printer profile".to_owned());
        }
        profiles.push(PrinterProfile {
            name: name.to_owned(),
            back_feed,
        });
    }
    Ok(profiles)
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty()
        || name.len() > 100
        || name.chars().any(|character| character.is_control())
    {
        return Err("Profile name must be 1-100 bytes without control characters".to_owned());
    }
    Ok(())
}

pub fn save(path: &Path, profiles: &[PrinterProfile]) -> Result<(), String> {
    let mut content = String::new();
    for profile in profiles {
        validate_name(&profile.name)?;
        let order = match profile.back_feed {
            FeedOrder::Same => "same",
            FeedOrder::Reversed => "reversed",
        };
        content.push_str(order);
        content.push('\t');
        content.push_str(&profile.name);
        content.push('\n');
    }
    let parent = path.parent().ok_or("Invalid configuration path")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temp = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    fs::write(temp.path(), content).map_err(|error| error.to_string())?;
    temp.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.txt");
        let profiles = vec![PrinterProfile {
            name: "My printer".to_owned(),
            back_feed: FeedOrder::Reversed,
        }];
        save(&path, &profiles).unwrap();
        assert_eq!(load(&path).unwrap(), profiles);
    }

    #[test]
    fn rejects_invalid_names() {
        assert!(validate_name("a\nb").is_err());
        assert!(validate_name("").is_err());
    }
}
