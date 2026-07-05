#!/bin/bash

DIR_PATH="$(cd "$(dirname "$0")" && pwd)" # 当前目录

merge_split_pdfs_in_directory() {
  cd "$DIR_PATH" || exit 1
  ls *.pdf.* 2>/dev/null | sed 's/\.[0-9]*$//' | sort -u | while IFS= read -r base_name; do
    IFS=$'\n' parts=( $(ls "${base_name}".* | sort -V) ) # 确保文件顺序正确
    merge_files "$base_name" "${parts[@]}"
  done
}

merge_files() {
  local output_file=$1
  shift
  local parts=("$@")
  cat "${parts[@]}" > "$output_file"
  rm -f "${parts[@]}" # 合并后删除分割文件
}

merge_split_pdfs_in_directory