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
    Ok(())
}

fn process_dir(dir: &Path) -> io::Result<()> {
    // 先递归处理子目录，释放深层空间
    let mut subdirs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().ends_with("_merge_folder") {
            subdirs.push(path);
        }
    }
    for subdir in subdirs {
        process_dir(&subdir)?;
    }

    // 扫描当前目录（排除 _merge_folder，稍后统一处理）
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
            if let Some(base) = name_str.rsplitn(2, '.').nth(1) {
                direct_parts.entry(base.to_string()).or_default().push(path.clone());
            }
        }
    }

    // 合并处理当前目录的所有目标
    let mut all_targets = HashSet::new();
    all_targets.extend(full_files.iter().cloned());
    all_targets.extend(direct_parts.keys().cloned());
    all_targets.extend(merge_folders.keys().cloned());

    for target_name in all_targets {
        let has_full = full_files.contains(&target_name);
        let has_folder = merge_folders.contains_key(&target_name);

        if has_full {
            // 有完整文件：删除所有分片和 merge_folder
            if let Some(folder) = merge_folders.get(&target_name) {
                if folder.exists() {
                    fs::remove_dir_all(folder)?;
                    println!("REMOVED dir (has full)   {}", folder.display());
                }
            }
            if let Some(parts) = direct_parts.get(&target_name) {
                for part in parts {
                    if part.exists() {
                        fs::remove_file(part)?;
                        println!("REMOVED part (has full)  {}", part.display());
                    }
                }
            }
        } else if let Some(folder) = merge_folders.get(&target_name) {
            // 用 folder 合并，然后删除
            let parts = collect_parts_from_folder(folder, &target_name)?;
            let dest = dir.join(&target_name);
            if !parts.is_empty() {
                merge_parts(&parts, &dest)?;
                println!("MERGE folder -> {}", dest.display());
                // 逐个删除已合并的分片（实时释放）
                for part in &parts {
                    if part.exists() {
                        fs::remove_file(part)?;
                    }
                }
            }
            if folder.exists() {
                fs::remove_dir_all(folder)?;
                println!("REMOVED dir              {}", folder.display());
            }
            // 同时清理直接分片（如果有，但通常不会有）
            if let Some(parts) = direct_parts.get(&target_name) {
                for part in parts {
                    if part.exists() {
                        fs::remove_file(part)?;
                        println!("REMOVED part (dup)       {}", part.display());
                    }
                }
            }
        } else if let Some(parts) = direct_parts.get(&target_name) {
            // 用直接分片合并
            let dest = dir.join(&target_name);
            merge_parts(parts, &dest)?;
            println!("MERGE direct -> {}", dest.display());
            for part in parts {
                if part.exists() {
                    fs::remove_file(part)?;
                    println!("REMOVED part             {}", part.display());
                }
            }
        }
    }

    Ok(())
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
    // 按末尾数字排序：.1 < .2 < .10
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
    Ok(parts)
}

fn merge_parts(parts: &[PathBuf], dest: &Path) -> io::Result<()> {
    let mut out = fs::File::create(dest)?;
    for part in parts {
        let mut file = fs::File::open(part)?;
        io::copy(&mut file, &mut out)?;
    }
    Ok(())
}
