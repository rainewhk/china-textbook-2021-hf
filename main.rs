use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct PdfSource {
    full_file: Option<PathBuf>,
    direct_parts: Vec<PathBuf>,
    folder_parts: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let origin = Path::new("origin");
    let data = Path::new("data");

    if data.exists() {
        fs::remove_dir_all(data)?;
    }

    let mut targets: BTreeMap<PathBuf, PdfSource> = BTreeMap::new();
    collect_files(origin, origin, &mut targets)?;

    for (rel_path, source) in targets {
        let dest = data.join(&rel_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Some(full) = source.full_file {
            fs::copy(&full, &dest)?;
            eprintln!("COPY  {}", rel_path.display());
        } else if !source.folder_parts.is_empty() {
            let mut parts = source.folder_parts;
            sort_parts(&mut parts);
            merge_parts(&parts, &dest)?;
            eprintln!(
                "MERGE folder {} ({} parts)",
                rel_path.display(),
                parts.len()
            );
        } else if !source.direct_parts.is_empty() {
            let mut parts = source.direct_parts;
            sort_parts(&mut parts);
            merge_parts(&parts, &dest)?;
            eprintln!(
                "MERGE direct {} ({} parts)",
                rel_path.display(),
                parts.len()
            );
        }
    }

    Ok(())
}

fn sort_parts(parts: &mut [PathBuf]) {
    parts.sort_by(|a, b| {
        let a_name = a.file_name().unwrap_or_default().to_string_lossy();
        let b_name = b.file_name().unwrap_or_default().to_string_lossy();
        let a_num = a_name
            .rsplitn(2, '.')
            .next()
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);
        let b_num = b_name
            .rsplitn(2, '.')
            .next()
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);
        a_num.cmp(&b_num)
    });
}

fn merge_parts(parts: &[PathBuf], dest: &Path) -> io::Result<()> {
    let mut out = fs::File::create(dest)?;
    for part in parts {
        let mut file = fs::File::open(part)?;
        io::copy(&mut file, &mut out)?;
    }
    Ok(())
}

fn collect_files(
    dir: &Path,
    origin_root: &Path,
    targets: &mut BTreeMap<PathBuf, PdfSource>,
) -> io::Result<()> {
    let rd = fs::read_dir(dir)?;
    for entry in rd {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if name_str.ends_with("_merge_folder") {
                let base_len = name_str.len() - "_merge_folder".len();
                let base_name = &name_str[..base_len];
                let parent_dir = path.parent().unwrap_or(Path::new(""));
                let rel_parent = parent_dir.strip_prefix(origin_root).unwrap_or(parent_dir);
                let target_rel = rel_parent.join(base_name);

                let mut parts = vec![];
                for part_entry in fs::read_dir(&path)? {
                    let part_entry = part_entry?;
                    let part_path = part_entry.path();
                    if !part_path.is_file() {
                        continue;
                    }
                    let part_name = part_entry.file_name();
                    let part_name_str = part_name.to_string_lossy();
                    if let Some(suffix) = part_name_str.strip_prefix(base_name) {
                        if suffix.len() > 1 && suffix.starts_with('.') {
                            if suffix[1..].parse::<u32>().is_ok() {
                                parts.push(part_path);
                            }
                        }
                    }
                }
                let source = targets.entry(target_rel).or_default();
                source.folder_parts = parts;
            } else {
                collect_files(&path, origin_root, targets)?;
            }
        } else {
            let file_name = name_str;

            if file_name.ends_with(".pdf") && !file_name.contains(".pdf.") {
                let rel_path = path.strip_prefix(origin_root).unwrap_or(&path).to_path_buf();
                let source = targets.entry(rel_path).or_default();
                source.full_file = Some(path.clone());
            } else if file_name.contains(".pdf.") {
                if let Some(base) = file_name.rsplitn(2, '.').nth(1) {
                    let parent_dir = path.parent().unwrap_or(Path::new(""));
                    let rel_parent = parent_dir.strip_prefix(origin_root).unwrap_or(parent_dir);
                    let target_rel = rel_parent.join(base);
                    let source = targets.entry(target_rel).or_default();
                    source.direct_parts.push(path.clone());
                }
            }
        }
    }
    Ok(())
}
