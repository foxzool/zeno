use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use zeno_core::{MarkdownParser, LocalFileStorage, FileStorage};

#[derive(Parser)]
#[command(name = "zeno")]
#[command(about = "Zeno - 个人知识管理与发布平台", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 解析 Markdown 文件
    Parse {
        /// 文件路径
        #[arg(short, long)]
        file: PathBuf,
        /// 输出格式 (json|yaml)
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// 列出指定目录下的所有 Markdown 文件
    List {
        /// 目录路径
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// 初始化新的知识库
    Init {
        /// 知识库路径
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    /// 显示版本信息
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Parse { file, format } => {
            parse_file(file, format).await?;
        }
        Commands::List { dir } => {
            list_files(dir).await?;
        }
        Commands::Init { path } => {
            init_workspace(path).await?;
        }
        Commands::Version => {
            println!("zeno-cli {}", env!("CARGO_PKG_VERSION"));
        }
    }
    
    Ok(())
}

async fn parse_file(file: PathBuf, format: String) -> Result<()> {
    let parser = MarkdownParser::new();
    let note = parser.parse_file(&file)?;
    
    match format.as_str() {
        "yaml" => {
            println!("{}", serde_yaml::to_string(&note)?);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&note)?);
        }
    }
    
    Ok(())
}

async fn list_files(dir: PathBuf) -> Result<()> {
    let storage = LocalFileStorage::new(dir);
    let files = storage.list_markdown_files(&PathBuf::from(".")).await?;
    
    if files.is_empty() {
        println!("未找到 Markdown 文件");
        return Ok(());
    }
    
    println!("找到 {} 个 Markdown 文件:", files.len());
    for file in files {
        println!("  {}", file.display());
    }
    
    Ok(())
}

async fn init_workspace(path: PathBuf) -> Result<()> {
    // 创建目录结构
    tokio::fs::create_dir_all(&path).await?;
    tokio::fs::create_dir_all(path.join("notes")).await?;
    tokio::fs::create_dir_all(path.join("assets")).await?;
    tokio::fs::create_dir_all(path.join("templates")).await?;
    
    // 创建配置文件
    let config = r#"# Zeno 知识库配置
title: "我的知识库"
description: "基于 Zeno 的个人知识管理系统"

# 目录配置
directories:
  notes: "notes"
  assets: "assets"
  templates: "templates"

# 默认设置
defaults:
  note_template: "default"
  publish: false
"#;
    
    tokio::fs::write(path.join("zeno.yml"), config).await?;
    
    // 创建示例笔记
    let sample_note = r#"---
title: "欢迎使用 Zeno"
date: 2024-06-30
tags: ["zeno", "知识管理"]
status: draft
---

# 欢迎使用 Zeno

这是你的第一个笔记！

## 功能特色

- 📝 Markdown 编辑和解析
- 🔗 双向链接支持
- 🏷️ 标签和分类管理
- 🔍 全文搜索
- 📊 知识图谱可视化
- 🚀 多平台发布

## 开始使用

1. 在 `notes/` 目录下创建新的 Markdown 文件
2. 使用前言（frontmatter）配置笔记元数据
3. 享受写作的乐趣！

---

祝你使用愉快！ 🎉
"#;
    
    tokio::fs::write(path.join("notes/welcome.md"), sample_note).await?;
    
    println!("✅ 知识库已初始化到: {}", path.display());
    println!("📝 已创建示例笔记: notes/welcome.md");
    println!("⚙️  配置文件: zeno.yml");
    
    Ok(())
}