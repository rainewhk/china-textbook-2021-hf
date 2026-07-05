---
language:
  - zh
license: unknown
size_categories:
  - 10GB<n<100GB
---

# China Textbook 2021

中国教育部审定的中小学教科书 PDF 合集。

## 简介

本数据集包含从小学到高中各年级的教育部审定教科书 PDF。原始数据来自 [TapXWorld/ChinaTextbook](https://github.com/TapXWorld/ChinaTextbook)，由本项目自动合并碎片化文件后托管在 Hugging Face Datasets 上。

**处理代码**：<https://github.com/rainewhk/china-textbook-2021-hf>

## 数据结构

教材按学段、学科、版本、年级分层存放：

```
小学/
├── ...
初中/
├── ...
初中（五•四学制）/
├── ...
小学（五•四学制）/
├── ...
高中/
└── ...
```

文件名为 `义务教育教科书·<学科> <年级> <册>册.pdf` 或类似格式。

## 处理说明

- 上游 PDF 存在碎片化存储（`.1`, `.2` 等分片），本项目通过 [Rust 合并器](https://github.com/rainewhk/china-textbook-2021-hf/blob/main/main.rs) 自动合并为完整文件。
- 部分教材同时存在完整 PDF 和 `_merge_folder` 分片，合并器已按优先级去重，确保每个文件仅保留一份完整版本。

## 使用方式

你可以直接下载或克隆本数据集：

```shell
# 安装 huggingface_hub
pip install "huggingface_hub[cli]"

# 下载到本地
hf hub download RainPPR/china-textbook-2021-hf --local-dir ./data
```

```python
from datasets import load_dataset

# 加载数据集
# 注意：数据集以原始 PDF 文件形式存在
ds = load_dataset("RainPPR/china-textbook-2021-hf", streaming=True)
```

## 许可

- 原始数据版权归各出版机构所有。
- 本数据集仅供个人学习、研究使用，请勿用于商业用途。
- 项目代码使用 MIT 许可。
