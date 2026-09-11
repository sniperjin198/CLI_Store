# word-cli

> Windows 原生高性能 Word 命令行处理工具

## 📋 **核心定位**
服务于本地大模型 (Goose/Qwen) 与自动化工作流的文档自省建构、大纲感知、精读讨论、公文报告生成等场景。

---

## 🏗️ **架构哲学**

> **三步认知闭环** → 宏观 `inspect` → 章节 `read` → 微观 `replace/para/table`

### 🎯 **设计原则**
- ✅ JSON-First 输出契约，统一单行 JSON 格式
- ✅ 短扁平动词命名 (`inspect/read/replace`)，防止连字符截断风险
- ✅ 原子写入机制：临时文件 + rename 保证字节级保真
- ✅ 内置 8KB 极简 docx 字节数组作为起点

---

## 📦 **技术栈**
| 组件 | 说明 |
|------|------|
| 🔧 语言 | Rust |
| 📖 解析器 | pulldown-cmark |
| 📄 OpenXML | ZIP 包原子写入机制 |

---

## 🚀 **子命令概览 (15+)**
```
create    : 创建文档 (内嵌模板/母版克隆)
inspect   : 检查/读取文档 (单次自省驾驶舱)
import    : 导入/合并文档
replace   : 替换内容
styles    : 样式操作 (字体、颜色、段落)
para      : 段落操作
table     : 表格操作
image     : 图片排版分节
highlight : 高亮标记
break     : 分页控制
page      : 页面管理
header    : 页眉设置
footer    : 页脚设置
merge     : 文档合并
```

---

## 📍 **实施阶段**
| Phase | 任务 |
|-------|------|
| ✅ Phase 1 | 脚手架 + 核心依赖配置 (进行中) |
| ⏳ Phase 2 | 精读替换功能 |
| ⏳ Phase 3 | 表格操作 |
| ⏳ Phase 4 | 图片排版分节 |

---

## 📦 **快速开始**

### 1. 安装依赖
```bash
cd /d E:\GooseCliBuilder\word_cli && cargo build --release
```

### 2. 使用示例
```powershell
cd /d E:\GooseCliBuilder\word_cli
.	arget\release\word_cli.exe inspect "C:\path\to\document.docx"
```

---

## 📜 **输出协议规范**
- ✅ stdout 仅输出单行 JSON，严禁调试日志
- ✅ 错误采用语义化英文字符串 (如 `CorruptedDocument`)
- ✅ 禁止十六进制错误码 (如 `0x8000_2001`)

---

## 📄 **License**
MIT License © AAIF (Agentic AI Foundation)
