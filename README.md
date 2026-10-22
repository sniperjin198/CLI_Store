# **🚀 GooseCliBuilder / Computer Controller Suite**

> **打造一套完全符合个人工作习惯与大模型 Agent 深度协同的原生高性能自动化工具箱（Computer Controller 扩展套件）。**

> 第一阶段目标：利用 AI 辅助编码（AICode），构建自主可控、极速响应、无环境依赖的本地系统级控制与办公自动化底座。欢迎关注、Star 与多多鼓励！✨

## **💡 项目理念与愿景 (Vision)**

在日常办公自动化与大语言模型（LLM / Agent 如 Goose, Qwen）落地的场景中，传统 Python / Node.js 脚本常常面临**启动冷延迟高、运行环境臃肿、第三方依赖冲突、Windows COM 独占崩溃**等痛点。

本项目基于 **Rust 系统级语言** 全程构建，坚持以下核心哲学：

1. **原生无依赖 (Zero Dependency)**：编译即交付单文件二进制（.exe），无须安装 Office、Python 运行时或庞大解释器。

2. **结构化契约 (JSON-First & No Hex Errors)**：所有命令标准输出统一返回规范单行/多行 JSON，错误信息大白话、语义化枚举，杜绝难懂的十六进制底层错误码，让大模型零成本直读。

3. **三步人机认知闭环 (Three-Step Cognitive Flow)**：针对超长文档提供“宏观驾驶舱建构 -> 章节下钻 -> 微观精准落盘修改”流程，杜绝长文本爆 Token，保证精确性。

4. **超级门面与引擎协同 (Super-CLI & Engine Facade)**：复杂任务通过统一的门面 CLI 对外提供极致简洁的交互接口，底层精准路由调度专业工业级引擎，避免大模型决策过载。

## **📦 工具箱矩阵 (Toolbox Matrix)**

### **一、 已就绪交付模块 (Production Ready)**

| 工具名称 | 对应格式/能力 | 核心功能概述 | 
| ----- | ----- | ----- | 
| **word_cli** | .docx 文档处理 | 大纲树驾驶舱自省、Markdown 极速转交付级 Word、跨 Run 占位符批量注入、表格矩阵读写与行列动态伸缩、DrawingML 嵌入图片排版、野生文档格式刷平、分节页面横纵混排与动态页码重编。 | 
| **excel_cli** | .xlsx 表格处理 | 工作簿元数据扫描、全文快速检索、矩阵范围读写与预览、单单元格公式写入、行列动态增删、单元格样式美化（字号/背景色/边框）及 UTF-8 BOM CSV 高速导出。 | 
| **pdf_cli** | .pdf 几何手术刀 | 宏观大纲自省、ToUnicode 中文字体容错回退、多栏阅读顺序重排、页眉页脚几何降噪、无框财报表格列投影与科目缩进审计、RAG 向量切片、位图解包与 300DPI 渲染、**多图合并装配 PDF**、页面拼接抽取、90 度旋转校正及半透明水印注入。 | 
| **webscrape** | 网页感知与归档 Super-CLI | Agent-First 纯内存免写盘感知提炼（微信公众号/技术博客正文清洗）、大模型直调容错（自动识别 `--url`）、资讯超链接差量监控、页面 `<table>` 表格抽取；底层自动分流调度专用引擎实现单文件 HTML 全要素离线打包、Office/PDF 附件多线程极速直流下载与流媒体音视频抓取。 | 

### **二、 正在规划与实施模块 (In Active Development)**

| 规划工具 / 模块 | 核心能力规划定位 | 
| ----- | ----- | 
| **ppt_cli** | 演示文稿幻灯片自动化创建、母版占位符注入、形状/文本框排版与图形图表批量生成。 | 
| **automation_script** | 跨平台自动化脚本调度引擎，安全创建、隔离运行与流式捕获 PowerShell、Bash 等自动化脚本。 | 
| **cache** | 专用缓存管理引擎，统一管理、检索、查看大模型与自动化工作流中的临时文件与持久化数据。 | 
| **computer_control** | 系统级计算机控制与自动化操作系统（GUI 事件交互、窗口管理、进程调度、环境控制），实现真实桌面端 Agent 闭环。 | 

## **🛠️ 安装与构建 (Build & Install)**

本项目基于现代稳定版 Rust 工具链构建：

```
# 1. 编译全部已就绪组件 (Release 优化构建)
cargo build --release

# 2. 部署到系统全局 CLI 目录 (推荐 E:\.Cli\)
cmd.exe /c "if not exist E:\.Cli mkdir E:\.Cli && copy /y target\release\*.exe E:\.Cli\"

# 3. 部署专用依赖引擎 (从项目 bin\ 目录拷贝至 E:\.Cli\bin\)
cmd.exe /c "if not exist E:\.Cli\bin mkdir E:\.Cli\bin && copy /y bin\*.* E:\.Cli\bin\"

```

建议将 `E:\.Cli\` 添加至系统环境变量 `PATH` 中，即可在任意命令行与 Agent 提示词环境中随处调用。

## **📖 核心组件速查指南 (Cheat Sheet)**

### **1. Word CLI (word_cli) 常用操作**

```
# 【新建与转换】派生新建 / Markdown 一键转合规 Word (自动继承母版样式)  
word_cli create --file "output.docx" [--template "tpl.docx"]  
word_cli import --input "report.md" --output "report.docx" [--template "tpl.docx"]  

# 【宏观认知驾驶舱】秒级摸清文档骨架，不输出漫天长文本  
word_cli inspect --file "report.docx"  
word_cli inspect --file "report.docx" --only outline  
word_cli inspect --file "report.docx" --only tables  

# 【定向下钻精读】按大纲章节读取 / 指定段落范围精读  
word_cli read --file "report.docx" --heading "第一章 项目概述"  
word_cli read --file "report.docx" --range "5:10"  

# 【正文安全修改】跨 Run 占位符批量注入 / 关键词高亮  
word_cli replace --file "report.docx" --pairs "{{NAME}}=张三,{{DATE}}=2026-09-01"  
word_cli highlight --file "report.docx" --text "重要审查项" --color yellow  

# 【排版精细调整】字体、字号、首行缩进与格式刷平  
word_cli para-format --file "report.docx" --index 2 --font "黑体" --size 18.0 --bold true --indent 2.0 --spacing 1.5  

# 【表格矩阵读写】Markdown 表格直读 / 单元格改写 / 动态加行  
word_cli read-table --file "report.docx" --index 0 --format markdown  
word_cli set-cell --file "report.docx" --index 0 --row 1 --col 2 --val "100.0"  
word_cli insert-row --file "report.docx" --index 0 --row 2 --cells "序号,指标名称,实测值"  

# 【版式与公文规范】横纵向混排 / 动态页码从第 1 页起算  
word_cli page --file "report.docx" --orient landscape --size A4  
word_cli set-footer --file "report.docx" --text "内部机密 · 第" --page-num --restart 1

```

### **2. Excel CLI (excel_cli) 常用操作**

```
# 【工作簿自省与检索】概览信息扫描 / 全文内容检索  
excel_cli inspect --file "data.xlsx"  
excel_cli search --file "data.xlsx" --keyword "预算汇总"  

# 【矩阵范围读取】切片读取指定坐标区域  
excel_cli read --file "data.xlsx" --range "A1:D10"  

# 【数据写入与计算】单元格公式注入 / 末行动态追加  
excel_cli cell --file "data.xlsx" --coord "C1" --value "=SUM(A1:B1)"  
excel_cli append --file "data.xlsx" --values "工程设备" "12" "5600.0"  

# 【格式导出】一键导出带 BOM 无乱码 UTF-8 CSV  
excel_cli convert --input "data.xlsx" --output "export.csv" --sheet "Sheet1"  

# 【网格与样式排版】行列增删 / 表头颜色样式设置  
excel_cli grid --file "data.xlsx" --action insert-row --target 2 --count 1  
excel_cli style --file "data.xlsx" --coord "A1:E1" --bold true --font-color "FFFFFF" --bg-color "4F81BD" --border true

```

### **3. PDF CLI (pdf_cli) 常用操作**

```
# 【宏观认知驾驶舱】秒级摸清文档大纲书签树、总页数与纸张规格  
pdf_cli inspect --file "doc.pdf"  
pdf_cli inspect --file "doc.pdf" --only outline  
pdf_cli inspect --file "doc.pdf" --only layout  

# 【定向下钻精读】按大纲章节读取 / 指定页提取 / 降噪过滤与 Markdown 导出  
pdf_cli extract-text --file "doc.pdf" --heading "第三章 技术方案" --clean-noise  
pdf_cli extract-text --file "doc.pdf" --page 1 --clean-noise  
pdf_cli extract-text --file "doc.pdf" --range "1:10" --clean-noise --format md  

# 【物理坐标审计】提取带物理外接矩形 [x, y, width, height] 的精准文字流  
pdf_cli extract-text --file "doc.pdf" --page 1 --with-bbox  

# 【财报几何自省】无框表格 X 轴列投影聚类 / 会计科目缩进树推算 / 覆盖篡改异常检测  
pdf_cli inspect-layout --file "report.pdf" --page 68  

# 【RAG 知识库语义切片】大纲章节驱动分块 / 滑动窗口分块（带面包屑路径与 Token 估算）  
pdf_cli chunk --file "doc.pdf" --strategy heading --clean-noise  
pdf_cli chunk --file "doc.pdf" --strategy sliding --max-chars 800 --overlap 100 --clean-noise  

# 【视觉切片与素材提取】内嵌位图无损解包 / 页面 300DPI 高清渲染（支撑后续 OCR）  
pdf_cli extract-images --file "doc.pdf" --out-dir "./images/"  
pdf_cli render-page --file "doc.pdf" --page 1 --dpi 300 --out "page_1.png"  

# 【装配与合成】多图合成标准 PDF / 多文档安全合并 / 页面截取 / 90度旋转 / 半透明水印  
pdf_cli images-to-pdf --images "p1.png,p2.jpg,p3.webp" --output "album.pdf" [--fit-page]
pdf_cli merge --files "part1.pdf,part2.pdf" --output "merged.pdf"  
pdf_cli extract-pages --file "doc.pdf" --pages "1:5" --output "cut.pdf"  
pdf_cli rotate --file "doc.pdf" --pages "1:3" --angle 90 --output "rotated.pdf"  
pdf_cli watermark --file "doc.pdf" --text "内部评审 · 请勿外传" --opacity 0.2 --output "marked.pdf"

```

### **4. 网页感知 (webscrape) 常用操作**

```
# 【纯读感知（零落盘）】控制台直接输出纯净正文 Markdown 供大模型理解思考
webscrape get "https://mp.weixin.qq.com/s/xxxxxx"
# 大模型扁平直调容错（自动识别为纯读）
webscrape --url "https://example.com/article"

# 【知识图文归档】提取正文并自动将插图下载至本地同名 _images 文件夹（保持相对路径）
webscrape save "https://mp.weixin.qq.com/s/xxxxxx" --format md --with-images -o "E:/Doc/文章.md"

# 【单文件 HTML 网页离线保存】自动分流调度 bin\monolith.exe 打包样式与资源
webscrape save "https://example.com/page" --format html -o "E:/WebArchives/网页.html"

# 【二进制附件极速多线程下载】自动分流调度 bin\aria2c.exe（支持 xlsx/pdf/zip 等）
webscrape save "https://gov.cn/files/名单.xlsx" -o "E:/Office/名单.xlsx"

# 【流媒体音视频提取】自动分流调度 bin\yt-dlp.exe（支持视频下载与纯音频提取）
webscrape save "https://www.bilibili.com/video/BVxxxxxx" --format video -o "E:/Videos/视频.mp4"
webscrape save "https://www.bilibili.com/video/BVxxxxxx" --format audio -o "E:/Audio/音频.mp3"
webscrape save "https://www.bilibili.com/video/BVxxxxxx" --resolution 1080 -o "E:/Videos/高清.mp4"

# 【要闻与列表巡检】扫描页面超链接绝对路径与关联发布日期
webscrape links "https://www.gov.cn/yaowen/" --limit 10

# 【表格与卡片抽取】提取网页中 <table> 标签二维数组或按 CSS 选择器抽取卡片
webscrape extract "https://example.com/data" --limit 1
webscrape extract "https://news.ycombinator.com/" --selector "tr.athing" --limit 10

```

## **🧩 第三方核心引擎与开源合规说明 (Third-Party Engines & Credits)**

为了避免重复造轮子，`webscrape` 采用 **Super-CLI 外观门面模式**。它专注于高效的内容感知、正文清洗与指令调度，并将特定的底层数据传输与打包任务透明地委托给存放在 `bin/` 目录下的三个顶级开源工业级工具。

本项目与以下外部工具仅以独立外部子进程命令行调用（Independent Process Invocation）的形式协同工作，不包含其内部源代码链接：

| 引擎工具 | 对应程序 | 原作者 / 开源项目 | 开源协议 (License) | 在本项目中的职责定位 | 
| ----- | ----- | ----- | ----- | ----- | 
| **Aria2** | `aria2c.exe` | [aria2/aria2](https://github.com/aria2/aria2) | **GPL-2.0+** | 负责 `.xlsx`、`.pdf`、`.zip` 等大文件及附件的极速 4 连接并发与断点续传下载。 | 
| **Monolith** | `monolith.exe` | [Y2Z/monolith](https://github.com/Y2Z/monolith) | **CC0-1.0 (Public Domain)** | 负责将完整网页的 CSS、字体与图片等内嵌为单一离线 HTML 网页。 | 
| **yt-dlp** | `yt-dlp.exe` | [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) | **The Unlicense (Public Domain)** | 负责针对主流视频平台的流媒体音视频多格式解析、清晰度调度与下载。 | 

*致谢：向以上优秀开源项目的作者与全球贡献者致以崇高敬意！*

## **🗺️ 阶段路线图与演进规划 (Roadmap)**

* \[x\] **第一阶段·办公核心数据底座**

  * \[x\] 完成 excel_cli 全套只读检索、数据读写、公式与行列排版引擎。

  * \[x\] 完成 word_cli OpenXML 解包打包、驾驶舱认知、Markdown 导入、文字精读、表格增删、DrawingML 图片管理及公文页眉页脚排版。

  * \[x\] 完成 pdf_cli 宏观大纲自省、高保真几何降噪抽取、无框表格列投影与科目缩进审计、RAG 向量切片、300DPI 渲染、**多图转 PDF 合并**、文档装配与半透明水印注入。

  * \[x\] 构建 webscrape：完成网页文章纯内存直读、正文提纯，并依托专用 `bin/` 引擎群实现多线程附件下载、HTML 完整打包及流媒体抓取。

  * \[x\] 全面统一 JSON-First 契约输出与抗脆弱性错误捕获。

* \[ \] **第一阶段·办公文档能力补全**

  * \[ \] 构建 ppt_cli：实现幻灯片批量创建、文本图形替换与模版样式渲染。

* \[ \] **第二阶段·系统级扩展与自动化闭环 (Computer Controller)**

  * \[ \] 构建 automation_script：PowerShell / Bash 自动化脚本隔离沙盒执行与结果采集。

  * \[ \] 构建 cache：多任务流水线与 Agent 上下文持久化缓存中枢。

  * \[ \] 构建 computer_control：鼠标键盘原生模拟、窗口状态捕获与 Windows 原生桌面级自动化支持。

## **💬 致谢与鼓励**

一套真正顺手、高反应速度、专为大模型量身定制的 CLI 工具集能够大幅提升人机协同与日常开发的幸福感。

如果这个项目对你的日常工作或 AI 自动化探索有所启发，**欢迎点亮 Star ⭐，更欢迎在 Issues / Discussions 中提出宝贵的使用意见与功能需求**！你的支持是持续完善这套工具箱的最大动力！

## **📄 授权协议 (License)**

本项目核心代码采用 [MIT License](LICENSE) 授权许可，开源透明，自由分发与商业友好。第三方引擎组件请遵循其各自的原生许可。

# **🌐 English Documentation**

# **🚀 GooseCliBuilder / Computer Controller Suite**

> **A native, high-performance automation toolbox designed for personalized workflows and deep collaboration with LLM Agents (Computer Controller Extension Suite).**

> Phase 1 Objective: Harness AI-assisted coding (AICode) to establish a sovereign, zero-dependency, ultra-fast system control and document automation bedrock. Stars ⭐ and feedback are warmly appreciated!

## **💡 Vision & Core Philosophy**

In traditional office automation and Agentic tool workflows (e.g., Goose, Qwen), traditional Python or Node.js scripts frequently encounter **cold-start latency, bloated runtime environments, brittle dependency conflicts, and Windows COM exclusivity crashes**.

Engineered from the ground up in **pure Rust**, this suite adheres to four fundamental design principles:

1. **Zero Runtime Dependency**: Compiles into standalone, single-file binaries (.exe). Runs instantly without Microsoft Office installations, Python interpreters, or auxiliary runtimes.

2. **JSON-First Contract & Semantic Errors**: Every command prints deterministic, parse-safe JSON to stdout. Underlying failures are translated into explicit, semantic string enumerations, eradicating obscure hexadecimal error codes so LLMs can reason over errors immediately.

3. **Three-Step Cognitive Flow**: For massive documents, the tool enforces an **Overview Cockpit -> Section Drilling -> In-Place Atomic Mutation** interaction loop, mitigating context-window exhaustion and token waste.

4. **Super-CLI & Engine Delegation**: High-level commands expose a clean unified interface while delegating specialized heavy-lifting to dedicated engines, minimizing Agent decision overhead.

## **📦 Toolbox Matrix**

### **1. Production Ready Modules**

| Tool Name | Scope / Target | Core Capabilities | 
| ----- | ----- | ----- | 
| **word_cli** | .docx Documents | Document outline cockpit, zero-dependency Markdown-to-Word conversion, multi-run safe placeholder injection, 2D table manipulation with dynamic row/column stretching, DrawingML inline image embedding, wildcard formatting strip, multi-section page orientation, and dynamic restartable page numbering. | 
| **excel_cli** | .xlsx Spreadsheets | Workbook metadata inspection, full-text cell keyword search, range slicing, single-cell formula injection, dynamic row/column mutation, cell styling (color, font, borders), and BOM-prefixed UTF-8 CSV exports. | 
| **pdf_cli** | .pdf Scalpel | Outline tree inspection, CMap/ToUnicode Chinese font fallback, geometric header/footer noise removal, multi-column reading order reconstruction, borderless table column clustering and accounting indentation analysis, RAG semantic chunking, bitmap unpack, 300DPI rendering, **images-to-PDF assembly**, page merge/split, rotation, and watermark stamping. | 
| **webscrape** | Web Perception & Archival Super-CLI | Agent-First zero-disk perception (clean article extraction for WeChat/blogs), tolerant parameter parsing (`--url`), link stream monitoring, `<table>` data extraction; seamless dispatch to dedicated sub-engines for single-file HTML bundling, multi-threaded binary downloads, and media stream acquisition. | 

### **2. Modules In Active Development**

| Planned Module | Scope & Objectives | 
| ----- | ----- | 
| **ppt_cli** | Programmatic presentation authoring, master placeholder injection, shape/textbox layout, and automated charting. | 
| **automation_script** | Cross-platform automation script runner for creating, sandboxing, and capturing PowerShell/Bash workflows. | 
| **cache** | Specialized persistent cache engine to store, retrieve, and inspect intermediate artifacts across Agent executions. | 
| **computer_control** | Native OS computer controller (GUI mouse/keyboard simulation, window state management, process orchestration). | 

## **🛠️ Build & Installation**

Built with the stable Rust toolchain:

```
# 1. Build all production-ready modules with release optimizations  
cargo build --release

# 2. Deploy to global CLI directory (e.g., E:\.Cli\)  
cmd.exe /c "if not exist E:\.Cli mkdir E:\.Cli && copy /y target\release\*.exe E:\.Cli\"

# 3. Deploy dedicated engines (from bin\ to E:\.Cli\bin\)
cmd.exe /c "if not exist E:\.Cli\bin mkdir E:\.Cli\bin && copy /y bin\*.* E:\.Cli\bin\"

```

Adding `E:\.Cli\` to your system PATH allows both human users and AI Agents to invoke the tools from any directory or prompt context.

## **📖 Component Cheat Sheet**

### **1. Word CLI (word_cli)**

```
# [Creation & Conversion] New blank document / Markdown-to-Word conversion  
word_cli create --file "output.docx" [--template "tpl.docx"]  
word_cli import --input "report.md" --output "report.docx" [--template "tpl.docx"]  

# [Cognitive Cockpit] Inspect document skeleton without dumping raw text  
word_cli inspect --file "report.docx"  
word_cli inspect --file "report.docx" --only outline  
word_cli inspect --file "report.docx" --only tables  

# [Targeted Drilling] Read section by heading / slice paragraphs by range  
word_cli read --file "report.docx" --heading "Chapter 1 Project Overview"  
word_cli read --file "report.docx" --range "5:10"  

# [Safe In-Place Mutation] Run-fragmentation-safe replacements / Highlight terms  
word_cli replace --file "report.docx" --pairs "{{NAME}}=John,{{DATE}}=2026-09-01"  
word_cli highlight --file "report.docx" --text "Review Item" --color yellow  

# [Paragraph Formatting] Font, size (pt), indentation, and format strip  
word_cli para-format --file "report.docx" --index 2 --font "Arial" --size 16.0 --bold true --indent 2.0 --spacing 1.5  

# [Table Grid Engine] Direct Markdown extraction / Set cell value / Insert row  
word_cli read-table --file "report.docx" --index 0 --format markdown  
word_cli set-cell --file "report.docx" --index 0 --row 1 --col 2 --val "100.0"  
word_cli insert-row --file "report.docx" --index 0 --row 2 --cells "ID,Metric,Value"  

# [Section & Page Layout] Switch orientation / Dynamic restartable page numbering  
word_cli page --file "report.docx" --orient landscape --size A4  
word_cli set-footer --file "report.docx" --text "Confidential · Page " --page-num --restart 1

```

### **2. Excel CLI (excel_cli)**

```
# [Inspection & Discovery] Metadata scanning / Full-text keyword search  
excel_cli inspect --file "data.xlsx"  
excel_cli search --file "data.xlsx" --keyword "Budget Summary"  

# [Range Slicing] Read 2D matrix slice  
excel_cli read --file "data.xlsx" --range "A1:D10"  

# [Cell Injection & Calculation] Formula injection / Row append  
excel_cli cell --file "data.xlsx" --coord "C1" --value "=SUM(A1:B1)"  
excel_cli append --file "data.xlsx" --values "Hardware" "12" "5600.0"  

# [Conversion] High-speed CSV export with UTF-8 BOM  
excel_cli convert --input "data.xlsx" --output "export.csv" --sheet "Sheet1"  

# [Grid & Aesthetics] Row/column insertion / Header styling  
excel_cli grid --file "data.xlsx" --action insert-row --target 2 --count 1  
excel_cli style --file "data.xlsx" --coord "A1:E1" --bold true --font-color "FFFFFF" --bg-color "4F81BD" --border true

```

### **3. PDF CLI (pdf_cli)**

```
# [Cognitive Cockpit] Inspect outline tree, total pages, and paper dimensions  
pdf_cli inspect --file "doc.pdf"  
pdf_cli inspect --file "doc.pdf" --only outline  
pdf_cli inspect --file "doc.pdf" --only layout  

# [Targeted Drilling] Read section by heading / page slice / noise removal / Markdown export  
pdf_cli extract-text --file "doc.pdf" --heading "Chapter 3 Technical Specs" --clean-noise  
pdf_cli extract-text --file "doc.pdf" --page 1 --clean-noise  
pdf_cli extract-text --file "doc.pdf" --range "1:10" --clean-noise --format md  

# [Geometry & BBox Audit] Extract precise text stream with [x, y, width, height] bounding boxes  
pdf_cli extract-text --file "doc.pdf" --page 1 --with-bbox  

# [Financial Audit & Layout] Borderless table X-projection clustering / Indent tree / Overlap detection  
pdf_cli inspect-layout --file "report.pdf" --page 68  

# [RAG Knowledge Base Chunking] Heading-driven chunking / Sliding window (breadcrumbs + token estimates)  
pdf_cli chunk --file "doc.pdf" --strategy heading --clean-noise  
pdf_cli chunk --file "doc.pdf" --strategy sliding --max-chars 800 --overlap 100 --clean-noise  

# [Visual Slicing & Assets] Bitmap extraction / 300DPI page rasterization (prepping for OCR)  
pdf_cli extract-images --file "doc.pdf" --out-dir "./images/"  
pdf_cli render-page --file "doc.pdf" --page 1 --dpi 300 --out "page_1.png"  

# [Assembly & Composition] Convert images to PDF / Merge PDFs / Page slice / 90° rotation / Watermark  
pdf_cli images-to-pdf --images "p1.png,p2.jpg,p3.webp" --output "album.pdf" [--fit-page]
pdf_cli merge --files "part1.pdf,part2.pdf" --output "merged.pdf"  
pdf_cli extract-pages --file "doc.pdf" --pages "1:5" --output "cut.pdf"  
pdf_cli rotate --file "doc.pdf" --pages "1:3" --angle 90 --output "rotated.pdf"  
pdf_cli watermark --file "doc.pdf" --text "CONFIDENTIAL" --opacity 0.2 --output "marked.pdf"

```

### **4. Web Scraping & Archiving Super-CLI (webscrape)**

```
# [Zero-Disk Perception] Print sanitized Markdown to console for Agent reading
webscrape get "https://mp.weixin.qq.com/s/xxxxxx"
# Tolerant argument format
webscrape --url "https://example.com/article"

# [Markdown Knowledge Archiving] Extract article and download embedded images to local directory
webscrape save "https://mp.weixin.qq.com/s/xxxxxx" --format md --with-images -o "E:/Doc/article.md"

# [Single-File Full HTML Offline Archival] Dispatched to bin\monolith.exe
webscrape save "https://example.com/page" --format html -o "E:/WebArchives/page.html"

# [High-Speed Binary Attachment Download] Dispatched to bin\aria2c.exe
webscrape save "https://gov.cn/files/report.xlsx" -o "E:/Office/report.xlsx"

# [Media Stream Extraction] Dispatched to bin\yt-dlp.exe
webscrape save "https://www.bilibili.com/video/BVxxxxxx" --format video -o "E:/Videos/video.mp4"
webscrape save "https://www.bilibili.com/video/BVxxxxxx" --format audio -o "E:/Audio/audio.mp3"
webscrape save "https://www.bilibili.com/video/BVxxxxxx" --resolution 1080 -o "E:/Videos/video_1080p.mp4"

# [Monitoring & Link Inspection] Absolute URLs and extracted publication dates
webscrape links "https://www.gov.cn/yaowen/" --limit 10

# [Structured Table & Card Extraction] <table> tags or custom CSS selectors
webscrape extract "https://example.com/data" --limit 1
webscrape extract "https://news.ycombinator.com/" --selector "tr.athing" --limit 10

```

## **🧩 Third-Party Engines & Credits**

To avoid reinventing the wheel, `webscrape` operates under a **Super-CLI Facade Architecture**. It focuses on web perception, content sanitization, and intent dispatching, delegating specific data transfer and serialization tasks to proven open-source CLI utilities placed in the `bin/` directory.

This project interacts with these utilities exclusively via **independent child process invocation**, without linking their internal source code:

| Engine Tool | Binary | Original Project / Author | License | Role in this Suite | 
| ----- | ----- | ----- | ----- | ----- | 
| **Aria2** | `aria2c.exe` | [aria2/aria2](https://github.com/aria2/aria2) | **GPL-2.0+** | Multi-connection chunked downloads for `.xlsx`, `.pdf`, `.zip` attachments. | 
| **Monolith** | `monolith.exe` | [Y2Z/monolith](https://github.com/Y2Z/monolith) | **CC0-1.0 (Public Domain)** | Inlining web pages into single standalone HTML archives. | 
| **yt-dlp** | `yt-dlp.exe` | [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) | **The Unlicense (Public Domain)** | Stream extraction, video format transcoding, and resolution filtering. | 

*Acknowledgments: Sincere gratitude to the authors and global contributors of these outstanding open-source projects!*

## **🗺️ Project Roadmap**

* \[x\] **Phase 1: Core Document Bedrock**

  * \[x\] Complete read, write, styling, and CSV pipelines for excel_cli.

  * \[x\] Complete OpenXML packaging, cockpit inspection, Markdown import, paragraph formatting, table operations, image management, and header/footer engines for word_cli.

  * \[x\] Complete outline inspection, CMap font fallback, geometry noise removal, financial layout audit, RAG chunking, 300DPI rendering, **images-to-PDF conversion**, page assembly, and watermarking for pdf_cli.

  * \[x\] Complete webscrape: zero-disk article perception, markdown cleanup, and multi-threaded engine routing via dedicated `bin/` dependencies.

  * \[x\] Enforce standardized JSON-first contract outputs and anti-fragile error logging.

* \[ \] **Phase 1: Document Tooling Completion**

  * \[ \] Build ppt_cli: Programmatic slide generation, shape manipulation, and master styling.

* \[ \] **Phase 2: System-Level Computer Control**

  * \[ \] Build automation_script: Sandboxed, multi-shell command orchestration.

  * \[ \] Build cache: Intermediate Agent artifact and state management.

  * \[ \] Build computer_control: Native Windows desktop automation, event injection, and window scheduling.

## **💬 Community & Feedback**

A purpose-built, high-velocity CLI toolkit tailored for LLM Agents transforms human-AI collaboration.

If this project helps streamline your workflows, **feel free to drop a Star ⭐ and share your thoughts in Issues and Discussions**! Your insights guide the evolution of this suite.

## **📄 License**

This repository's core code is distributed under the [MIT License](LICENSE). Third-party engine utilities in `bin/` are governed by their respective upstream open-source licenses.