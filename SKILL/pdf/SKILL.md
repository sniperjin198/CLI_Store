---
name: pdf
description: "Use this skill for any PDF document processing using the local CLI tool E:\\.Cli\\pdf_cli.exe."
license: Proprietary. LICENSE.txt has complete terms
---

# PDF 几何自省与高精装配专家技能 (pdf)

本技能定义了本地高性能 PDF 命令行工具 `E:\.Cli\pdf_cli.exe` 的调用契约与交互行为规范。

## 一、核心调用范式 (必须严格对照此格式执行)

遇到任何包含【中文】、【空格】、【盘符】的路径，统一使用单层标准双引号包裹，一步到位直接输出，绝对禁止分步重命名：

### 1. 宏观感知与骨架摸底 (Step 1 - 最常用认知入口)
处理任何未知 PDF 或回答内容前，必须首先自省摸清大纲结构、总页数与版面：
```bash
E:\.Cli\pdf_cli.exe inspect --file "E:/GooseProjects/Doc/方案汇报.pdf"
E:\.Cli\pdf_cli.exe inspect --file "E:/GooseProjects/Doc/方案汇报.pdf" --only outline
```

### 2. 文本高保真提取与定向精读 (Step 2 - 核心阅读能力)
根据自省获得的大纲标题定向精读，或提取特定单页/页码区间（带自动降噪）：
```bash
E:\.Cli\pdf_cli.exe extract-text --file "E:/GooseProjects/Doc/方案汇报.pdf" --heading "工程概况" --clean-noise
E:\.Cli\pdf_cli.exe extract-text --file "E:/GooseProjects/Doc/方案汇报.pdf" --page 1 --clean-noise
E:\.Cli\pdf_cli.exe extract-text --file "E:/GooseProjects/Doc/方案汇报.pdf" --range "1:5" --clean-noise --format md
```

### 3. 版式几何自省与无框表格审计 (Step 3 - 财报/工程图表推算)
推算页面列宽投影、会计科目缩进层级树、检测图层遮盖异常：
```bash
E:\.Cli\pdf_cli.exe inspect-layout --file "E:/GooseProjects/Doc/工程预算表.pdf" --page 3
```

### 4. RAG 知识库语义切片预处理
```bash
E:\.Cli\pdf_cli.exe chunk --file "E:/GooseProjects/Doc/技术规范.pdf" --strategy heading --clean-noise
```

### 5. 多图直接打包成 PDF (图片/扫描件无损装配交付)
将目录中的照片、扫描件、图纸（支持 jpg/png/webp/bmp 等格式混排）按图片原尺寸 1:1 无损生成独立页面的 PDF：
```bash
# 目录全量多图打包 (优先使用 --dir，按文件名自然序自动编排页码)
E:\.Cli\pdf_cli.exe images-to-pdf --dir "E:/GooseProjects/Doc/扫描底件" --output "E:/GooseProjects/Doc/扫描装配全集.pdf"

# 离散图片指定打包 (可使用精简别名 img2pdf)
E:\.Cli\pdf_cli.exe img2pdf --files "E:/Doc/封面.png,E:/Doc/现场图.jpg,E:/Doc/航测.webp" --output "E:/Doc/图集.pdf"
```

### 6. 物理装配、批量校正与交付 (按需执行的手术刀)
```bash
# 目录全量合并 (优先使用 --dir，原生免疫中文与空格)
E:\.Cli\pdf_cli.exe merge --dir "E:/GooseProjects/Doc/图纸目录" --output "E:/GooseProjects/Doc/图纸目录/图纸合集.pdf"

# 批量旋转与批量水印
E:\.Cli\pdf_cli.exe rotate --dir "E:/GooseProjects/Doc/图纸目录" --angle 90
E:\.Cli\pdf_cli.exe watermark --dir "E:/GooseProjects/Doc/图纸目录" --text "内部评审"
```

---

## 二、核心执行守则 (硬性约束)

1. **唯一合法工具绝对路径 (注意点号)**：
   * 必须且仅允许调用带点号的绝对路径：`E:\.Cli\pdf_cli.exe`。
   * **绝对禁止拼写为 `E:\Cli\pdf_cli.exe`**。严禁编写 Python 脚本、PowerShell 或调用外部库。
2. **完整盘符与正斜杠法则**：
   * 传递给 CLI 的所有路径，**必须统一使用正斜杠 `/` 且包含盘符**（如 `"E:/GooseProjects/Doc/report.pdf"`）。
   * **绝对禁止省略盘符写成 `/GooseProjects/...`**（省略盘符在跨驱动器或子终端环境下会导致致命的路径丢失）。
   * **严禁输出临时文件名后再用 move 重命名**，`--output` 原生支持直接指定最终中文目标文件名。
3. **三步人机协同认知闭环 (Three-Step Cognitive Flow)**：
   * **宏观建构 (Step 1)**：处理未知 PDF 时，首先调用 `inspect` 秒级摸清骨架（总页数、纸张尺寸与方向、书签大纲树），严禁盲目全量抽取全文。
   * **章节下钻 (Step 2)**：通过 `extract-text --heading "章节名"` 定向调取章节内容提炼汇报。
   * **微观精读与几何审计 (Step 3)**：通过 `extract-text --page N` 精读单页，或通过 `inspect-layout --page N` 审计无框表格、列宽投影与科目缩进。
4. **禁止任何冗余探测与校验读回**：
   * CLI 每次执行成功均会向 stdout 返回标准 JSON，包含 `{"status":"success", ...}`。
   * **执行写操作后，严禁使用 shell 命令（如 `dir`、`ls`、`Test-Path`、`Get File Info`）去二次检验文件是否存在或查看大小**。看到 JSON 中的 `status: "success"` 即代表物理写入成功，直接进入下一步或向用户汇报。
5. **装配与合并 (Images-To-Pdf / Merge) 行为铁律**：
   * **目录扫描优先**：凡是处理某目录下的图片打包或 PDF 合并，**必须且只能使用 `--dir`**。严禁外部枚举拼接 `--files`，严禁循环反复调用。
   * **全量覆写原则**：`images-to-pdf` 和 `merge` 均为一次性全量生成/覆写，**绝不是追加（Append）**。CLI 每次执行都会直接生成完整目标文件，严禁拆分为多次两两操作。
   * **多图打包尺寸自适应**：`images-to-pdf` 会将每张图片的原生像素宽高直接映射为独立页面的 MediaBox，无需人工计算或缩放 A4 比例，保持原汁原味无拉伸。
6. **Shell 参数包裹与开关规范**：
   * 遇到空格或中文参数，在参数外侧加单层标准双引号：`--file "E:/GooseProjects/Doc/工程方案.pdf"`。**严禁出现 `\"` 或 `""` 嵌套转义**。
   * 布尔类型参数（如 `--clean-noise`、`--with-bbox`）均为**开关型标志（Flag）**，出现即生效，严禁后接 `true/false`。

---

## 三、完整命令语法速查手册

### 1. 认知导航与宏观驾驶舱
* **全局宏观自省（返回版本、加密、页数、元数据、纸张尺寸、书签树）**
  `E:\.Cli\pdf_cli.exe inspect --file "<文件路径>"`
* **自省轻量过滤（降低 Token 消耗）**
  `E:\.Cli\pdf_cli.exe inspect --file "<文件路径>" --only metadata|layout|outline`

### 2. 文本高保真抽取与精读下钻
* **提取单页纯净文本（带页眉页脚降噪）**
  `E:\.Cli\pdf_cli.exe extract-text --file "<文件路径>" --page <页码> --clean-noise`
* **按页码区间提取并转换为结构化 Markdown（XY-Cut 启发式多栏重排）**
  `E:\.Cli\pdf_cli.exe extract-text --file "<文件路径>" --range "<起始:结束>" --clean-noise --format md`
* **按大纲章节标题定向精读（按书签自动切分章节）**
  `E:\.Cli\pdf_cli.exe extract-text --file "<文件路径>" --heading "<章节名称>" --clean-noise`
* **高精坐标级精读（输出包含每个文本块 [x, y, width, height] 物理坐标矩形）**
  `E:\.Cli\pdf_cli.exe extract-text --file "<文件路径>" --page <页码> --with-bbox`

### 3. 几何版式自省与财报审计
* **页面几何自省（推算无框表格 X 轴列宽投影、会计科目缩进树、检测遮盖异常）**
  `E:\.Cli\pdf_cli.exe inspect-layout --file "<文件路径>" --page <页码>`

### 4. RAG 知识库语义切片预处理
* **基于书签大纲驱动切片（按章节层级切分，带面包屑路径与 Token 估算）**
  `E:\.Cli\pdf_cli.exe chunk --file "<文件路径>" --strategy heading --clean-noise`
* **基于滑动窗口重叠切片（针对无大纲文档，均匀切分）**
  `E:\.Cli\pdf_cli.exe chunk --file "<文件路径>" --strategy sliding --max-chars 800 --overlap 100 --clean-noise`

### 5. 多图直接打包成 PDF (支持照片/扫描件/图纸混排，原比例自适应)
* **文件夹一键打包多图为 PDF（最常用，自动按文件名正序扫描目录下所有图片）**
  `E:\.Cli\pdf_cli.exe images-to-pdf --dir "<输入图片文件夹路径>" --output "<合并输出.pdf>"`
* **离散图片列表打包（支持别名 img2pdf，多图逗号分隔）**
  `E:\.Cli\pdf_cli.exe img2pdf --files "<图片1.jpg>,<图片2.png>,<图片3.webp>" --output "<合并输出.pdf>"`
* **清单文件多图打包（按文本清单文件顺序逐行打包）**
  `E:\.Cli\pdf_cli.exe images-to-pdf --list "<图片清单.txt>" --output "<合并输出.pdf>"`

### 6. 视觉切片、图像解包与 OCR 前置渲染
* **批量解包导出内嵌位图（输出 PNG/JPEG）**
  `E:\.Cli\pdf_cli.exe extract-images --file "<文件路径>" --out-dir "<输出目录>"`
* **单页高清光栅化渲染（导出 300DPI PNG 图像，支撑 OCR/视觉模型）**
  `E:\.Cli\pdf_cli.exe render-page --file "<文件路径>" --page <页码> --dpi 300 --out "<输出图片路径.png>"`

### 7. 文档物理装配与几何手术刀
* **文件夹一键全量合并（推荐，自动按文件名正序扫描目录下所有 PDF）**
  `E:\.Cli\pdf_cli.exe merge --dir "<输入文件夹路径>" --output "<合并输出.pdf>"`
* **清单文件合并（跨目录组合特定少数文件）**
  `E:\.Cli\pdf_cli.exe merge --list "<文件清单.txt>" --output "<合并输出.pdf>"`
* **指定离散文件合并**
  `E:\.Cli\pdf_cli.exe merge --files "<文件1.pdf>,<文件2.pdf>,<文件3.pdf>" --output "<合并输出.pdf>"`
* **物理抽取独立页面**
  `E:\.Cli\pdf_cli.exe extract-pages --file "<文件路径>" --pages "1:5" --output "<截取输出.pdf>"`
* **页面旋转校正（支持单文件或 --dir 批量就地旋转，90 度倍数）**
  `E:\.Cli\pdf_cli.exe rotate --dir "<输入文件夹路径>" --angle 90`
  `E:\.Cli\pdf_cli.exe rotate --file "<文件路径>" --pages "1:3" --angle 90 [--output "<另存路径.pdf>"]`
* **半透明文本水印印章注入（支持单文件或 --dir 批量注入，中文不乱码）**
  `E:\.Cli\pdf_cli.exe watermark --dir "<输入文件夹路径>" --text "<水印文本>"`
  `E:\.Cli\pdf_cli.exe watermark --file "<文件路径>" --text "<水印文本>" --opacity 0.2 --font-size 42.0 --rotation 45.0 [--output "<另存路径.pdf>"]`