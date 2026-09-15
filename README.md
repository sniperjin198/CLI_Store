# **🚀 GooseCliBuilder / Computer Controller Suite**

> **打造一套完全符合个人工作习惯与大模型 Agent 深度协同的原生高性能自动化工具箱（Computer Controller 扩展套件）。**

> 第一阶段目标：利用 AI 辅助编码（AICode），构建自主可控、极速响应、无环境依赖的本地系统级控制与办公自动化底座。欢迎关注、Star 与多多鼓励！✨

## **💡 项目理念与愿景 (Vision)**

在日常办公自动化与大语言模型（LLM / Agent 如 Goose, Qwen）落地的场景中，传统 Python / Node.js 脚本常常面临**启动冷延迟高、运行环境臃肿、第三方依赖冲突、Windows COM 独占崩溃**等痛点。

本项目基于 **Rust 系统级语言** 全程构建，坚持以下核心哲学：

1. **原生无依赖 (Zero Dependency)**：编译即交付单文件二进制（.exe），无须安装 Office、Python 运行时或庞大解释器。  
2. **结构化契约 (JSON-First & No Hex Errors)**：所有命令标准输出统一返回规范单行/多行 JSON，错误信息大白话、语义化枚举，杜绝难懂的十六进制底层错误码，让大模型零成本直读。  
3. **三步人机认知闭环 (Three-Step Cognitive Flow)**：针对超长文档提供“宏观驾驶舱建构 \-\> 章节下钻 \-\> 微观精准落盘修改”流程，杜绝长文本爆 Token，保证精确性。

## **📦 工具箱矩阵 (Toolbox Matrix)**

### **一、 已就绪交付模块 (Production Ready)**

| 工具名称 | 对应格式/能力 | 核心功能概述 |
| :---- | :---- | :---- |
| **word\_cli** | .docx 文档处理 | 大纲树驾驶舱自省、Markdown 极速转交付级 Word、跨 Run 占位符批量注入、表格矩阵读写与行列动态伸缩、DrawingML 嵌入图片排版、野生文档格式刷平、分节页面横纵混排与动态页码重编。 |
| **excel\_cli** | .xlsx 表格处理 | 工作簿元数据扫描、全文快速检索、矩阵范围读写与预览、单单元格公式写入、行列动态增删、单元格样式美化（字号/背景色/边框）及 UTF-8 BOM CSV 高速导出。 |
| **pdf\_cli** | .pdf 几何手术刀 | 宏观大纲自省、ToUnicode 中文字体容错回退、多栏阅读顺序重排、页眉页脚几何降噪、无框财报表格列投影与科目缩进审计、RAG 向量切片、位图解包与 300DPI 渲染、页面拼接抽取、90 度旋转校正及半透明水印注入。 |

### **二、 正在规划与实施模块 (In Active Development)**

| 规划工具 / 模块 | 核心能力规划定位 |
| :---- | :---- |
| **ppt\_cli** | 演示文稿幻灯片自动化创建、母版占位符注入、形状/文本框排版与图形图表批量生成。 |
| **automation\_script** | 跨平台自动化脚本调度引擎，安全创建、隔离运行与流式捕获 PowerShell、Bash 等自动化脚本。 |
| **cache** | 专用缓存管理引擎，统一管理、检索、查看大模型与自动化工作流中的临时文件与持久化数据。 |
| **computer\_control** | 系统级计算机控制与自动化操作系统（GUI 事件交互、窗口管理、进程调度、环境控制），实现真实桌面端 Agent 闭环。 |
| **web\_scrape** | 高性能网页抓取与预处理工具，将目标网页内容快速提炼并保存为纯文本、结构化 JSON 或二进制资源。 |

## **🛠️ 安装与构建 (Build & Install)**

本项目基于现代稳定版 Rust 工具链构建：

\# 1\. 编译全部已就绪组件 (Release 优化构建)  
cargo build \--release

\# 2\. 部署到系统全局 CLI 目录 (推荐 E:\\.Cli\\)  
cmd.exe /c "if not exist E:\\.Cli mkdir E:\\.Cli && copy /y target\\release\\\*.exe E:\\.Cli\\"

建议将 E:\\.Cli\\ 添加至系统环境变量 PATH 中，即可在任意命令行与 Agent 提示词环境中随处调用。

## **📖 核心组件速查指南 (Cheat Sheet)**

### **1\. Word CLI (word\_cli) 常用操作**

\# 【新建与转换】派生新建 / Markdown 一键转合规 Word (自动继承母版样式)  
word\_cli create \--file "output.docx" \[--template "tpl.docx"\]  
word\_cli import \--input "report.md" \--output "report.docx" \[--template "tpl.docx"\]

\# 【宏观认知驾驶舱】秒级摸清文档骨架，不输出漫天长文本  
word\_cli inspect \--file "report.docx"  
word\_cli inspect \--file "report.docx" \--only outline  
word\_cli inspect \--file "report.docx" \--only tables

\# 【定向下钻精读】按大纲章节读取 / 指定段落范围精读  
word\_cli read \--file "report.docx" \--heading "第一章 项目概述"  
word\_cli read \--file "report.docx" \--range "5:10"

\# 【正文安全修改】跨 Run 占位符批量注入 / 关键词高亮  
word\_cli replace \--file "report.docx" \--pairs "{{NAME}}=张三,{{DATE}}=2026-09-01"  
word\_cli highlight \--file "report.docx" \--text "重要审查项" \--color yellow

\# 【排版精细调整】字体、字号、首行缩进与格式刷平  
word\_cli para-format \--file "report.docx" \--index 2 \--font "黑体" \--size 18.0 \--bold true \--indent 2.0 \--spacing 1.5

\# 【表格矩阵读写】Markdown 表格直读 / 单元格改写 / 动态加行  
word\_cli read-table \--file "report.docx" \--index 0 \--format markdown  
word\_cli set-cell \--file "report.docx" \--index 0 \--row 1 \--col 2 \--val "100.0"  
word\_cli insert-row \--file "report.docx" \--index 0 \--row 2 \--cells "序号,指标名称,实测值"

\# 【版式与公文规范】横纵向混排 / 动态页码从第 1 页起算  
word\_cli page \--file "report.docx" \--orient landscape \--size A4  
word\_cli set-footer \--file "report.docx" \--text "内部机密 · 第" \--page-num \--restart 1

### **2\. Excel CLI (excel\_cli) 常用操作**

\# 【工作簿自省与检索】概览信息扫描 / 全文内容检索  
excel\_cli inspect \--file "data.xlsx"  
excel\_cli search \--file "data.xlsx" \--keyword "预算汇总"

\# 【矩阵范围读取】切片读取指定坐标区域  
excel\_cli read \--file "data.xlsx" \--range "A1:D10"

\# 【数据写入与计算】单元格公式注入 / 末行动态追加  
excel\_cli cell \--file "data.xlsx" \--coord "C1" \--value "=SUM(A1:B1)"  
excel\_cli append \--file "data.xlsx" \--values "工程设备" "12" "5600.0"

\# 【格式导出】一键导出带 BOM 无乱码 UTF-8 CSV  
excel\_cli convert \--input "data.xlsx" \--output "export.csv" \--sheet "Sheet1"

\# 【网格与样式排版】行列增删 / 表头颜色样式设置  
excel\_cli grid \--file "data.xlsx" \--action insert-row \--target 2 \--count 1  
excel\_cli style \--file "data.xlsx" \--coord "A1:E1" \--bold true \--font-color "FFFFFF" \--bg-color "4F81BD" \--border true

### **3\. PDF CLI (pdf\_cli) 常用操作**

\# 【宏观认知驾驶舱】秒级摸清文档大纲书签树、总页数与纸张规格  
pdf\_cli inspect \--file "doc.pdf"  
pdf\_cli inspect \--file "doc.pdf" \--only outline  
pdf\_cli inspect \--file "doc.pdf" \--only layout

\# 【定向下钻精读】按大纲章节读取 / 指定页提取 / 降噪过滤与 Markdown 导出  
pdf\_cli extract-text \--file "doc.pdf" \--heading "第三章 技术方案" \--clean-noise  
pdf\_cli extract-text \--file "doc.pdf" \--page 1 \--clean-noise  
pdf\_cli extract-text \--file "doc.pdf" \--range "1:10" \--clean-noise \--format md

\# 【物理坐标审计】提取带物理外接矩形 \[x, y, width, height\] 的精准文字流  
pdf\_cli extract-text \--file "doc.pdf" \--page 1 \--with-bbox

\# 【财报几何自省】无框表格 X 轴列投影聚类 / 会计科目缩进树推算 / 覆盖篡改异常检测  
pdf\_cli inspect-layout \--file "report.pdf" \--page 68

\# 【RAG 知识库语义切片】大纲章节驱动分块 / 滑动窗口分块（带面包屑路径与 Token 估算）  
pdf\_cli chunk \--file "doc.pdf" \--strategy heading \--clean-noise  
pdf\_cli chunk \--file "doc.pdf" \--strategy sliding \--max-chars 800 \--overlap 100 \--clean-noise

\# 【视觉切片与素材提取】内嵌位图无损解包 / 页面 300DPI 高清渲染（支撑后续 OCR）  
pdf\_cli extract-images \--file "doc.pdf" \--out-dir "./images/"  
pdf\_cli render-page \--file "doc.pdf" \--page 1 \--dpi 300 \--out "page\_1.png"

\# 【物理装配与图纸校正】多文档顺畅合并 / 页面区间截取 / 90度旋转 / 半透明水印  
pdf\_cli merge \--files "part1.pdf,part2.pdf" \--output "merged.pdf"  
pdf\_cli extract-pages \--file "doc.pdf" \--pages "1:5" \--output "cut.pdf"  
pdf\_cli rotate \--file "doc.pdf" \--pages "1:3" \--angle 90 \--output "rotated.pdf"  
pdf\_cli watermark \--file "doc.pdf" \--text "内部评审 · 请勿外传" \--opacity 0.2 \--output "marked.pdf"

## **🗺️ 阶段路线图与演进规划 (Roadmap)**

* \[x\] **第一阶段·办公核心数据底座**  
  * \[x\] 完成 excel\_cli 全套只读检索、数据读写、公式与行列排版引擎。  
  * \[x\] 完成 word\_cli OpenXML 解包打包、驾驶舱认知、Markdown 导入、文字精读、表格增删、DrawingML 图片管理及公文页眉页脚排版。  
  * \[x\] 完成 pdf\_cli 宏观大纲自省、高保真几何降噪抽取、无框表格列投影与科目缩进审计、RAG 向量切片、300DPI 渲染、文档装配与半透明水印注入。  
  * \[x\] 全面统一 JSON-First 契约输出与抗脆弱性错误捕获。  
* \[ \] **第一阶段·办公文档能力补全**  
  * \[ \] 构建 ppt\_cli：实现幻灯片批量创建、文本图形替换与模版样式渲染。  
* \[ \] **第二阶段·系统级扩展与自动化闭环 (Computer Controller)**  
  * \[ \] 构建 automation\_script：PowerShell / Bash 自动化脚本隔离沙盒执行与结果采集。  
  * \[ \] 构建 cache：多任务流水线与 Agent 上下文持久化缓存中枢。  
  * \[ \] 构建 computer\_control：鼠标键盘原生模拟、窗口状态捕获与 Windows 原生桌面级自动化支持。  
  * \[ \] 构建 web\_scrape：轻量极速的网页静态与动态数据清洗提取管线。

## **💬 致谢与鼓励**

一套真正顺手、高反应速度、专为大模型量身定制的 CLI 工具集能够大幅提升人机协同与日常开发的幸福感。

如果这个项目对你的日常工作或 AI 自动化探索有所启发，**欢迎点亮 Star ⭐，更欢迎在 Issues / Discussions 中提出宝贵的使用意见与功能需求**！你的支持是持续完善这套工具箱的最大动力！

## **📄 授权协议 (License)**

本项目采用 [MIT License](http://docs.google.com/LICENSE) 授权许可，开源透明，自由分发与商业友好。

# **🌐 English Documentation**

# **🚀 GooseCliBuilder / Computer Controller Suite**

> **A native, high-performance automation toolbox designed for personalized workflows and deep collaboration with LLM Agents (Computer Controller Extension Suite).**

> Phase 1 Objective: Harness AI-assisted coding (AICode) to establish a sovereign, zero-dependency, ultra-fast system control and document automation bedrock. Stars ⭐ and feedback are warmly appreciated\!

## **💡 Vision & Core Philosophy**

In traditional office automation and Agentic tool workflows (e.g., Goose, Qwen), traditional Python or Node.js scripts frequently encounter **cold-start latency, bloated runtime environments, brittle dependency conflicts, and Windows COM exclusivity crashes**.

Engineered from the ground up in **pure Rust**, this suite adheres to three fundamental design principles:

1. **Zero Runtime Dependency**: Compiles into standalone, single-file binaries (.exe). Runs instantly without Microsoft Office installations, Python interpreters, or auxiliary runtimes.  
2. **JSON-First Contract & Semantic Errors**: Every command prints deterministic, parse-safe JSON to stdout. Underlying failures are translated into explicit, semantic string enumerations, eradicating obscure hexadecimal error codes so LLMs can reason over errors immediately.  
3. **Three-Step Cognitive Flow**: For massive documents, the tool enforces an **Overview Cockpit \-\> Section Drilling \-\> In-Place Atomic Mutation** interaction loop, mitigating context-window exhaustion and token waste.

## **📦 Toolbox Matrix**

### **1\. Production Ready Modules**

| Tool Name | Scope / Target | Core Capabilities |
| :---- | :---- | :---- |
| **word\_cli** | .docx Documents | Document outline cockpit, zero-dependency Markdown-to-Word conversion, multi-run safe placeholder injection, 2D table manipulation with dynamic row/column stretching, DrawingML inline image embedding, wildcard formatting strip, multi-section page orientation, and dynamic restartable page numbering. |
| **excel\_cli** | .xlsx Spreadsheets | Workbook metadata inspection, full-text cell keyword search, range slicing, single-cell formula injection, dynamic row/column mutation, cell styling (color, font, borders), and BOM-prefixed UTF-8 CSV exports. |
| **pdf\_cli** | .pdf Scalpel | Outline tree inspection, CMap/ToUnicode Chinese font fallback, geometric header/footer noise removal, multi-column reading order reconstruction, borderless table column clustering and accounting indentation analysis, RAG semantic chunking, bitmap unpack, 300DPI rendering, page merge/split, rotation, and watermark stamping. |

### **2\. Modules In Active Development**

| Planned Module | Scope & Objectives |
| :---- | :---- |
| **ppt\_cli** | Programmatic presentation authoring, master placeholder injection, shape/textbox layout, and automated charting. |
| **automation\_script** | Cross-platform automation script runner for creating, sandboxing, and capturing PowerShell/Bash workflows. |
| **cache** | Specialized persistent cache engine to store, retrieve, and inspect intermediate artifacts across Agent executions. |
| **computer\_control** | Native OS computer controller (GUI mouse/keyboard simulation, window state management, process orchestration). |
| **web\_scrape** | High-throughput web scraping pipeline to extract clean text, structured JSON, or binary assets. |

## **🛠️ Build & Installation**

Built with the stable Rust toolchain:

\# 1\. Build all production-ready modules with release optimizations  
cargo build \--release

\# 2\. Deploy to global CLI directory (e.g., E:\\.Cli\\)  
cmd.exe /c "if not exist E:\\.Cli mkdir E:\\.Cli && copy /y target\\release\\\*.exe E:\\.Cli\\"

Adding E:\\.Cli\\ to your system PATH allows both human users and AI Agents to invoke the tools from any directory or prompt context.

## **📖 Component Cheat Sheet**

### **1\. Word CLI (word\_cli)**

\# \[Creation & Conversion\] New blank document / Markdown-to-Word conversion  
word\_cli create \--file "output.docx" \[--template "tpl.docx"\]  
word\_cli import \--input "report.md" \--output "report.docx" \[--template "tpl.docx"\]

\# \[Cognitive Cockpit\] Inspect document skeleton without dumping raw text  
word\_cli inspect \--file "report.docx"  
word\_cli inspect \--file "report.docx" \--only outline  
word\_cli inspect \--file "report.docx" \--only tables

\# \[Targeted Drilling\] Read section by heading / slice paragraphs by range  
word\_cli read \--file "report.docx" \--heading "Chapter 1 Project Overview"  
word\_cli read \--file "report.docx" \--range "5:10"

\# \[Safe In-Place Mutation\] Run-fragmentation-safe replacements / Highlight terms  
word\_cli replace \--file "report.docx" \--pairs "{{NAME}}=John,{{DATE}}=2026-09-01"  
word\_cli highlight \--file "report.docx" \--text "Review Item" \--color yellow

\# \[Paragraph Formatting\] Font, size (pt), indentation, and format strip  
word\_cli para-format \--file "report.docx" \--index 2 \--font "Arial" \--size 16.0 \--bold true \--indent 2.0 \--spacing 1.5

\# \[Table Grid Engine\] Direct Markdown extraction / Set cell value / Insert row  
word\_cli read-table \--file "report.docx" \--index 0 \--format markdown  
word\_cli set-cell \--file "report.docx" \--index 0 \--row 1 \--col 2 \--val "100.0"  
word\_cli insert-row \--file "report.docx" \--index 0 \--row 2 \--cells "ID,Metric,Value"

\# \[Section & Page Layout\] Switch orientation / Dynamic restartable page numbering  
word\_cli page \--file "report.docx" \--orient landscape \--size A4  
word\_cli set-footer \--file "report.docx" \--text "Confidential · Page " \--page-num \--restart 1

### **2\. Excel CLI (excel\_cli)**

\# \[Inspection & Discovery\] Metadata scanning / Full-text keyword search  
excel\_cli inspect \--file "data.xlsx"  
excel\_cli search \--file "data.xlsx" \--keyword "Budget Summary"

\# \[Range Slicing\] Read 2D matrix slice  
excel\_cli read \--file "data.xlsx" \--range "A1:D10"

\# \[Cell Injection & Calculation\] Formula injection / Row append  
excel\_cli cell \--file "data.xlsx" \--coord "C1" \--value "=SUM(A1:B1)"  
excel\_cli append \--file "data.xlsx" \--values "Hardware" "12" "5600.0"

\# \[Conversion\] High-speed CSV export with UTF-8 BOM  
excel\_cli convert \--input "data.xlsx" \--output "export.csv" \--sheet "Sheet1"

\# \[Grid & Aesthetics\] Row/column insertion / Header styling  
excel\_cli grid \--file "data.xlsx" \--action insert-row \--target 2 \--count 1  
excel\_cli style \--file "data.xlsx" \--coord "A1:E1" \--bold true \--font-color "FFFFFF" \--bg-color "4F81BD" \--border true

### **3\. PDF CLI (pdf\_cli)**

\# \[Cognitive Cockpit\] Inspect outline tree, total pages, and paper dimensions  
pdf\_cli inspect \--file "doc.pdf"  
pdf\_cli inspect \--file "doc.pdf" \--only outline  
pdf\_cli inspect \--file "doc.pdf" \--only layout

\# \[Targeted Drilling\] Read section by heading / page slice / noise removal / Markdown export  
pdf\_cli extract-text \--file "doc.pdf" \--heading "Chapter 3 Technical Specs" \--clean-noise  
pdf\_cli extract-text \--file "doc.pdf" \--page 1 \--clean-noise  
pdf\_cli extract-text \--file "doc.pdf" \--range "1:10" \--clean-noise \--format md

\# \[Geometry & BBox Audit\] Extract precise text stream with \[x, y, width, height\] bounding boxes  
pdf\_cli extract-text \--file "doc.pdf" \--page 1 \--with-bbox

\# \[Financial Audit & Layout\] Borderless table X-projection clustering / Indent tree / Overlap detection  
pdf\_cli inspect-layout \--file "report.pdf" \--page 68

\# \[RAG Knowledge Base Chunking\] Heading-driven chunking / Sliding window (breadcrumbs \+ token estimates)  
pdf\_cli chunk \--file "doc.pdf" \--strategy heading \--clean-noise  
pdf\_cli chunk \--file "doc.pdf" \--strategy sliding \--max-chars 800 \--overlap 100 \--clean-noise

\# \[Visual Slicing & Assets\] Bitmap extraction / 300DPI page rasterization (prepping for OCR)  
pdf\_cli extract-images \--file "doc.pdf" \--out-dir "./images/"  
pdf\_cli render-page \--file "doc.pdf" \--page 1 \--dpi 300 \--out "page\_1.png"

\# \[Assembly & Maintenance\] Safe multi-PDF merge / Page extraction / 90° rotation / Watermarking  
pdf\_cli merge \--files "part1.pdf,part2.pdf" \--output "merged.pdf"  
pdf\_cli extract-pages \--file "doc.pdf" \--pages "1:5" \--output "cut.pdf"  
pdf\_cli rotate \--file "doc.pdf" \--pages "1:3" \--angle 90 \--output "rotated.pdf"  
pdf\_cli watermark \--file "doc.pdf" \--text "CONFIDENTIAL" \--opacity 0.2 \--output "marked.pdf"

## **🗺️ Project Roadmap**

* \[x\] **Phase 1: Core Document Bedrock**  
  * \[x\] Complete read, write, styling, and CSV pipelines for excel\_cli.  
  * \[x\] Complete OpenXML packaging, cockpit inspection, Markdown import, paragraph formatting, table operations, image management, and header/footer engines for word\_cli.  
  * \[x\] Complete outline inspection, CMap font fallback, geometry noise removal, financial layout audit, RAG chunking, 300DPI rendering, page assembly, and watermarking for pdf\_cli.  
  * \[x\] Enforce standardized JSON-first contract outputs and anti-fragile error logging.  
* \[ \] **Phase 1: Document Tooling Completion**  
  * \[ \] Build ppt\_cli: Programmatic slide generation, shape manipulation, and master styling.  
* \[ \] **Phase 2: System-Level Computer Control**  
  * \[ \] Build automation\_script: Sandboxed, multi-shell command orchestration.  
  * \[ \] Build cache: Intermediate Agent artifact and state management.  
  * \[ \] Build computer\_control: Native Windows desktop automation, event injection, and window scheduling.  
  * \[ \] Build web\_scrape: High-performance static and dynamic web content normalization.

## **💬 Community & Feedback**

A purpose-built, high-velocity CLI toolkit tailored for LLM Agents transforms human-AI collaboration.

If this project helps streamline your workflows, **feel free to drop a Star ⭐ and share your thoughts in Issues and Discussions**\! Your insights guide the evolution of this suite.

## **📄 License**

This repository is distributed under the [MIT License](http://docs.google.com/LICENSE).