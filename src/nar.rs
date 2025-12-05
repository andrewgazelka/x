//! NAR (Nix ARchive) format unpacking
//!
//! NAR format specification:
//! ```text
//! NAR = "nix-archive-1" contents
//! contents = "(" type (contents-specific) ")"
//!
//! For regular file:
//!   "(" "type" "regular" ("executable" "")? "contents" <file-contents> ")"
//!
//! For directory:
//!   "(" "type" "directory" (entry)* ")"
//!   entry = "entry" "(" "name" <name> "node" contents ")"
//!
//! For symlink:
//!   "(" "type" "symlink" "target" <target> ")"
//!
//! Strings are: 8-byte little-endian length + contents + padding to 8-byte boundary
//! ```

use std::io::{BufReader, Read};

const NAR_MAGIC: &str = "nix-archive-1";

/// Unpack a NAR archive to the given destination path
pub fn unpack<R: Read>(reader: R, dest: &std::path::Path) -> eyre::Result<()> {
    let mut reader = BufReader::new(reader);

    let magic = read_string(&mut reader)?;
    if magic != NAR_MAGIC {
        eyre::bail!("invalid NAR magic: expected '{NAR_MAGIC}', got '{magic}'");
    }

    unpack_node(&mut reader, dest)
}

fn read_u64<R: Read>(reader: &mut R) -> eyre::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_string<R: Read>(reader: &mut R) -> eyre::Result<String> {
    let len = read_u64(reader)? as usize;

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    // Skip padding to 8-byte boundary
    let padding = (8 - (len % 8)) % 8;
    if padding > 0 {
        let mut pad = vec![0u8; padding];
        reader.read_exact(&mut pad)?;
    }

    String::from_utf8(buf).map_err(|e| eyre::eyre!("invalid UTF-8 in NAR string: {e}"))
}

fn read_bytes<R: Read>(reader: &mut R) -> eyre::Result<Vec<u8>> {
    let len = read_u64(reader)? as usize;

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    // Skip padding to 8-byte boundary
    let padding = (8 - (len % 8)) % 8;
    if padding > 0 {
        let mut pad = vec![0u8; padding];
        reader.read_exact(&mut pad)?;
    }

    Ok(buf)
}

fn expect_string<R: Read>(reader: &mut R, expected: &str) -> eyre::Result<()> {
    let actual = read_string(reader)?;
    if actual != expected {
        eyre::bail!("NAR parse error: expected '{expected}', got '{actual}'");
    }
    Ok(())
}

fn unpack_node<R: Read>(reader: &mut R, path: &std::path::Path) -> eyre::Result<()> {
    expect_string(reader, "(")?;
    expect_string(reader, "type")?;

    let node_type = read_string(reader)?;

    match node_type.as_str() {
        "regular" => unpack_regular(reader, path)?,
        "directory" => unpack_directory(reader, path)?,
        "symlink" => unpack_symlink(reader, path)?,
        _ => eyre::bail!("unknown NAR node type: '{node_type}'"),
    }

    expect_string(reader, ")")?;
    Ok(())
}

fn unpack_regular<R: Read>(reader: &mut R, path: &std::path::Path) -> eyre::Result<()> {
    let mut executable = false;
    let mut contents: Option<Vec<u8>> = None;

    loop {
        let tag = read_string(reader)?;
        match tag.as_str() {
            "executable" => {
                // Read empty string marker
                expect_string(reader, "")?;
                executable = true;
            }
            "contents" => {
                contents = Some(read_bytes(reader)?);
            }
            ")" => {
                // Put the closing paren back conceptually - we need to handle this
                // by writing the file and returning early
                break;
            }
            _ => eyre::bail!("unexpected tag in regular file: '{tag}'"),
        }
    }

    // Write file contents
    if let Some(data) = contents {
        std::fs::write(path, data)?;
    } else {
        // Empty file
        std::fs::write(path, [])?;
    }

    // Set executable permission on Unix
    if executable {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms)?;
        }
    }

    Ok(())
}

fn unpack_directory<R: Read>(reader: &mut R, path: &std::path::Path) -> eyre::Result<()> {
    std::fs::create_dir_all(path)?;

    loop {
        let tag = read_string(reader)?;
        match tag.as_str() {
            "entry" => {
                expect_string(reader, "(")?;
                expect_string(reader, "name")?;
                let name = read_string(reader)?;
                expect_string(reader, "node")?;

                let child_path = path.join(&name);
                unpack_node(reader, &child_path)?;

                expect_string(reader, ")")?;
            }
            ")" => {
                // End of directory - put paren back
                break;
            }
            _ => eyre::bail!("unexpected tag in directory: '{tag}'"),
        }
    }

    Ok(())
}

fn unpack_symlink<R: Read>(reader: &mut R, path: &std::path::Path) -> eyre::Result<()> {
    loop {
        let tag = read_string(reader)?;
        match tag.as_str() {
            "target" => {
                let target = read_string(reader)?;

                #[cfg(unix)]
                std::os::unix::fs::symlink(&target, path)?;

                #[cfg(not(unix))]
                {
                    // On Windows, try to create a symlink (may need admin rights)
                    // Fall back to copying if that fails
                    let _ = target; // suppress unused warning
                    eyre::bail!("symlinks not fully supported on this platform");
                }
            }
            ")" => break,
            _ => eyre::bail!("unexpected tag in symlink: '{tag}'"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nar_string(s: &str) -> Vec<u8> {
        let len = s.len() as u64;
        let mut result = len.to_le_bytes().to_vec();
        result.extend_from_slice(s.as_bytes());
        // Add padding
        let padding = (8 - (s.len() % 8)) % 8;
        result.extend(vec![0u8; padding]);
        result
    }

    #[test]
    fn test_read_string() {
        let data = make_nar_string("hello");
        let mut cursor = std::io::Cursor::new(data);
        let result = read_string(&mut cursor).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_read_string_with_padding() {
        // "hi" is 2 bytes, needs 6 bytes padding
        let data = make_nar_string("hi");
        let mut cursor = std::io::Cursor::new(data);
        let result = read_string(&mut cursor).unwrap();
        assert_eq!(result, "hi");
    }
}
