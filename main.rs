use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let data = Path::new("data");
    if !data.exists() {
        eprintln!("error: 'data' directory does not exist");
        std::process::exit(1);
    }
    process_dir(data)?;

    if let Ok(content) = fs::read_to_string("DATASET_CARD.md") {
        fs::write(data.join("README.md"), content)?;
    }

    Ok(())
}

fn process_dir(dir: &Path) -> io::Result<()> {
    // 先递归处理子目录，释放深层空间
    let mut subdirs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !is_merge_folder(&path) {
            subdirs.push(path);
        }
    }
    for subdir in subdirs {
        process_dir(&subdir)?;
    }

    // 扫描当前目录
    let mut full_files: HashSet<String> = HashSet::new();
    let mut direct_parts: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut merge_folders: HashMap<String, PathBuf> = HashMap::new();

    let entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    for entry in &entries {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if let Some(base) = name_str.strip_suffix("_merge_folder") {
                merge_folders.insert(base.to_string(), path.clone());
            }
        } else if name_str.ends_with(".pdf") && !name_str.contains(".pdf.") {
            full_files.insert(name_str.to_string());
        } else if name_str.contains(".pdf.") {
            // 与 Go 代码一致：strings.Split(fileName, ".pdf.")[0] + ".pdf"
            let base = name_str.split(".pdf.").next().unwrap_or(&name_str);
            let base_name = format!("{}.pdf", base);
            direct_parts.entry(base_name).or_default().push(path.clone());
        }
    }

    // 排序：按末尾数字排序（.1 < .2 < .10 < .11）
    for parts in direct_parts.values_mut() {
        parts.sort_by(|a, b| part_num(a).cmp(&part_num(b)));
    }

    // 合并处理当前目录的所有目标
    let mut all_targets = HashSet::new();
    all_targets.extend(full_files.iter().cloned());
    all_targets.extend(direct_parts.keys().cloned());
    all_targets.extend(merge_folders.keys().cloned());

    for target_name in all_targets {
        let has_full = full_files.contains(&target_name);

        if has_full {
            // 有完整文件：删除所有分片和 merge_folder
            if let Some(folder) = merge_folders.get(&target_name) {
                if folder.exists() {
                    let _ = fs::remove_dir_all(folder);
                }
            }
            if let Some(parts) = direct_parts.get(&target_name) {
                for part in parts {
                    if part.exists() {
                        let _ = fs::remove_file(part);
                    }
                }
            }
        } else if let Some(folder) = merge_folders.get(&target_name) {
            // 用 folder 合并，然后删除
            let parts = collect_parts_from_folder(folder, &target_name)?;
            let dest = dir.join(&target_name);

            if !parts.is_empty() {
                print_merge_log("MERGE folder", &dest, &parts);
                merge_parts(&parts, &dest)?;
                for part in &parts {
                    if part.exists() {
                        let _ = fs::remove_file(part);
                    }
                }
            }
            if folder.exists() {
                let _ = fs::remove_dir_all(folder);
            }
            // 同时清理直接分片（如果有）
            if let Some(dparts) = direct_parts.get(&target_name) {
                for part in dparts {
                    if part.exists() {
                        let _ = fs::remove_file(part);
                    }
                }
            }
        } else if let Some(parts) = direct_parts.get(&target_name) {
            // 用直接分片合并
            let dest = dir.join(&target_name);

            print_merge_log("MERGE direct", &dest, parts);
            merge_parts(parts, &dest)?;
            for part in parts {
                if part.exists() {
                    let _ = fs::remove_file(part);
                }
            }
        }
    }

    // 删除所有非 .pdf 文件
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // 保留以 .pdf 结尾的完整文件，删除其他所有文件（包括残留的分片、非 PDF 文件等）
            if !name_str.ends_with(".pdf") || name_str.contains(".pdf.") {
                let _ = fs::remove_file(&path);
            }
        }
    }

    Ok(())
}

fn is_merge_folder(path: &Path) -> bool {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .ends_with("_merge_folder")
}

fn print_merge_log(label: &str, dest: &Path, parts: &[PathBuf]) {
    let names: Vec<String> = parts
        .iter()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .collect();
    println!("{} -> {} [{}]", label, dest.display(), names.join(" -> "));
}

fn collect_parts_from_folder(folder: &Path, base_name: &str) -> io::Result<Vec<PathBuf>> {
    let mut parts = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if let Some(suffix) = name_str.strip_prefix(base_name) {
            if suffix.starts_with('.') && suffix[1..].parse::<u32>().is_ok() {
                parts.push(path);
            }
        }
    }
    // 按末尾数字排序：.1 < .2 < .10 < .11
    parts.sort_by(|a, b| part_num(a).cmp(&part_num(b)));
    Ok(parts)
}

fn part_num(p: &Path) -> u32 {
    p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .rsplitn(2, '.')
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0)
}

fn merge_parts(parts: &[PathBuf], dest: &Path) -> io::Result<()> {
    let mut out = fs::File::create(dest)?;
    for part in parts {
        let mut file = fs::File::open(part)?;
        io::copy(&mut file, &mut out)?;
    }
    Ok(())
}
