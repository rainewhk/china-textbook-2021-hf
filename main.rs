use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let data = Path::new("data");

    if !data.exists() {
        eprintln!("error: 'data' directory does not exist");
        std::process::exit(1);
    }

    let mut targets: BTreeMap<PathBuf, Parts> = BTreeMap::new();
    let mut folders_to_remove: BTreeSet<PathBuf> = BTreeSet::new();
    let mut parts_to_remove: BTreeSet<PathBuf> = BTreeSet::new();

    // 第一轮扫描：构建目标映射
    scan_targets(data, data, &mut targets, &mut folders_to_remove, &mut parts_to_remove)?;

    // 处理合并
    for (target_rel, parts) in targets {
        let target_path = data.join(&target_rel);

        if parts.has_full_file {
            // 已有完整文件，什么都不做，残留在 targets 中的分片百事典在 loop 外清理
            println!("SKIP  {} (already complete)", target_rel.display());
        } else if !parts.folder_parts.is_empty() {
            println!("MERGE folder -> {}", target_rel.display());
            merge_parts(&parts.folder_parts, &target_path)?;
        } else if !parts.direct_parts.is_empty() {
            println!("MERGE direct -> {}", target_rel.display());
            merge_parts(&parts.direct_parts, &target_path)?;
        }
    }

    // 删除所有 _merge_folder 目录
    for folder in folders_to_remove {
        if folder.exists() {
            fs::remove_dir_all(&folder)?;
            println!("REMOVED dir  {}", folder.display());
        }
    }

    // 删除所有直接分包文件
    for part_path in parts_to_remove {
        if part_path.exists() {
            fs::remove_file(&part_path)?;
            println!("REMOVED part {}", part_path.display());
        }
    }

    Ok(())
}

#[derive(Default, Clone)]
struct Parts {
    has_full_file: bool,
    folder_parts: Vec<PathBuf>,
    direct_parts: Vec<PathBuf>,
}

fn sort_parts(parts: &mut [PathBuf]) {
    parts.sort_by(|a, b| {
        fn extract_num(p: &Path) -> u32 {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .rsplitn(2, '.')
                .next()
                .unwrap_or("0")
                .parse::<u32>()
                .unwrap_or(0)
        }
        extract_num(a).cmp(&extract_num(b))
    });
}

fn merge_parts(parts: &[PathBuf], dest: &Path) -> io::Result<()> {
    let mut out = fs::File::create(dest)?;
    let mut sorted_parts = parts.to_vec();
    sort_parts(&mut sorted_parts);
    for part in &sorted_parts {
        let mut file = fs::File::open(part)?;
        io::copy(&mut file, &mut out)?;
    }
    Ok(())
}

fn scan_targets(
    root: &Path,
    dir: &Path,
    targets: &mut BTreeMap<PathBuf, Parts>,
    folders_to_remove: &mut BTreeSet<PathBuf>,
    parts_to_remove: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if name_str.ends_with("_merge_folder") {
                let base_len = name_str.len() - "_merge_folder".len();
                let base_name = &name_str[..base_len];

                let parent_dir = path.parent().unwrap_or(Path::new(""));
                let target_rel = parent_dir.strip_prefix(root).unwrap_or(parent_dir);
                let target_rel = target_rel.join(base_name);

                let mut parts = Vec::new();
                for part_entry in fs::read_dir(&path)? {
                    let part_entry = part_entry?;
                    let part_path = part_entry.path();
                    if !part_path.is_file() {
                        continue;
                    }
                    let part_name = part_entry.file_name();
                    let part_name_str = part_name.to_string_lossy();
                    if let Some(suffix) = part_name_str.strip_prefix(base_name) {
                        if suffix.starts_with('.') && suffix[1..].parse::<u32>().is_ok() {
                            parts.push(part_path);
                        }
                    }
                }

                if !parts.is_empty() {
                    let entry = targets.entry(target_rel).or_default();
                    if entry.folder_parts.is_empty() {
                        entry.folder_parts = parts;
                    }
                }
                folders_to_remove.insert(path.clone());
            } else {
                scan_targets(root, &path, targets, folders_to_remove, parts_to_remove)?;
            }
        } else {
            let file_name = name_str;
            if file_name.ends_with(".pdf") && !file_name.contains(".pdf.") {
                // 这是合并后的目标文件
                let rel = path.strip_prefix(root).unwrap_or(&path);
                let entry = targets.entry(rel.to_path_buf()).or_default();
                entry.has_full_file = true;
            } else if file_name.contains(".pdf.") {
                // 这是直接分包文件
                if let Some(base) = file_name.rsplitn(2, '.').nth(1) {
                    let parent_dir = path.parent().unwrap_or(Path::new(""));
                    let target_rel = parent_dir.strip_prefix(root).unwrap_or(parent_dir);
                    let target_rel = target_rel.join(base);

                    let entry = targets.entry(target_rel).or_default();
                    entry.direct_parts.push(path.clone());
                    parts_to_remove.insert(path.clone());
                }
            }
        }
    }
    Ok(())
}
