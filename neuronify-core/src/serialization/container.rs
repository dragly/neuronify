use serde::{Deserialize, Serialize};

pub const LONGEST_INDEX: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part<'a> {
    pub kind: &'a str,
    pub version: u32,
    pub bytes: &'a [u8],
}

#[derive(Serialize, Deserialize)]
struct Entry<'a> {
    kind: &'a str,
    version: u32,
    length: u64,
}

pub fn write(parts: &[Part]) -> Result<Vec<u8>, String> {
    let entries: Vec<Entry> = parts
        .iter()
        .map(|part| Entry {
            kind: part.kind,
            version: part.version,
            length: part.bytes.len() as u64,
        })
        .collect();
    let index = postcard::to_stdvec(&entries).map_err(|e| format!("Could not save: {e}"))?;
    let index_length =
        u32::try_from(index.len()).map_err(|_| "Could not save: the index is too long")?;
    let mut bytes = index_length.to_le_bytes().to_vec();
    bytes.extend_from_slice(&index);
    for part in parts {
        bytes.extend_from_slice(part.bytes);
    }
    Ok(bytes)
}

pub fn read(bytes: &[u8]) -> Result<Vec<Part<'_>>, String> {
    let (index_length, rest) = bytes
        .split_first_chunk()
        .ok_or("The save file ends before its index")?;
    let index_length = u32::from_le_bytes(*index_length) as usize;
    if index_length > LONGEST_INDEX {
        return Err("The index of the save file is too long".to_owned());
    }
    let (index, mut rest) = rest
        .split_at_checked(index_length)
        .ok_or("The save file ends before the end of its index")?;
    let entries: Vec<Entry> = postcard::from_bytes(index)
        .map_err(|e| format!("Could not read the index of the save file: {e}"))?;
    let mut parts: Vec<Part> = Vec::with_capacity(entries.len());
    for entry in entries {
        if parts.iter().any(|part| part.kind == entry.kind) {
            return Err(format!(
                "The save file has more than one part called {}",
                entry.kind
            ));
        }
        let (part, after) = usize::try_from(entry.length)
            .ok()
            .and_then(|length| rest.split_at_checked(length))
            .ok_or_else(|| format!("The save file ends before the end of its {}", entry.kind))?;
        parts.push(Part {
            kind: entry.kind,
            version: entry.version,
            bytes: part,
        });
        rest = after;
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    const META: Part = Part {
        kind: "meta",
        version: 1,
        bytes: &[1, 2, 3],
    };
    const WORLD: Part = Part {
        kind: "world",
        version: 18,
        bytes: &[4, 5],
    };

    #[test]
    fn reads_the_parts_it_wrote() {
        let bytes = write(&[META, WORLD]).unwrap();
        assert_eq!(read(&bytes).unwrap(), [META, WORLD]);
    }

    #[test]
    fn reads_parts_of_kinds_it_has_never_seen() {
        let thumbnail = Part {
            kind: "thumbnail",
            version: 7,
            bytes: &[9; 300],
        };
        let bytes = write(&[META, thumbnail, WORLD]).unwrap();
        assert_eq!(read(&bytes).unwrap(), [META, thumbnail, WORLD]);
    }

    #[test]
    fn rejects_two_parts_of_the_same_kind() {
        let bytes = write(&[META, META]).unwrap();
        assert!(read(&bytes).is_err_and(|e| e.contains("more than one part called meta")));
    }

    #[test]
    fn rejects_parts_longer_than_the_file() {
        let mut bytes = write(&[META, WORLD]).unwrap();
        bytes.pop();
        assert!(read(&bytes).is_err_and(|e| e.contains("before the end of its world")));
    }

    #[test]
    fn rejects_an_index_longer_than_the_file() {
        let mut bytes = write(&[META]).unwrap();
        bytes.truncate(5);
        assert!(read(&bytes).is_err_and(|e| e.contains("end of its index")));
    }

    #[test]
    fn rejects_an_index_longer_than_the_longest_allowed() {
        let mut bytes = (LONGEST_INDEX as u32 + 1).to_le_bytes().to_vec();
        bytes.resize(LONGEST_INDEX + 10, 0);
        assert!(read(&bytes).is_err_and(|e| e.contains("too long")));
    }

    #[test]
    fn rejects_a_length_beyond_any_file() {
        let entries = [Entry {
            kind: "world",
            version: 18,
            length: u64::MAX,
        }];
        let index = postcard::to_stdvec(&entries[..]).unwrap();
        let mut bytes = (index.len() as u32).to_le_bytes().to_vec();
        bytes.extend(index);
        assert!(read(&bytes).is_err_and(|e| e.contains("before the end of its world")));
    }
}
