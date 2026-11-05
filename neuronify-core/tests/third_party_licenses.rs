use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use neuronify_core::third_party::{Package, ThirdParty};
use serde::Deserialize;

const BINARY: &str = "neuronify";
const RELEASE_TARGETS: [&str; 5] = [
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "wasm32-unknown-unknown",
];
const LIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/third-party-licenses.json"
);
const LICENSES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/licenses");
const WRITE: &str = "NEURONIFY_WRITE_LICENSES";
const LICENSE_FILE_NAMES: [&str; 6] = [
    "license",
    "licence",
    "copying",
    "copyright",
    "notice",
    "unlicense",
];
const EXTRA_LICENSE_FILES: [(&str, &[&str]); 1] = [(
    "epaint_default_fonts",
    &[
        "fonts/Hack-Regular.txt",
        "fonts/OFL.txt",
        "fonts/UFL.txt",
        "fonts/emoji-icon-font-mit-license.txt",
    ],
)];
const ASSETS: [(&str, &str, &str, &str); 1] = [(
    "DejaVu Sans",
    "2.37",
    "DejaVu Fonts License",
    "assets/DejaVu-LICENSE",
)];
const REPOSITORIES_WITHOUT_LICENSE_FILES: [&str; 2] = [
    "github.com/SSheldon/rust-dispatch",
    "github.com/lifthrasiir/hexf",
];
const FILES_THAT_ONLY_LINK_TO_LICENSES: [(&str, &str); 1] =
    [("github.com/madsmtm/objc2", "LICENSE.md")];

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<MetadataPackage>,
    resolve: Resolve,
}

#[derive(Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    version: String,
    license: Option<String>,
    license_file: Option<String>,
    repository: Option<String>,
    source: Option<String>,
    manifest_path: PathBuf,
}

#[derive(Deserialize)]
struct VcsInfo {
    git: Commit,
}

#[derive(Deserialize)]
struct Commit {
    sha1: String,
}

#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    id: String,
    deps: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    pkg: String,
    dep_kinds: Vec<DependencyKind>,
}

#[derive(Deserialize)]
struct DependencyKind {
    kind: Option<String>,
}

struct Texts {
    list: Vec<String>,
    indices: HashMap<String, usize>,
}

impl Texts {
    fn add(&mut self, text: &str) -> usize {
        let text = text.replace("\r\n", "\n").trim_end().to_owned();
        if let Some(&index) = self.indices.get(&text) {
            return index;
        }
        self.list.push(text.clone());
        self.indices.insert(text, self.list.len() - 1);
        self.list.len() - 1
    }
}

fn metadata(target: &str) -> Metadata {
    let cargo = std::env::var_os("CARGO").unwrap_or("cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--locked"])
        .args(["--filter-platform", target])
        .arg("--manifest-path")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"))
        .output()
        .expect("cargo metadata should run");
    assert!(
        output.status.success(),
        "cargo metadata failed for {target}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn shipped(metadata: &Metadata) -> Vec<&MetadataPackage> {
    let packages: HashMap<&str, &MetadataPackage> = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect();
    let nodes: HashMap<&str, &Node> = metadata
        .resolve
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let root = metadata
        .packages
        .iter()
        .find(|package| package.name == BINARY && package.source.is_none())
        .unwrap();
    let mut seen = HashSet::new();
    let mut unvisited = vec![root.id.as_str()];
    while let Some(id) = unvisited.pop() {
        if !seen.insert(id) {
            continue;
        }
        for dependency in &nodes[id].deps {
            let normal = dependency.dep_kinds.iter().any(|kind| kind.kind.is_none());
            if normal {
                unvisited.push(&dependency.pkg);
            }
        }
    }
    seen.into_iter()
        .map(|id| packages[id])
        .filter(|package| package.source.is_some())
        .collect()
}

fn is_license_file(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
    LICENSE_FILE_NAMES
        .iter()
        .any(|start| name.starts_with(start))
        && !name.ends_with(".rs")
}

fn all_files_in(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(all_files_in(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn files_in(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if !is_license_file(&path) {
            continue;
        }
        if path.is_dir() {
            files.extend(all_files_in(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn license_files(package: &MetadataPackage) -> Vec<PathBuf> {
    let folder = package.manifest_path.parent().unwrap();
    let mut files = Vec::new();
    for ancestor in folder.ancestors() {
        files = files_in(ancestor);
        let checkout_root = ancestor.join(".cargo-ok").exists();
        if !files.is_empty() || checkout_root {
            break;
        }
    }
    if let Some(license_file) = &package.license_file {
        let path = folder.join(license_file);
        if !files.contains(&path) {
            files.push(path);
        }
    }
    files
}

fn license_ids(expression: &str) -> impl Iterator<Item = &str> {
    expression
        .split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '/')
        .filter(|word| !word.is_empty() && !["AND", "OR", "WITH"].contains(word))
}

fn standard_text(id: &str) -> String {
    let path = Path::new(LICENSES).join(format!("{id}.txt"));
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "a package without license files is under {id}; \
             add its text from https://github.com/spdx/license-list-data as {}",
            path.display()
        )
    })
}

fn repository(package: &MetadataPackage) -> String {
    let url = package.repository.as_deref().unwrap_or_else(|| {
        panic!(
            "{} {} ships no license files and names no repository",
            package.name, package.version
        )
    });
    let address = url.split_once("://").map_or(url, |(_, address)| address);
    let parts: Vec<&str> = address.split('/').take(3).collect();
    format!(
        "{}/{}/{}",
        parts[0],
        parts[1],
        parts[2].trim_end_matches(".git")
    )
}

fn published_commit(package: &MetadataPackage) -> String {
    let path = package
        .manifest_path
        .parent()
        .unwrap()
        .join(".cargo_vcs_info.json");
    let info = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "{} {} ships no license files and no {}",
            package.name,
            package.version,
            path.display()
        )
    });
    serde_json::from_str::<VcsInfo>(&info).unwrap().git.sha1
}

struct Copied {
    files: Vec<PathBuf>,
    standard_texts: bool,
}

fn copied_from_repository(package: &MetadataPackage, used: &mut HashSet<PathBuf>) -> Copied {
    let repository = repository(package);
    if REPOSITORIES_WITHOUT_LICENSE_FILES.contains(&repository.as_str()) {
        return Copied {
            files: Vec::new(),
            standard_texts: true,
        };
    }
    let commit = published_commit(package);
    let folder = Path::new(LICENSES).join(&repository).join(&commit);
    assert!(
        folder.is_dir(),
        "{} {} ships no license files. Copy the license files from \
         https://{repository}/tree/{commit} into {}",
        package.name,
        package.version,
        folder.display()
    );
    let files = all_files_in(&folder);
    let standard_texts = files.iter().any(|file| {
        FILES_THAT_ONLY_LINK_TO_LICENSES.contains(&(
            repository.as_str(),
            &file.strip_prefix(&folder).unwrap().to_string_lossy(),
        ))
    });
    used.insert(folder);
    Copied {
        files,
        standard_texts,
    }
}

fn copied_folders() -> Vec<PathBuf> {
    let mut folders = vec![PathBuf::from(LICENSES)];
    for _ in ["host", "owner", "repository", "commit"] {
        folders = folders
            .iter()
            .flat_map(|folder| std::fs::read_dir(folder).unwrap())
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
    }
    folders.sort();
    folders
}

fn generate(used: &mut HashSet<PathBuf>) -> ThirdParty {
    let metadata: Vec<Metadata> = RELEASE_TARGETS.iter().map(|t| metadata(t)).collect();
    let mut shipped_packages = BTreeMap::new();
    for metadata in &metadata {
        for package in shipped(metadata) {
            let key = (package.name.to_lowercase(), package.version.clone());
            shipped_packages.insert(key, package);
        }
    }
    let mut texts = Texts {
        list: Vec::new(),
        indices: HashMap::new(),
    };
    let mut packages = Vec::new();
    for package in shipped_packages.values() {
        let license = package.license.clone().unwrap_or_else(|| {
            panic!(
                "{} {} has no SPDX license expression",
                package.name, package.version
            )
        });
        let mut files = license_files(package);
        let mut standard_texts = false;
        if files.is_empty() {
            let copied = copied_from_repository(package, used);
            files = copied.files;
            standard_texts = copied.standard_texts;
        }
        let mut indices: Vec<usize> = files
            .iter()
            .map(|path| texts.add(&std::fs::read_to_string(path).unwrap()))
            .collect();
        if standard_texts {
            for id in license_ids(&license) {
                indices.push(texts.add(&standard_text(id)));
            }
        }
        let folder = package.manifest_path.parent().unwrap();
        for (name, extra) in EXTRA_LICENSE_FILES {
            if package.name == name {
                for file in extra {
                    let text = std::fs::read_to_string(folder.join(file)).unwrap();
                    indices.push(texts.add(&text));
                }
            }
        }
        let mut listed = HashSet::new();
        indices.retain(|&index| listed.insert(index));
        packages.push(Package {
            name: package.name.clone(),
            version: package.version.clone(),
            license,
            texts: indices,
        });
    }
    for (name, version, license, file) in ASSETS {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
        let text = std::fs::read_to_string(path).unwrap();
        packages.push(Package {
            name: name.to_owned(),
            version: version.to_owned(),
            license: license.to_owned(),
            texts: vec![texts.add(&text)],
        });
    }
    packages.sort_by_key(|package| (package.name.to_lowercase(), package.version.clone()));
    let mut third_party = ThirdParty {
        packages,
        texts: Vec::new(),
    };
    renumber_texts_in_order_of_use(&mut third_party, texts.list);
    third_party
}

fn renumber_texts_in_order_of_use(third_party: &mut ThirdParty, texts: Vec<String>) {
    let mut renumbered = HashMap::new();
    for package in &mut third_party.packages {
        for index in &mut package.texts {
            let next = renumbered.len();
            let new = *renumbered.entry(*index).or_insert(next);
            if new == next {
                third_party.texts.push(texts[*index].clone());
            }
            *index = new;
        }
    }
}

fn to_json(third_party: &ThirdParty) -> String {
    let lines = |items: Vec<String>| items.join(",\n");
    let packages = third_party
        .packages
        .iter()
        .map(|package| serde_json::to_string(package).unwrap())
        .collect();
    let texts = third_party
        .texts
        .iter()
        .map(|text| serde_json::to_string(text).unwrap())
        .collect();
    format!(
        "{{\n\"packages\": [\n{}\n],\n\"texts\": [\n{}\n]\n}}\n",
        lines(packages),
        lines(texts)
    )
}

#[test]
fn the_third_party_licenses_cover_every_package_we_ship() {
    let mut used = HashSet::new();
    let generated = to_json(&generate(&mut used));
    let unused: Vec<PathBuf> = copied_folders()
        .into_iter()
        .filter(|folder| !used.contains(folder))
        .collect();
    assert!(
        unused.is_empty(),
        "No package we ship uses the license files in {unused:?}. Remove them."
    );
    if std::env::var_os(WRITE).is_some() {
        std::fs::write(LIST, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(LIST).unwrap_or_default();
    assert!(
        committed == generated,
        "{LIST} is out of date. Run `{WRITE}=1 cargo test -p neuronify-core --test third_party_licenses` to update it."
    );
}
