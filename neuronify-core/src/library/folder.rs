use std::fmt::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use super::SavedSimulation;

const EXTENSION: &str = ".neuronify";
const ESCAPED: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '%'];

pub struct Store {
    directory: PathBuf,
}

#[cfg(target_os = "windows")]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join("Neuronify"))
}

#[cfg(target_os = "macos")]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support/Neuronify"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn data_directory() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .filter(|directory| !directory.is_empty())
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".local/share")))?;
    Some(data.join("neuronify"))
}

impl Store {
    pub fn in_data_directory() -> Option<Store> {
        Some(Store::new(data_directory()?.join("simulations")))
    }

    pub fn new(directory: PathBuf) -> Store {
        Store { directory }
    }

    pub fn read_all(&self) -> Vec<SavedSimulation> {
        let Ok(entries) = std::fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = name_of(path.file_name()?.to_str()?)?;
                read(&path, name)
                    .map_err(|e| log::error!("Could not read {}: {e}", path.display()))
                    .ok()
            })
            .collect()
    }

    pub fn write(&self, simulation: &SavedSimulation) {
        let path = self.path(&simulation.name);
        let written = std::fs::create_dir_all(&self.directory)
            .and_then(|()| std::fs::write(&path, &simulation.bytes));
        if let Err(error) = written {
            log::error!("Could not save {}: {error}", path.display());
        }
    }

    pub fn rename(&self, old_name: &str, simulation: &SavedSimulation) {
        let (from, to) = (self.path(old_name), self.path(&simulation.name));
        if let Err(error) = std::fs::rename(&from, &to) {
            log::error!(
                "Could not rename {} to {}: {error}",
                from.display(),
                to.display()
            );
        }
    }

    pub fn delete(&self, simulation: &SavedSimulation) {
        let path = self.path(&simulation.name);
        if let Err(error) = std::fs::remove_file(&path) {
            log::error!("Could not delete {}: {error}", path.display());
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(file_name(name))
    }
}

fn read(path: &Path, name: String) -> std::io::Result<SavedSimulation> {
    let modified = std::fs::metadata(path)?.modified()?;
    Ok(SavedSimulation {
        id: format!("{:016x}", rand::random::<u64>()),
        name,
        modified: DateTime::<Utc>::from(modified),
        bytes: std::fs::read(path)?,
    })
}

fn file_name(name: &str) -> String {
    let mut file_name = String::new();
    for character in name.chars() {
        if character.is_control() || ESCAPED.contains(&character) {
            write!(file_name, "%{:02X}", character as u32).unwrap();
        } else {
            file_name.push(character);
        }
    }
    file_name + EXTENSION
}

fn name_of(file_name: &str) -> Option<String> {
    let mut rest = file_name.strip_suffix(EXTENSION)?;
    let mut name = String::new();
    while let Some(index) = rest.find('%') {
        name.push_str(&rest[..index]);
        let code = rest
            .get(index + 1..index + 3)
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match code {
            Some(code) => {
                name.push(char::from(code));
                rest = &rest[index + 3..];
            }
            None => {
                name.push('%');
                rest = &rest[index + 1..];
            }
        }
    }
    name.push_str(rest);
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{Autosave, Library};

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new(test: &str) -> TemporaryDirectory {
            let path = std::env::temp_dir().join(format!(
                "neuronify-{test}-{}-{:x}",
                std::process::id(),
                rand::random::<u32>()
            ));
            TemporaryDirectory(path)
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn file_names_keep_any_name() {
        for name in [
            "Tutorial 1: Intro",
            "a/b\\c",
            "50% <more>",
            "tab\there",
            "plain",
        ] {
            let file_name = file_name(name);
            assert!(!file_name.contains(['/', '\\', ':', '\t']), "{file_name}");
            assert_eq!(name_of(&file_name).as_deref(), Some(name));
        }
        assert_eq!(
            file_name("Tutorial 1: Intro"),
            "Tutorial 1%3A Intro.neuronify"
        );
        assert_eq!(name_of("100%.neuronify").as_deref(), Some("100%"));
        assert_eq!(name_of("notes.txt"), None);
    }

    #[test]
    fn simulations_survive_a_restart() {
        let directory = TemporaryDirectory::new("restart");
        let mut library = Library::in_folder(Store::new(directory.0.clone()));
        let kept = library.create("Tutorial 1: Intro", vec![1, 2, 3]);
        let renamed = library.create("Leaky", vec![4]);
        let deleted = library.create("Burst", vec![5]);
        Autosave::saved(&library, kept).save(&mut library, vec![6, 7]);
        library.rename(&renamed, "Renamed");
        library.delete(&deleted);

        let reopened = Library::in_folder(Store::new(directory.0.clone()));
        let mut saved: Vec<(&str, &[u8])> = reopened
            .simulations()
            .iter()
            .map(|s| (s.name.as_str(), &s.bytes[..]))
            .collect();
        saved.sort();
        assert_eq!(
            saved,
            [("Renamed", &[4][..]), ("Tutorial 1: Intro", &[6, 7][..])]
        );
    }
}
