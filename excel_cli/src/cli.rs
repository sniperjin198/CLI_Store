use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "excel_cli")]
#[command(about = "高性能 CLI Excel 工具箱", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 检查并提取工作簿元数据
    Inspect(InspectArgs),
    /// 全文检索关键词
    Search(SearchArgs),
    /// 范围读取或预览数据
    Read(ReadArgs),
    /// 将工作表导出为 UTF-8 CSV
    Convert(ConvertArgs),
    /// 创建新工作簿
    Create(CreateArgs),
    /// 向单元格写入数值、文本或公式
    Cell(CellArgs),
    /// 追加一行数据
    Append(AppendArgs),
    /// 工作表管理 (add / remove / rename)
    Sheet(SheetArgs),
    /// 设置单元格或区域样式 (字体、字号、颜色、对齐、边框)
    Style(StyleArgs),
    /// 获取指定单元格或区域的样式属性（不含数据内容）
    StyleGet(StyleGetArgs),
    /// 调整行高与列宽
    Dimension(DimensionArgs),
    /// 合并单元格
    Merge(MergeArgs),
   /// 插入或删除行/列 (insert-row / delete-row / insert-col / delete-col)
    Grid(GridArgs),
    /// 自动调整行高与列宽 (智能自适应文本长度)
    Autofit(AutofitArgs),
}

#[derive(Args)]
pub struct InspectArgs {
    #[arg(short, long)]
    pub file: String,
	#[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct SearchArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub keyword: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
    #[arg(short, long)]
    pub limit: Option<usize>,
}

#[derive(Args)]
pub struct ReadArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
    #[arg(short, long)]
    pub range: Option<String>,
    #[arg(long)]
    pub head: Option<usize>,
}

#[derive(Args)]
pub struct ConvertArgs {
    #[arg(short, long)]
    pub input: String,
    #[arg(short, long)]
    pub output: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct CellArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub coord: String,
    #[arg(short, long)]
    pub value: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct AppendArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
    #[arg(short, long, num_args = 1..)]
    pub values: Vec<String>,
}

#[derive(Args)]
pub struct SheetArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub action: String,
    #[arg(short, long)]
    pub sheet: String,
    #[arg(long)]
    pub new_name: Option<String>,
}

#[derive(Args)]
pub struct StyleArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub coord: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
    #[arg(long)]
    pub bold: Option<bool>,
    #[arg(long)]
    pub font_size: Option<f64>,
    #[arg(long)]
    pub font_name: Option<String>, 
    #[arg(long)]
    pub font_color: Option<String>,
    #[arg(long)]
    pub bg_color: Option<String>,
    #[arg(long)]
    pub align_h: Option<String>,
    #[arg(long)]
    pub align_v: Option<String>,
    #[arg(long)]
    pub border: Option<bool>,
}

#[derive(Args)]
pub struct StyleGetArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub coord: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct DimensionArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
    #[arg(long)]
    pub col: Option<String>,
    #[arg(long)]
    pub width: Option<f64>,
    #[arg(long)]
    pub row: Option<u32>,
    #[arg(long)]
    pub height: Option<f64>,
}

#[derive(Args)]
pub struct MergeArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub range: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct GridArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub action: String, 
    #[arg(short, long)]
    pub target: String, 
    #[arg(short, long, default_value_t = 1)]
    pub count: u32,
    #[arg(short, long)]
    pub sheet: Option<String>,
}

#[derive(Args)]
pub struct AutofitArgs {
    #[arg(short, long)]
    pub file: String,
    #[arg(short, long)]
    pub sheet: Option<String>,
    #[arg(long, default_value_t = true)]
    pub col: bool,
    #[arg(long, default_value_t = false)]
    pub row: bool,
    #[arg(long)]
    pub min_width: Option<f64>,
    #[arg(long)]
    pub max_width: Option<f64>,
}