---
name: docx
description: "Use this skill for any Word document processing using the local CLI tool E:\\.Cli\\word_cli.exe."
license: Proprietary. LICENSE.txt has complete terms
---

# Word 操作专家技能 (docx)

本技能定义了本地高性能 Word 命令行工具 `E:\.Cli\word_cli.exe` 的调用契约与交互行为规范。

## 一、核心调用范式 (必须严格对照此格式执行)

遇到任何包含【中文】、【空格】、【盘符】的路径，统一使用单层标准双引号包裹，一步到位直接输出，绝对禁止分步重命名：

### 1. 宏观感知与骨架摸底 (Step 1 - 最常用认知入口)
处理任何未知 docx 或回答内容前，必须首先自省摸清大纲结构、总段落数与版面：
```bash
E:\.Cli\word_cli.exe inspect --file "E:/GooseProjects/Office/实施方案.docx"
E:\.Cli\word_cli.exe inspect --file "E:/GooseProjects/Office/实施方案.docx" --only outline
```

### 2. 文本定向精读与章节下钻 (Step 2 - 核心阅读能力)
根据自省获得的大纲标题定向精读，或提取特定段落区间：
```bash
E:\.Cli\word_cli.exe read --file "E:/GooseProjects/Office/实施方案.docx" --heading "工程概况"
E:\.Cli\word_cli.exe read --file "E:/GooseProjects/Office/实施方案.docx" --range "0:15"
```

### 3. Markdown / TXT 一键转换交付 (Step 3 - 报告成型)
```bash
E:\.Cli\word_cli.exe import --input "E:/GooseProjects/Office/方案初稿.md" --output "E:/GooseProjects/Office/方案初稿.docx" [--template "E:/GooseProjects/Office/模板.docx"]
```

### 4. 正文全局批量排版与格式刷平 (批量成型法则)
单次调用即可完成正文宋体、小四（12pt）、首行缩进 2 字符、1.5 倍行距排版，工具自动保护大纲标题不被误伤：
```bash
E:\.Cli\word_cli.exe para-format --file "E:/GooseProjects/Office/实施方案.docx" --range all --target body --font "宋体" --size 12.0 --indent 2.0 --spacing 1.5 --strip-formatting
```

### 5. 跨 Run 占位符与键值批量替换
```bash
E:\.Cli\word_cli.exe replace --file "E:/GooseProjects/Office/实施方案.docx" --pairs "{{PROJECT}}=铜川牡丹园,{{DATE}}=2026-09-16"
```

### 6. 表格矩阵直读与改写
```bash
# Markdown 矩阵直读（零解析损耗，大模型最易理解）
E:\.Cli\word_cli.exe read-table --file "E:/GooseProjects/Office/实施方案.docx" --index 0 --format markdown

# 单元格精准修改
E:\.Cli\word_cli.exe set-cell --file "E:/GooseProjects/Office/实施方案.docx" --index 0 --row 1 --col 2 --val "50000.00"
```

### 7. 多文档追加合并 (Merge)
```bash
# 目录扫描全量追加合并（推荐，自动按自然顺序把目录下所有 docx 追加至主文档）
E:\.Cli\word_cli.exe merge --file "E:/GooseProjects/Office/主报告.docx" --dir "E:/GooseProjects/Office/各分部附录"

# 指定追加特定文档
E:\.Cli\word_cli.exe merge --file "E:/GooseProjects/Office/主报告.docx" --append "E:/GooseProjects/Office/附录.docx"
```

---

## 二、核心执行守则 (硬性约束)

1. **唯一合法工具绝对路径 (注意点号)**：  
   * 必须且仅允许调用带点号的绝对路径可执行文件：`E:\.Cli\word_cli.exe`。  
   * **绝对禁止拼写为 `E:\Cli\word_cli.exe`（缺失点号会导致系统报“找不到指定的路径”）**。  
   * 严禁编写 Python 脚本（如 `python-docx`）、PowerShell、VBScript 或调用 Office COM 组件操作文档。
2. **正斜杠与完整盘符法则**：  
   * 传递给 CLI 的所有路径，**必须统一使用正斜杠 `/` 且包含盘符**（如 `--file "E:/GooseProjects/Office/report.docx"`）。  
   * **绝对禁止省略盘符写成 `/GooseProjects/...`**（省略盘符在 Windows CMD/Shell 跨驱动器或子终端环境下会导致严重路径解析异常与挂起）。  
   * **严禁先输出为临时文件名再调用 move 移动**，`--output` 原生支持直接指定最终中文目标文件名。
3. **禁止任何冗余探测与校验读回**：  
   * CLI 每次执行成功均会向 stdout 返回单行标准 JSON，格式为 `{"status":"success", ...}`。  
   * **执行写操作后，严禁使用 shell 命令（如 type、cat、dir、Get-Content、Get File Info）去解包或检查文件，严禁通过循环 inspect 校验更改**。看到 JSON 中的 `status: "success"` 即代表物理写入成功，直接进入下一步或向用户汇报。
4. **正文排版严禁逐段循环 (批量成型法则)**：  
   * 遇到全文统一排版（如宋体、小四、首行缩进 2 字符、1.5 倍行距）时，**绝对禁止**按段落索引从 0 到 N 逐段循环调用 `para-format`！  
   * 必须单次使用批量参数 `--range all --target body` 一步成型，工具会自动保护并跳过各级大纲标题。
5. **三步人机协同认知闭环 (Three-Step Cognitive Flow)**：  
   * **宏观建构 (Step 1)**：处理复杂长文档时，首先调用 `inspect` 秒级摸清骨架（总段落数、纸张方向与规格、1~6 级大纲树、表格与高亮分布），严禁盲目全量精读正文。  
   * **章节下钻 (Step 2)**：通过 `read --heading "章节名"` 定向调取章节内容提炼汇报。  
   * **微观精读与修改 (Step 3)**：通过 `read --range A:B` 精读指定段落讨论，再通过对应指令精准落盘。
6. **Shell 参数包裹与开关规范**：  
   * 遇到空格或中文参数，在参数外侧加单层标准双引号：`--file "E:/GooseProjects/Office/方案.docx"`。**严禁出现 `\"` 或 `""` 嵌套转义**。  
   * 布尔类型参数（如 `--strip-formatting`、`--page-num`、`--unlink`）均为**开关型标志（Flag）**，出现即生效，严禁后接 `true/false`（严禁出现 `--strip-formatting true`）。

---

## 三、完整命令语法速查手册

### 1. 文档生命周期与格式转换

* **从零新建极简合规标准空白文档（或基于母版物理克隆）**  
  `E:\.Cli\word_cli.exe create --file "<文件路径>" [--template "<母版路径>"]`

* **复制另存为新文档**  
  `E:\.Cli\word_cli.exe save-as --file "<源文件路径>" --output "<新文件路径>"`

* **Markdown / TXT 转换为交付级 Word 文档（自动继承母版公文样式、加粗、代码块与表格）**  
  `E:\.Cli\word_cli.exe import --input "<输入.md>" --output "<输出.docx>" [--template "<母版路径>"]`

---

### 2. 认知导航与精读下钻

* **全局宏观自省驾驶舱（返回纸张方向与规格、大纲树、表格概览、图片清单与高亮分布）**  
  `E:\.Cli\word_cli.exe inspect --file "<文件路径>"`

* **自省驾驶舱轻量切片过滤（降低 Token 消耗）**  
  `E:\.Cli\word_cli.exe inspect --file "<文件路径>" --only outline|tables|images|highlights|summary`

* **按标题定向调取章节正文**  
  `E:\.Cli\word_cli.exe read --file "<文件路径>" --heading "<章节名称>"`

* **按索引区间精读指定段落**  
  `E:\.Cli\word_cli.exe read --file "<文件路径>" --range "5:10"`

* **查询文档样式库已定义样式与 styleId 对照清单**  
  `E:\.Cli\word_cli.exe styles --file "<文件路径>"`

---

### 3. 段落编辑、占位符替换与批量格式刷平

* **正文全局批量排版与格式刷平（推荐：单次调用，自动保护大纲标题不被误伤）**  
  `E:\.Cli\word_cli.exe para-format --file "<文件路径>" --range all --target body --font "宋体" --size 12.0 --indent 2.0 --spacing 1.5 [--strip-formatting]`

* **指定段落排版微调（单段精准调整）**  
  `E:\.Cli\word_cli.exe para-format --file "<文件路径>" --index 2 --font "黑体" --size 18.0 --bold true --indent 2.0 --spacing 1.5`

* **跨 Run 占位符安全替换（单项替换）**  
  `E:\.Cli\word_cli.exe replace --file "<文件路径>" --from "{{PROJECT_NAME}}" --to "2026工程实施方案"`

* **跨 Run 占位符批量注入（多键值对映射）**  
  `E:\.Cli\word_cli.exe replace --file "<文件路径>" --pairs "{{NAME}}=张三,{{DATE}}=2026-09-01"`

* **关键词突出高亮标注（支持 yellow, green, cyan 等；设为 none 可一键清除高亮）**  
  `E:\.Cli\word_cli.exe highlight --file "<文件路径>" --text "<关键词>" [--color yellow|green|none]`

* **在指定段落前插入新段落**  
  `E:\.Cli\word_cli.exe insert-para --file "<文件路径>" --index 2 --text "<新内容>" [--style "Heading 2"]`

* **在文档末尾追加新段落**  
  `E:\.Cli\word_cli.exe append-para --file "<文件路径>" --text "<追加文本>" [--style "Normal"]`

* **删除指定索引段落**  
  `E:\.Cli\word_cli.exe delete-para --file "<文件路径>" --index 3`

* **套用模板内预设样式**  
  `E:\.Cli\word_cli.exe apply-style --file "<文件路径>" --index 1 --style "Heading 1"`

* **多文档全量追加合并（支持目录扫描 --dir 或单文件 --append）**  
  `E:\.Cli\word_cli.exe merge --file "<母文档路径>" --dir "<待追加文档目录>"`  
  `E:\.Cli\word_cli.exe merge --file "<母文档路径>" --append "<副文档路径>"`

---

### 4. 表格矩阵读写与行列动态伸缩

* **大模型直读模式（以 Markdown 原生表格直读，零解析损耗）**  
  `E:\.Cli\word_cli.exe read-table --file "<文件路径>" --index 0 --format markdown`

* **表格结构化读取（支持 JSON 二维矩阵或 CSV 导出）**  
  `E:\.Cli\word_cli.exe read-table --file "<文件路径>" --index 0 --format json|csv`

* **精准改写指定单元格内容**  
  `E:\.Cli\word_cli.exe set-cell --file "<文件路径>" --index 0 --row 1 --col 2 --val "<新数值>"`

* **在指定行前插入新数据行（逗号分隔单元格）**  
  `E:\.Cli\word_cli.exe insert-row --file "<文件路径>" --index 0 --row 2 --cells "项次,指标,说明"`

* **删除表格指定行**  
  `E:\.Cli\word_cli.exe delete-row --file "<文件路径>" --index 0 --row 3`

---

### 5. 页面版式、下一页分节、硬分页与页眉页脚

* **在文档末尾插入独立下一页分节符（实现新章节独立排版/横向混排）**  
  `E:\.Cli\word_cli.exe insert-section --file "<文件路径>"`

* **插入硬分页符（末尾分页：不传 index 或传 end；段落前分页：传段落序号）**  
  `E:\.Cli\word_cli.exe insert-page-break --file "<文件路径>" [--index end|5]`

* **页面规格与横纵方向切换（解决大表格横向混排需求）**  
  `E:\.Cli\word_cli.exe page --file "<文件路径>" --orient landscape|portrait [--size A4|A3]`

* **配置文档页眉（支持 --unlink 解除节继承绑定）**  
  `E:\.Cli\word_cli.exe set-header --file "<文件路径>" --text "【机密】工程技术实施方案" [--unlink]`

* **配置文档页脚与动态页码（支持从 1 重新起算，--unlink 实现首页无页码）**  
  `E:\.Cli\word_cli.exe set-footer --file "<文件路径>" --text "内部报告 · 第" --page-num --restart 1 [--unlink]`

---

### 6. 嵌入图片排版与素材维护

* **在指定段落前插入图片（指定毫米尺寸与自动 DrawingML 计算）**  
  `E:\.Cli\word_cli.exe insert-image --file "<文件路径>" --index 2 --image "<图片路径.png>" --width 120.0 --height 80.0`

* **原地二进制无损替换图片素材（支持按 rId 或文件名匹配）**  
  `E:\.Cli\word_cli.exe replace-image --file "<文件路径>" --rid "rId100" --image "<新图片.png>"`

* **批量解包导出文档中所有嵌入图片**  
  `E:\.Cli\word_cli.exe extract-images --file "<文件路径>" --out-dir "<输出目录>"`