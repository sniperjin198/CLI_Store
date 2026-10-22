---
name: webscrape
description: "Use this skill for web browsing, live text summarization, government notification monitoring, on-demand archiving (HTML/Markdown), high-speed attachment downloading (Excel/PDF/ZIP via aria2c), and media acquisition (Video/Audio via yt-dlp) using the local Super-CLI E:\\.Cli\\webscrape.exe."
license: Proprietary. LICENSE.txt has complete terms
---

# 网页感知、信息萃取与全能调度专家技能 (webscrape v3.0)

本技能严格定义本地高性能超级门面命令行工具 `E:\.Cli\webscrape.exe` 的调用契约与交互行为准则。

---

## 一、三大严厉禁令（违者视为严重事故）

1. 🚫 **绝对禁止伪造与手写文件（严禁调用 Write File / write 工具）**：
   * **严禁调用 `Write File`、`Create File`、`write`、PowerShell 或任何写文件工具来“生成/拼凑”网页、音视频或附件内容**！
   * **严禁使用大模型自身的理解、记忆或总结来伪造 HTML、Markdown、TXT、XLSX 或 MP4 文件**！
   * 本地 CLI 原生具备高度工业化的存盘与专用引擎调度机制（`save` 命令）。所有物理文件必须且只能由 `webscrape.exe save` 命令直接落盘。工具成功则汇报成功，工具失败必须立即如实汇报，绝对禁止私自兜底。

2. 🚫 **绝对禁止调用系统级网络工具（禁止 curl / wget / python）**：
   * 严禁私自执行 `curl`、`wget`、`Invoke-WebRequest` 或编写 Python 脚本抓取网页、下载音视频或下载文件。
   * `E:\.Cli\webscrape.exe` 是唯一合法的网络请求、正文解析、流媒体提取与附件下载抓手。

3. 🚫 **禁止越过用户擅自存盘（先问后存机制）**：
   * 用户仅给出链接时，**只准阅读总结，绝对禁止写盘**！
   * 当用户说“帮我保存/下载”但未指定格式与细节时，**严禁直接执行写盘命令**！必须停下来向用户确认格式与偏好（详见“第二规程”中的确认规范）。
   * **特例**：若用户明确指明格式（如“存为 markdown 笔记带图”、“下载 1080P 视频”）或明确下载具体文件（如“下载网页里的 Excel 表格”），则无需二次询问，直接调用 `save` 物理落盘。

---

## 二、标准交互与执行规程 (四阶段交互闭环)

### 阶段 1：纯读感知与重点提炼 (`get` · 默认入口 · 零写盘)
* **适用场景**：用户发送任何网页或文章链接，询问“看看这个讲了什么”、“提炼重点”、“网页上有哪些信息”。
* **执行规范**：调用 `get` 命令（支持位置参数或 `--url` 具名参数）。工具在内存中完成正文提取与清洗并在 stdout 返回标准 JSON（内含 `content` 纯净 Markdown 正文），**绝对不写磁盘**。
* **交付输出**：在对话框中直接向用户输出清晰的结构化总结、核心论点或目标数据；汇报完毕后，如内容有留存价值，可主动提示用户：“如需本地存档，可随时告诉我”。
```bash
# 纯阅读感知调用（完全不写入磁盘）
E:\.Cli\webscrape.exe get "https://example.com/article"
# 或使用大模型常用具名参数
E:\.Cli\webscrape.exe get --url "https://example.com/article"
```

### 阶段 2：按需归档前置确认与原生落盘 (`save` · 仅在用户要求保存时触发)

#### 场景 A：文章或网页归档需求
* **触发意图**：用户明确提出“保存这篇文章”、“把这个网页下载到本地”。
* **确认规范（必须停顿询问）**：
  > “已为您准备就绪，请问您需要保存为：
  > 1. **原始网页 HTML 存档**（由内建 monolith 引擎打包为独立单文件，100% 还原样式与排版）
  > 2. **图文 Markdown 笔记**（提取提纯正文，自动将插图下载至本地并改写相对路径）？”
* **执行落盘**：
```bash
# 1. 完整原始单文件 HTML 归档 (底层自动调度 monolith)
E:\.Cli\webscrape.exe save "目标URL" --format html -o "E:/GooseProjects/WebArchives/文件名.html"

# 2. 完整图文 Markdown 笔记 (底层自动清洗正文并下载图片至本地同级 _images 目录)
E:\.Cli\webscrape.exe save "目标URL" --format md --with-images -o "E:/GooseProjects/Doc/文件名.md"
```

#### 场景 B：音视频流媒体下载需求 (B站 / YouTube / 视频外链)
* **触发意图**：用户给出视频链接并要求“下载这个视频/音频”。
* **确认规范（必须停顿询问）**：
  > “已识别到音视频资源，请问您需要：
  > 1. **默认最佳画质视频（MP4）**
  > 2. **仅提取纯音频（MP3，适合听播客/配乐）**
  > 3. **限制分辨率（如 1080P 或 720P，节省磁盘空间）**？”
* **执行落盘 (底层自动调度 yt-dlp)**：
```bash
# 1. 最佳画质视频保存
E:\.Cli\webscrape.exe save "视频URL" --format video -o "E:/GooseProjects/Videos/视频名.mp4"

# 2. 提取纯音频 (MP3)
E:\.Cli\webscrape.exe save "视频URL" --format audio -o "E:/GooseProjects/Audio/音频名.mp3"

# 3. 指定最大分辨率下载 (例如 1080P)
E:\.Cli\webscrape.exe save "视频URL" --resolution 1080 -o "E:/GooseProjects/Videos/视频名.mp4"
```

### 阶段 3：二进制资源与附件极速直流落盘 (`save` · 委托 aria2c 多线程)
* **适用场景**：用户需要下载网页中指向的 `.xlsx`、`.xls`、`.pdf`、`.docx`、`.zip`、`.7z` 等文件。
* **执行规范**：直接调用 `save` 命令并指定目标后缀路径。CLI 底层会自动调度专用多线程引擎（`aria2c`），以 4 连接并发、浏览器 UA 伪装执行无损直流下载，杜绝字符转码破坏。
```bash
# 原生多线程极速下载 Excel、PDF 等附件文件（自动路由至 aria2c）
E:\.Cli\webscrape.exe save "附件URL" -o "E:/GooseProjects/Office/文件名.xlsx"
```

### 阶段 4：资讯流差量比对与表格直取 (`links` / `extract` · 默认零落盘)
* **适用场景**：政企公示通告、要闻列表巡检、网页表格数据提取。
* **执行规范**：直接抓取结构化 JSON 数据，在对话框内以表格/列表呈现：
```bash
# 提取前 N 条有效资讯（链接绝对路径 + 伴随发布日期，控制台直出）
E:\.Cli\webscrape.exe links "目标URL" --limit 10

# 提取页面 <table> 表格并以二维数据直读
E:\.Cli\webscrape.exe extract "目标URL" --limit 1

# 基于 CSS 选择器提取商品/模型卡片
E:\.Cli\webscrape.exe extract "目标URL" --selector "tr.athing" --limit 10
```

### 阶段 5：工具报错即停法则 (拒绝脑补)
* 若 `webscrape.exe` 返回状态为 `error`（如 HTTP 403 触发反爬、HTTP 404 文件不存在、组件缺失等）：
  * **必须立即停止后续操作**，原样向用户汇报真实错误（例如：“下载失败：目标服务器响应 404 Not Found”或“缺少核心下载引擎组件，请检查 E:\.Cli\bin\ 目录”）；
  * **严禁自行尝试使用 Write File 伪造内容或生成空壳文件**。

---

## 三、核心硬性约束

1. **唯一合法绝对路径 (必须带点号)**：  
   * 必须且仅允许调用带点号的全局路径：`E:\.Cli\webscrape.exe`。  
   * **绝对禁止拼写为 `E:\Cli\webscrape.exe`（漏掉点号会导致系统报“找不到指定的路径”）**。

2. **命令职责分离原则（黑白分明）**：  
   * **仅阅读/总结/提炼**：必须使用 `get`（零写盘，直接解析 stdout）。  
   * **任何文件保存/下载**：必须使用 `save`（强制包含合法 `-o` 路径）。

3. **URL 传参及引号包裹准则 (微信长链与 URL 特殊字符防御)**：  
   * 必须使用标准单层双引号完整包裹 URL，**绝对禁止裸露传入导致控制台中的 `&` 符号截断命令**。  
   * 遇到带有长串追踪参数的微信长链（包含 `scene=1`, `srcid=...`, `sharer_shareinfo=...`, `#rd`），大模型必须主动蒸馏清洗：
     1. 丢弃末尾的 `#rd` 锚点；
     2. 丢弃所有营销与追踪埋点参数，仅保留 `__biz`, `mid`, `idx`, `sn` 四大核心参数（或直接使用 `/s/xxxx` 短链）。

4. **网络自治与代理自主调度准则 (Agent Proxy Rule)**：  
   * **境内站点**（如 `.cn`、`.gov.cn`、微信公众号等）：**严禁追加 `--proxy`**，采用默认直连以确保毫秒级高速响应。  
   * **境外受限站点**（如 GitHub、国外模型站、YouTube、Hacker News 等）：由大模型**自主决策追加 `--proxy "http://127.0.0.1:7890"`**（或本地指定代理端口）。  
   * 遇到慢速站点，可主动追加 `--timeout 60` 防止连接提早中断。

5. **正斜杠与路径规范 (落盘时生效)**：  
   * 传递给 CLI 的所有本地输出路径，必须统一使用正斜杠 `/` 且包含盘符（如 `-o "E:/GooseProjects/Doc/文章.md"`）。  
   * CLI 原生支持直接指定最终中文目标文件名及目标目录，工具会自动递归创建父级目录，**严禁先输出为临时文件名再调用 move 重命名**。

6. **禁止冗余探测与读回**：  
   * CLI 每次执行均会向 stdout 返回标准 JSON，包含 `{"status":"success", ...}`。  
   * 执行写操作后，**严禁使用 shell 命令（如 `dir`、`ls`、`Get-Content`、`Get File Info`）二次探测文件是否存在**。看到 JSON 中的 `status: "success"` 即代表物理操作已完成，直接向用户汇报。

---

## 四、完整命令语法速查手册

### 1. 网页文章纯读浏览 (`get`)

* **纯读正文（不落盘，大模型首选感知入口）**  
  `E:\.Cli\webscrape.exe get "<目标URL>"`

* **使用具名参数模式纯读**  
  `E:\.Cli\webscrape.exe get --url "<目标URL>"`

* **外网文章自主挂载代理浏览**  
  `E:\.Cli\webscrape.exe get "<外网URL>" --proxy "http://127.0.0.1:7890"`

* **微信长链接清洗抓取范例**  
  `E:\.Cli\webscrape.exe get "https://mp.weixin.qq.com/s?__biz=MzcwMDM4NDc1Mw==&mid=2247484466&idx=1&sn=f61646ff500382ae5052dc4201a7bd11"`

---

### 2. 网页、附件与多媒体物理存盘 (`save`)

* **图文 Markdown 深度归档（自动下载插图并改写为相对路径）**  
  `E:\.Cli\webscrape.exe save "<目标URL>" --format md --with-images -o "<输出路径.md>"`

* **完整原始单文件 HTML 归档（自动调度 monolith）**  
  `E:\.Cli\webscrape.exe save "<目标URL>" --format html -o "<输出路径.html>"`

* **原生下载 Excel / PDF / ZIP 等二进制附件（自动调度 aria2c 多线程）**  
  `E:\.Cli\webscrape.exe save "<附件URL>" -o "<输出路径.xlsx>"`

* **下载最佳画质视频（自动调度 yt-dlp）**  
  `E:\.Cli\webscrape.exe save "<视频URL>" --format video -o "<输出路径.mp4>"`

* **提取视频中的纯音频文件 (MP3)**  
  `E:\.Cli\webscrape.exe save "<视频URL>" --format audio -o "<输出路径.mp3>"`

* **限制最大分辨率下载视频 (例如 1080P)**  
  `E:\.Cli\webscrape.exe save "<视频URL>" --resolution 1080 -o "<输出路径.mp4>"`

* **走代理保存境外资源**  
  `E:\.Cli\webscrape.exe save "<境外URL>" --proxy "http://127.0.0.1:7890" -o "<输出路径>"`

---

### 3. 超链接与要闻公告监控采集 (`links`)

* **页面有效链接扫描（自动绝对路径补全 + 发布日期识别）**  
  `E:\.Cli\webscrape.exe links "<目标URL>" --limit 10`

* **将差量数据存盘为 JSON 文件**  
  `E:\.Cli\webscrape.exe links "<目标URL>" --limit 10 -o "<输出路径.json>"`

---

### 4. 结构化卡片与表格数据抽取 (`extract`)

* **自动识别并提取网页 `<table>` 表格**  
  `E:\.Cli\webscrape.exe extract "<目标URL>" --limit 1`

* **基于 CSS 选择器抽取商品/模型卡片并落盘**  
  `E:\.Cli\webscrape.exe extract "<目标URL>" --selector "<CSS选择器>" --limit 10 -o "<输出路径.json>"`