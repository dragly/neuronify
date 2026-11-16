use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use super::SavedSimulation;

const EXTENSION: &str = ".neuronify";
const ESCAPED: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '%'];

pub struct Store {
    directory: PathBuf,
    files: HashMap<String, PathBuf>,
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
        Store {
            directory,
            files: HashMap::new(),
        }
    }

    pub fn read_all(&mut self) -> Vec<SavedSimulation> {
        let Ok(entries) = std::fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let read: Vec<(PathBuf, SavedSimulation)> = entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = name_of(path.file_name()?.to_str()?)?;
                read(&path, name)
                    .map_err(|e| log::error!("Could not read {}: {e}", path.display()))
                    .ok()
                    .map(|simulation| (path, simulation))
            })
            .collect();
        read.into_iter()
            .map(|(path, simulation)| {
                self.files.insert(simulation.id.clone(), path);
                simulation
            })
            .collect()
    }

    pub fn write(&mut self, simulation: &SavedSimulation) -> Result<(), String> {
        let path = match self.files.get(&simulation.id) {
            Some(path) => path.clone(),
            None => self.free_path(&simulation.name, &simulation.id),
        };
        std::fs::create_dir_all(&self.directory)
            .and_then(|()| std::fs::write(&path, &simulation.bytes))
            .map_err(|error| format!("Could not write {}: {error}", path.display()))?;
        self.files.insert(simulation.id.clone(), path);
        Ok(())
    }

    pub fn rename(&mut self, old_name: &str, simulation: &SavedSimulation) -> Result<(), String> {
        let from = match self.files.get(&simulation.id) {
            Some(path) => path.clone(),
            None => self.path(old_name),
        };
        let wanted = self.path(&simulation.name);
        let to = if self.is_free(&wanted, &simulation.id) || is_same_file(&wanted, &from) {
            wanted
        } else {
            self.free_path(&simulation.name, &simulation.id)
        };
        std::fs::rename(&from, &to).map_err(|error| {
            format!(
                "Could not rename {} to {}: {error}",
                from.display(),
                to.display()
            )
        })?;
        self.files.insert(simulation.id.clone(), to.clone());
        std::fs::write(&to, &simulation.bytes)
            .map_err(|error| format!("Could not write {}: {error}", to.display()))
    }

    pub fn delete(&mut self, simulation: &SavedSimulation) {
        let path = match self.files.remove(&simulation.id) {
            Some(path) => path,
            None => self.path(&simulation.name),
        };
        if let Err(error) = std::fs::remove_file(&path) {
            log::error!("Could not delete {}: {error}", path.display());
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(file_name(name))
    }

    fn free_path(&self, name: &str, id: &str) -> PathBuf {
        (1..)
            .map(|number| match number {
                1 => self.path(name),
                _ => self.path(&format!("{name} {number}")),
            })
            .find(|path| self.is_free(path, id))
            .unwrap_or_else(|| self.path(name))
    }

    fn is_free(&self, path: &Path, id: &str) -> bool {
        let lowercase = path.to_string_lossy().to_lowercase();
        !path.exists()
            && !self.files.iter().any(|(owner, owned)| {
                owner != id && owned.to_string_lossy().to_lowercase() == lowercase
            })
    }
}

#[cfg(unix)]
fn is_same_file(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(a), Ok(b)) => (a.dev(), a.ino()) == (b.dev(), b.ino()),
        _ => false,
    }
}

#[cfg(not(unix))]
fn is_same_file(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
}

fn read(path: &Path, name: String) -> std::io::Result<SavedSimulation> {
    let modified = std::fs::metadata(path)?.modified()?;
    Ok(SavedSimulation {
        id: format!("{:016x}", rand::random::<u64>()),
        name,
        description: String::new(),
        modified: DateTime::<Utc>::from(modified),
        bytes: std::fs::read(path)?,
    })
}

fn file_name(name: &str) -> String {
    let mut file_name = String::new();
    for (index, character) in name.chars().enumerate() {
        let device = index == 0 && is_windows_device(name);
        if device || character.is_control() || ESCAPED.contains(&character) {
            file_name.push_str(&format!("%{:02X}", character as u32));
        } else {
            file_name.push(character);
        }
    }
    file_name + EXTENSION
}

fn is_windows_device(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit())
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
            let _ = std::fs::remove_dir_all(&self.0).or_else(|_| std::fs::remove_file(&self.0));
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
    fn windows_device_names_get_their_first_letter_escaped() {
        for name in ["CON", "con", "Nul", "COM1", "lpt9", "aux.backup"] {
            let file_name = file_name(name);
            assert!(file_name.starts_with('%'), "{file_name}");
            assert_eq!(name_of(&file_name).as_deref(), Some(name));
        }
        for name in ["Console", "COM", "COMA", "Nullcline", "LPT10"] {
            assert!(!file_name(name).starts_with('%'), "{name}");
        }
    }

    #[test]
    fn the_longest_name_still_makes_a_file_name_most_file_systems_allow() {
        let mut library = Library::default();
        let id = library.create(&"\u{1F9E0}".repeat(300), "", Vec::new());
        let name = &library.get(&id).unwrap().name;
        assert!(file_name(&format!("{name} 999")).len() <= 255);
    }

    #[test]
    fn a_failed_rename_keeps_the_old_name_and_says_why() {
        let directory = TemporaryDirectory::new("rename");
        let mut library = Library::in_folder(Store::new(directory.0.clone()));
        let id = library.create("Leaky", "", vec![1]);
        std::fs::remove_file(directory.0.join(file_name("Leaky"))).unwrap();

        library.edit_details(&id, "Renamed", "");

        assert_eq!(library.get(&id).unwrap().name, "Leaky");
        assert!(library
            .take_problem()
            .is_some_and(|problem| problem.contains("rename")));
    }

    #[test]
    fn a_failed_write_is_reported() {
        let directory = TemporaryDirectory::new("write");
        std::fs::write(&directory.0, b"not a folder").unwrap();
        let mut library = Library::in_folder(Store::new(directory.0.clone()));

        library.create("Leaky", "", vec![1]);

        assert!(library
            .take_problem()
            .is_some_and(|problem| problem.contains("write")));
    }

    #[test]
    fn simulations_survive_a_restart() {
        let directory = TemporaryDirectory::new("restart");
        let mut library = Library::in_folder(Store::new(directory.0.clone()));
        let kept = library.create("Tutorial 1: Intro", "", vec![1, 2, 3]);
        let renamed = library.create("Leaky", "", vec![4]);
        let deleted = library.create("Burst", "", vec![5]);
        Autosave::saved(&library, kept).save(&mut library, vec![6, 7]);
        library.edit_details(&renamed, "Renamed", "");
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

    fn file(name: &str) -> Vec<u8> {
        let meta = crate::serialization::Meta {
            name: name.to_owned(),
            ..Default::default()
        };
        crate::serialization::save(
            &hecs::World::new(),
            crate::components::Milliseconds::ZERO,
            crate::constants::DEFAULT_SPEED,
            &meta,
        )
        .unwrap()
    }

    fn files_in(directory: &Path) -> Vec<String> {
        let mut files: Vec<String> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        files
    }

    #[test]
    fn a_file_renamed_by_hand_keeps_its_name_and_its_file() {
        let directory = TemporaryDirectory::new("by-hand");
        let mut library = Library::in_folder(Store::new(directory.0.clone()));
        library.create("Leaky", "", file("Leaky"));
        std::fs::rename(
            directory.0.join("Leaky.neuronify"),
            directory.0.join("Mine.neuronify"),
        )
        .unwrap();

        let mut reopened = Library::in_folder(Store::new(directory.0.clone()));
        let simulation = reopened.simulations()[0].clone();
        assert_eq!(simulation.name, "Leaky");
        Autosave::saved(&reopened, simulation.id).save(&mut reopened, file("Leaky"));

        assert_eq!(files_in(&directory.0), ["Mine.neuronify"]);
    }

    #[test]
    fn simulations_never_overwrite_a_file_they_did_not_write() {
        let directory = TemporaryDirectory::new("overwrite");
        std::fs::create_dir_all(&directory.0).unwrap();
        std::fs::write(directory.0.join("Other.neuronify"), file("Leaky")).unwrap();
        let mut library = Library::in_folder(Store::new(directory.0.clone()));

        library.create("Other", "", file("Other"));
        let renamed = library.create("Burst", "", file("Burst"));
        library.edit_details(&renamed, "other ", "");

        assert_eq!(
            files_in(&directory.0),
            [
                "Other 2.neuronify",
                "Other.neuronify",
                "other 2 2.neuronify"
            ]
        );
        let reopened = Library::in_folder(Store::new(directory.0.clone()));
        let mut names: Vec<&str> = reopened
            .simulations()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        names.sort();
        assert_eq!(names, ["Leaky", "Other", "other 2"]);
    }

    #[test]
    fn changing_only_the_case_of_a_name_renames_the_file() {
        let directory = TemporaryDirectory::new("case");
        let mut library = Library::in_folder(Store::new(directory.0.clone()));
        let id = library.create("Leaky", "", file("Leaky"));
        library.edit_details(&id, "leaky", "Leaks.");
        assert_eq!(files_in(&directory.0), ["leaky.neuronify"]);
        let reopened = Library::in_folder(Store::new(directory.0.clone()));
        let simulation = &reopened.simulations()[0];
        assert_eq!(
            (simulation.name.as_str(), simulation.description.as_str()),
            ("leaky", "Leaks.")
        );
    }
}
