# 为什么会有这个项目？
此前，我们上传的 PDF 合并程序在 VirusTotal 上被检测出可能含有病毒。然而，正如许多安全领域的朋友所知，这种检测结果往往是误报（false positive）。为了彻底打消用户的最后一丝疑虑，并确保程序的安全性与透明度，我们创建了此项目并进行了改进。

## 最新更新：
* 如果您使用苹果电脑，可执行 Mac 目录的 mergePDFs.command 程序。
* 使用 GitHub Actions 编译：我们现已采用 GitHub Actions 来自动化编译流程。程序的构建过程完全在开源环境中进行，确保安全性和一致性，用户可以随时查看完整的编译步骤。
* 支持多平台：程序现已支持 macOS、Windows 和 Linux 三个主流操作系统版本，不仅提升了可访问性，也保证了跨平台的安全性与可靠性。
* 公开透明：通过 GitHub Actions，所有编译流程都公开可见。用户可以直接访问 GitHub 仓库，审查工作流和代码，验证程序的安全性。

### macOS

> 1. 将合并脚本 `mergePDFs.command` 下载到包含 PDF 文件的文件夹中。
> 2. 确保 `mergePDFs.command` 和被拆分的 PDF 文件在同一目录下。
> 3. 双击 `mergePDFs.command` 脚本即可自动完成文件合并。
> 4. 如果遇到权限问题，搜索并打开 Terminal.app（终端.app）
> 5. 输入 `sudo chmod 777 ` 注意最后的空格

### 其它合并技巧

> 1. Windows 系统打开 CMD 或者 Windows Terminal 使用命令：copy /b "文件名1" + "文件名2" "合并后的文件名" 
> 2. MacOS 或 Linux 系统打开终端使用命令：cat "文件名1" "文件名2" > "合并后的文件名"


## 原因
* https://github.com/TapXWorld/ChinaTextbook/issues/35

## 感谢
[ @lisonge](https://github.com/lisonge/)
