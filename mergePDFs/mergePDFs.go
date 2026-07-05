package main

import (
	"io/fs"
	"io/ioutil"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

func main() {
	dirPath := "." // 当前目录
	mergeSplitPDFsRecursively(dirPath)
}

func mergeSplitPDFsRecursively(root string) {
	splitFiles := make(map[string][]string)

	err := filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err // 若有访问错误，返回它
		}
		if d.IsDir() {
			return nil // 忽略目录
		}
		fileName := d.Name()
		if strings.Contains(fileName, ".pdf.") {
			baseName := strings.Split(fileName, ".pdf.")[0] + ".pdf"
			basePath := filepath.Join(filepath.Dir(path), baseName)
			splitFiles[basePath] = append(splitFiles[basePath], path)
		}
		return nil
	})

	if err != nil {
		panic(err)
	}

	for basePath, parts := range splitFiles {
		sort.Strings(parts)
		mergeFiles(basePath, parts)
	}
}

func mergeFiles(basePath string, parts []string) {
	mergedFile, err := os.Create(basePath)
	if err != nil {
		panic(err)
	}
	defer mergedFile.Close()

	for _, part := range parts {
		data, err := ioutil.ReadFile(part)
		if err != nil {
			panic(err)
		}
		_, err = mergedFile.Write(data)
		if err != nil {
			panic(err)
		}
		os.Remove(part) // 合并后删除分割文件
	}
}
