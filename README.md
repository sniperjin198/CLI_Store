# **Excel CLI (excel\_cli)**

一个基于 Rust 开发的高性能、轻量级命令行 Excel 处理工具。旨在通过命令行交互高效解决日常办公与工程数据处理中的表格检查、检索、读写、样式调整及格式转换等需求。

相比传统的 Python 脚本，Rust 编译生成的二进制文件具备**原生单文件无依赖、启动飞快、内存占用低、准确可靠**的显著优势。

## **🚀 核心功能特性 (Commands)**

本工具支持丰富的子命令，涵盖表格处理的高频核心操作：

| 子命令 (Command) | 描述 (Description) |
| :---- | :---- |
| inspect | 检查并提取工作簿元数据（工作表列表、维度等） |
| search | 全文检索指定关键词 |
| read | 范围读取或快速预览表格数据 |
| convert | 将工作表高效率导出为 UTF-8 编码的 CSV 格式 |
| create | 快速创建全新工作簿 |
| cell | 向指定单元格写入数值、文本或公式 |
| append | 向工作表末尾追加整行数据 |
| sheet | 工作表生命周期管理 (add / remove / rename) |
| style | 设置单元格或区域样式（字体、字号、颜色、对齐方式、边框） |
| dimension | 智能调整指定行高与列宽 |
| merge | 合并指定单元格区域 |
| grid | 行列增删管理 (insert-row / delete-row / insert-col / delete-col) |
| help | 打印全局帮助信息或指定子命令的详细帮助说明 |

## **🛠️ 安装与编译 (Build & Install)**

在项目根目录下使用 Cargo 即可一键构建 Release 版本：

cargo build \--release

编译完成后，二进制可执行程序位于 target/release/excel\_cli（Windows 下为 excel\_cli.exe）。将其加入系统环境变量 PATH 中即可在任意路径调用。

## **📖 使用示例 (Usage)**

### **1\. 查看帮助**

\# 全局帮助  
excel\_cli \--help

\# 查看特定子命令用法（例如样式设置）  
excel\_cli style \--help

### **2\. 检查与检索**

\# 检查工作簿基本信息  
excel\_cli inspect data.xlsx

\# 全文检索关键词  
excel\_cli search data.xlsx "绿化工程"

### **3\. 数据读写与追加**

\# 读取 A1:D10 区域内容  
excel\_cli read data.xlsx \--range A1:D10

\# 向指定单元格写入内容  
excel\_cli cell data.xlsx \--sheet Sheet1 \--coord B2 \--value "100.5"

\# 向工作表追加一行  
excel\_cli append data.xlsx \--sheet Sheet1 \--values "苗木名称,数量,单价"

### **4\. 样式与版面调整**

\# 调整列宽  
excel\_cli dimension data.xlsx \--sheet Sheet1 \--col A \--width 25

\# 导出为 CSV  
excel\_cli convert data.xlsx \--sheet Sheet1 \--output result.csv

## **🗺️ 后续路线图 (Roadmap)**

* \[ \] 根据实际工程与办公场景，扩充更多批处理子命令  
* \[ \] 持续集成到个人的 CLI 工具箱合集 (CLI\_Store)  
* \[ \] 丰富样式定制语法与批量公式填充能力

## **📄 开源许可 (License)**

本项目采用 [MIT License](http://docs.google.com/LICENSE) 授权许可。
