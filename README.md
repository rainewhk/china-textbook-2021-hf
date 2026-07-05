# china-textbook-2021-hf

> 中国教育部审定中小学教科书 PDF 自动合并与 Hugging Face 镜像

本项目地址：<https://github.com/rainewhk/china-textbook-2021-hf>  
上游数据：<https://github.com/TapXWorld/ChinaTextbook>

## 简介

本项目从 GitHub 上游拉取教育部审定的中小学教科书 PDF，自动合并碎片化文件，并托管到 [Hugging Face Datasets](https://huggingface.co/datasets/RainPPR/china-textbook-2021-hf)。

本项目从 <https://github.com/TapXWorld/ChinaTextbook/pull/204> 构建，即 <https://github.com/pubuyun/ChinaTextbook/tree/65054ef0e8323e772dc99e6eddba9715672d0f91>，因为其更新了一个不好的 PDF，经本人验证该分支无误。

## 技术方案

### Rust 合并器 (`main.rs`)

核心逻辑是一个**零外部依赖**的 Rust 单文件程序：

| 功能 | 说明 |
|------|------|
| 完整文件保留 | 已存在的 `.pdf` 直接复制到 `data/` 对应路径 |
| 直接分片合并 | `xxx.pdf.1`、`xxx.pdf.2` 按序号合并为 `xxx.pdf` |
| `_merge_folder` 提取 | `xxx.pdf_merge_folder/` 内分片提取并合并为 `xxx.pdf` |
| 去重策略 | 完整 PDF > `_merge_folder` 分片 > 直接分片 |
| 流式拷贝 | `std::io::copy` 处理大文件，不占用额外内存 |

### 处理规则

1. **完整文件**：`xxx.pdf`，从 `origin/` 直接复制到 `data/`。
2. **直接分片**：`xxx.pdf.1`, `xxx.pdf.2`, ... → 按末尾数字排序后合并为 `xxx.pdf`。
3. **`_merge_folder` 分片**：`xxx.pdf_merge_folder/xxx.pdf.1`, ... → 合并后输出到上级目录的 `xxx.pdf`。

当同一目的地同时满足多种来源时，优先保留完整文件，其余冲突按 `_merge_folder` > `直接分片` 选择一套，避免重复。

### CI 流水线 (`.github/workflows/ci.yml`)

全部在 GitHub Actions 上自动化完成：

1. **Checkout 本仓库** – 获取 `main.rs` 与 `ci.yml`
2. **Sparse-checkout 上游** – 仅拉取 `小学`、`小学（五•四学制）`、`初中`、`初中（五•四学制）`、`高中` 目录，节省带宽
3. **安装环境** – 安装 `hf` CLI 和 Rust 工具链
4. **编译执行** – `rustc --edition 2021 -O main.rs -o process-pdfs && ./process-pdfs`
5. **目录预览** – `tree data` 查看最终结构
6. **（可选）上传** – 通过 `hf upload` 推送至 Hugging Face Datasets

### 性能与优化

- **零外部依赖**：仅使用 Rust 标准库，编译快、运行快
- **稀疏检出**：`sparse-checkout` 减少 80% 以上的 clone 体积
- **Release 编译**：`-O` 优化开启，单次遍历 + 流式拷贝
- **自动去重**：同一 PDF 仅保留一份完整版本

## 数据结构

```
data/
├── 小学/
├── 小学（五•四学制）/
├── 初中/
├── 初中（五•四学制）/
└── 高中/
```

每个目录下按照**学科/版本/年级**组织教材 PDF 文件。

## 使用

在 GitHub 仓库页面手动触发 `CI/CD Pipeline`，或在 Actions 页面点击 `Run workflow`。

## 版权

- 上游数据版权归各出版社所有，本项目仅作技术整理与自动化处理。
- 项目代码使用 MIT 许可。
