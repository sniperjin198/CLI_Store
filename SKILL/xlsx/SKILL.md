---
name: xls
description: "Use this skill for any spreadsheet using the local CLI tool E:\\.Cli\\excel_cli.exe."
license: Proprietary. LICENSE.txt has complete terms
---

# **Excel 操作专家技能 (xls)**

本技能定义了本地高性能 Excel 命令行工具 E:\\.Cli\\excel\_cli.exe 的调用契约与交互行为规范。

当遇到任何 .xlsx 文件的创建、结构修改、数据读写、行列调整、样式美化与格式导出任务时，必须严格执行本规范。

## **核心执行守则 (硬性约束)**

1. **唯一合法工具路径**：  
   * 必须且仅允许调用绝对路径可执行文件：E:\\.Cli\\excel\_cli.exe。  
   * 严禁使用 Python 脚本（如 openpyxl、pandas）、Node.js 脚本或临时代码操控 Excel。  
2. **禁止任何冗余探测与读回**：  
   * CLI 每次执行成功均会向 stdout 返回单行标准 JSON，格式为 {"status":"success", "command":"...", ...}。  
   * **执行写操作后，严禁使用 shell 命令（如 type、cat、Get-Content）或任何文件读取插件去读取或检查文件**，直接信任 CLI 返回的成功状态即可。  
3. **Shell 调用规范**：  
   * 推荐在系统 Shell 工具中直接单行调用绝对路径命令。  
   * 遇到跨盘符或含空格路径，使用批处理调用：  
     cmd.exe /c "E:\\.Cli\\excel\_cli.exe \<COMMAND\> \[OPTIONS\]"  
4. **参数规范与格式契约**：  
   * **单元格坐标**：大写列标 \+ 行号（如 A1、B2:B5）。  
   * **Hex 颜色格式**：标准 6 位十六进制色值（例如：4F81BD 工程蓝、FFFFFF 纯白、F2F2F2 浅灰，前缀 \# 可选）。  
   * **公式输入**：必须以 \= 符号开头，例如 \--value "=SUM(B2:B5)"。
5. **严禁创建中间批处理脚本：严禁编写 .bat、.cmd、.ps1 或 .py 文件来包装执行命令。面对多步操作，必须使用系统 shell 工具单行、逐条依次调用。**
6. **严禁臆造语法与参数：严禁凭经验猜测参数（例如不存在 update、--worksheet 等）。在构造命令前若不确定参数名，必须先在 shell 中执行 E:\.Cli\excel_cli.exe <COMMAND> --help 查阅，严格按输出的 Usage 传参。**

---

## **完整命令语法速查手册**

### **1. 数据检查与只读检索**

- **查看工作簿概况（包含各 Sheet 名称、行数与列数）  
E:\\.Cli\\excel\_cli.exe inspect \--file "\<文件路径\>"

- **全文关键词检索（返回匹配行号、列号及整行明细）  
E:\\.Cli\\excel\_cli.exe search \--file "\<文件路径\>" \--keyword "\<关键词\>" \[--sheet "\<工作表名\>"\] \[--limit 10\]

- **范围读取或快速预览（返回标准化二维纯文本矩阵）  
E:\\.Cli\\excel\_cli.exe read \--file "\<文件路径\>" \[--sheet "\<工作表名\>"\] \[--range "A1:D10"\] \[--head 5\]

- **导出指定工作表为带 BOM 的标准 UTF-8 CSV（Excel 双击打开无乱码）  
E:\\.Cli\\excel\_cli.exe convert \--input "\<文件路径\>" \--output "\<目标.csv\>" \[--sheet "\<工作表名\>"\]

---

### **2. 工作簿生命周期与数据写入**

- **从零新建空白工作簿  
E:\\.Cli\\excel\_cli.exe create \--file "\<文件路径\>" \[--sheet "\<初始工作表名\>"\]

- **单单元格精准设值（自动识别文本、整数、浮点数、布尔与公式）  
E:\\.Cli\\excel\_cli.exe cell \--file "\<文件路径\>" \--coord "\<A1\>" \--value "\<写入内容\>" \[--sheet "\<工作表名\>"\]

- **向表格末尾追加整行数据（参数末尾支持传入多个值）  
E:\\.Cli\\excel\_cli.exe append \--file "\<文件路径\>" \[--sheet "\<工作表名\>"\] \--values "第一列数据" "第二列数据" "第三列数据"

---

### **3. 工作表 (Sheet) 管理**

- **新建工作表  
E:\\.Cli\\excel\_cli.exe sheet \--file "\<文件路径\>" \--action add \--sheet "\<新工作表名\>"

- **删除工作表（保护机制：若仅剩一张表则自动拦截报错）  
E:\\.Cli\\excel\_cli.exe sheet \--file "\<文件路径\>" \--action remove \--sheet "\<待删工作表名\>"

- **重命名工作表  
E:\\.Cli\\excel\_cli.exe sheet \--file "\<文件路径\>" \--action rename \--sheet "\<原工作表名\>" \--new-name "\<新工作表名\>"

---

### **4. 矩阵结构调整 (行列插入与删除)**

- **插入行（在第 N 行前插入指定行数）  
E:\\.Cli\\excel\_cli.exe grid \--file "\<文件路径\>" \--action insert-row \--target 2 \--count 1 \[--sheet "\<工作表名\>"\]

:: 删除行（从第 N 行开始删除指定行数）  
E:\\.Cli\\excel\_cli.exe grid \--file "\<文件路径\>" \--action delete-row \--target 3 \--count 2 \[--sheet "\<工作表名\>"\]

:: 插入列（在指定列前插入指定列数，target 支持列字母如 B）  
E:\\.Cli\\excel\_cli.exe grid \--file "\<文件路径\>" \--action insert-col \--target B \--count 1 \[--sheet "\<工作表名\>"\]

:: 删除列（从指定列开始删除指定列数）  
E:\\.Cli\\excel\_cli.exe grid \--file "\<文件路径\>" \--action delete-col \--target C \--count 1 \[--sheet "\<工作表名\>"\]

---

### **5. 样式美化、尺寸与排版**

- **区域样式设置（加粗、字号、文字颜色、背景填充、水平/垂直对齐、细边框）  
E:\\.Cli\\excel\_cli.exe style \--file "\<文件路径\>" \--coord "A1:E1" \--bold true \--font-size 11 \--font-color "FFFFFF" \--bg-color "4F81BD" \--align-h "center" \--align-v "center" \--border true \[--sheet "\<工作表名\>"\]

- **设置行高与列宽（防止文字超出或显示为 \#\#\#）  
E:\\.Cli\\excel\_cli.exe dimension \--file "\<文件路径\>" \[--sheet "\<工作表名\>"\] \[--col "A" \--width 20\] \[--row 1 \--height 28\]

- **单元格区域合并（常用于报表大标题跨列居中）  
E:\\.Cli\\excel\_cli.exe merge \--file "\<文件路径\>" \--range "A1:D1" \[--sheet "\<工作表名\>"\]  

- **If size > 0**, the spreadsheet generation is verified and successful.