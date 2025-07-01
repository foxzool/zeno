Zeno - 基于 Zettelkasten 的知识管理与发布平台
1. 项目概述
1.1 核心理念
Zeno 是一个基于 Zettelkasten 方法论的现代知识管理工具，旨在帮助用户构建、发展和分享他们的个人知识网络。

1.2 设计原则
原子化思维：每个笔记是一个独立的思想单元
网络化连接：通过双向链接构建知识网络
渐进式发展：支持思想的演化和分支
公开学习：将个人知识库转化为公共知识花园
1.3 技术特色
时间戳 ID 系统（如 202401201430）
混合发布模式（单卡片 + 主题文章）
保留完整的双向链接网络
交互式知识图谱可视化
2. Zettelkasten 数据模型
2.1 卡片类型定义
rust
// models/zettel.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Zettel {
    pub id: String,              // 时间戳ID: 202401201430
    pub title: String,           // 简短标题
    pub content: String,         // Markdown 内容
    pub zettel_type: ZettelType,
    pub tags: Vec<String>,       // 标签
    pub links: Vec<ZettelLink>,  // 链接关系
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub publish_status: PublishStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ZettelType {
    Permanent,      // 永久笔记（核心想法）
    Literature,     // 文献笔记（引用和评论）
    Structure,      // 结构笔记（组织其他笔记）
    Index,          // 索引笔记（入口点）
    Daily,          // 日常笔记（临时想法）
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZettelLink {
    pub target_id: String,
    pub link_type: LinkType,
    pub context: Option<String>,  // 链接上下文
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LinkType {
    Reference,      // 引用
    Continuation,   // 继续发展
    Contradiction,  // 反对/质疑
    Branch,         // 分支发展
    Structure,      // 结构组织
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PublishStatus {
    Private,        // 私有
    Public,         // 公开
    Draft,          // 草稿（占位符）
}
2.2 文件组织结构
zeno-zettelkasten/
├── zettel/                     # 所有卡片
│   ├── 202401201430.md        # 单个卡片文件
│   ├── 202401201445.md
│   └── ...
├── structures/                 # 结构笔记
│   ├── programming-paradigms.md
│   └── philosophy-of-mind.md
├── indices/                    # 索引系统
│   ├── main-index.md          # 主索引
│   ├── topic-indices/         # 主题索引
│   └── chronological.md       # 时间索引
├── assets/                     # 资源文件
├── .zeno/
│   ├── config.toml
│   ├── graph.db               # 图数据库
│   └── publish/               # 发布配置
└── public/                    # 静态网站输出
    ├── zettel/               # 单卡片页面
    ├── threads/              # 主题串联页面
    ├── graph/                # 知识图谱
    └── index.html            # 首页
2.3 卡片格式示例
markdown
---
id: "202401201430"
title: "知识的原子化组织"
type: "permanent"
tags: ["知识管理", "Zettelkasten", "原子笔记"]
created: 2024-01-20T14:30:00Z
modified: 2024-01-20T15:45:00Z
publish: "public"
---

# 知识的原子化组织

知识的原子化是 Zettelkasten 方法的核心原则。每个笔记应该包含一个独立、完整的想法。

这种方法的优势在于：
- 易于链接和组合 [[202401201445|组合性思维]]
- 降低认知负担
- 促进思维的精确性

与传统的主题式笔记不同 [[202401201020|传统笔记的局限性]]，原子化笔记更像是思维的乐高积木。

## 相关概念
- [[202401191230|最小可行知识单元]]
- [[202401201520|链接的密度与质量]]

## 参考文献
- Ahrens, S. (2017). How to Take Smart Notes.
3. 双向链接与图谱系统
3.1 链接解析与管理
rust
// services/link_manager.rs
pub struct LinkManager {
    graph: Graph<String, LinkType>,
    index: HashMap<String, NodeIndex>,
}

impl LinkManager {
    pub fn parse_links(content: &str) -> Vec<ZettelLink> {
        let link_pattern = Regex::new(r"\[\[(\d{12})(?:\|([^\]]+))?\]\]").unwrap();

        link_pattern.captures_iter(content)
            .map(|cap| ZettelLink {
                target_id: cap[1].to_string(),
                link_type: LinkType::Reference,
                context: cap.get(2).map(|m| m.as_str().to_string()),
            })
            .collect()
    }

    pub fn build_graph(&mut self, zettels: &[Zettel]) -> Result<()> {
        // 构建知识图谱
        for zettel in zettels {
            let node = self.graph.add_node(zettel.id.clone());
            self.index.insert(zettel.id.clone(), node);
        }

        for zettel in zettels {
            let source = self.index[&zettel.id];
            for link in &zettel.links {
                if let Some(&target) = self.index.get(&link.target_id) {
                    self.graph.add_edge(source, target, link.link_type.clone());
                }
            }
        }

        Ok(())
    }

    pub fn get_backlinks(&self, zettel_id: &str) -> Vec<String> {
        if let Some(&node) = self.index.get(zettel_id) {
            self.graph.edges_directed(node, Direction::Incoming)
                .map(|edge| self.graph[edge.source()].clone())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn find_paths(&self, from: &str, to: &str, max_depth: usize) -> Vec<Vec<String>> {
        // 查找两个笔记之间的所有路径
        // 用于发现隐藏的联系
    }

    pub fn detect_clusters(&self) -> Vec<Vec<String>> {
        // 检测紧密相关的笔记群组
        // 用于发现主题和概念群
    }
}
3.2 知识图谱数据结构
sql
-- 图数据库schema (使用SQLite实现)
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    type TEXT NOT NULL,
    word_count INTEGER,
    created_at TIMESTAMP,
    publish_status TEXT
);

CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    link_type TEXT NOT NULL,
    context TEXT,
    created_at TIMESTAMP,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
);

-- 用于快速查询的索引
CREATE INDEX idx_edges_target ON edges(target_id);
CREATE INDEX idx_nodes_type ON nodes(type);
CREATE INDEX idx_nodes_publish ON nodes(publish_status);

-- 图谱统计视图
CREATE VIEW graph_stats AS
SELECT
    n.id,
    n.title,
    COUNT(DISTINCT e_out.target_id) as out_degree,
    COUNT(DISTINCT e_in.source_id) as in_degree,
    COUNT(DISTINCT e_out.target_id) + COUNT(DISTINCT e_in.source_id) as total_degree
FROM nodes n
LEFT JOIN edges e_out ON n.id = e_out.source_id
LEFT JOIN edges e_in ON n.id = e_in.target_id
GROUP BY n.id;
4. 发布系统设计
4.1 混合发布模式
rust
// services/publisher/zettel_publisher.rs
pub struct ZettelPublisher {
    link_manager: LinkManager,
    template_engine: Tera,
    config: PublishConfig,
}

impl ZettelPublisher {
    pub async fn publish(&self, zettels: &[Zettel]) -> Result<()> {
        // 1. 发布单个卡片
        self.publish_individual_zettels(zettels).await?;

        // 2. 生成结构化文章
        self.publish_structured_articles(zettels).await?;

        // 3. 生成索引页面
        self.generate_indices(zettels).await?;

        // 4. 生成知识图谱
        self.generate_graph_visualization(zettels).await?;

        Ok(())
    }

    async fn publish_individual_zettels(&self, zettels: &[Zettel]) -> Result<()> {
        for zettel in zettels {
            match zettel.publish_status {
                PublishStatus::Public => {
                    let html = self.render_zettel(zettel).await?;
                    let path = format!("public/zettel/{}.html", zettel.id);
                    fs::write(path, html).await?;
                }
                PublishStatus::Draft => {
                    // 生成占位符页面
                    let placeholder = self.render_placeholder(zettel).await?;
                    let path = format!("public/zettel/{}.html", zettel.id);
                    fs::write(path, placeholder).await?;
                }
                PublishStatus::Private => {
                    // 不发布，但可能需要在其他页面中处理链接
                }
            }
        }
        Ok(())
    }

    async fn render_zettel(&self, zettel: &Zettel) -> Result<String> {
        let mut context = Context::new();
        context.insert("zettel", &zettel);

        // 获取反向链接
        let backlinks = self.link_manager.get_backlinks(&zettel.id);
        context.insert("backlinks", &backlinks);

        // 处理内容中的链接
        let processed_content = self.process_links(&zettel.content, &zettel.links).await?;
        context.insert("content", &processed_content);

        self.template_engine.render("zettel.html", &context)
            .map_err(Into::into)
    }

    async fn process_links(&self, content: &str, links: &[ZettelLink]) -> Result<String> {
        let mut processed = content.to_string();

        for link in links {
            let target_zettel = self.get_zettel(&link.target_id).await?;
            let replacement = match target_zettel.publish_status {
                PublishStatus::Public => {
                    format!(
                        r#"<a href="/zettel/{}.html" class="zettel-link" data-type="{:?}">{}</a>"#,
                        link.target_id,
                        link.link_type,
                        link.context.as_ref().unwrap_or(&target_zettel.title)
                    )
                }
                PublishStatus::Draft => {
                    format!(
                        r#"<a href="/zettel/{}.html" class="zettel-link draft" title="草稿">{}</a>"#,
                        link.target_id,
                        link.context.as_ref().unwrap_or(&target_zettel.title)
                    )
                }
                PublishStatus::Private => {
                    format!(
                        r#"<span class="zettel-link private" title="私有笔记">{}</span>"#,
                        link.context.as_ref().unwrap_or(&target_zettel.title)
                    )
                }
            };

            let pattern = format!(r"\[\[{}\|?[^\]]*\]\]", link.target_id);
            processed = Regex::new(&pattern)?.replace(&processed, replacement).to_string();
        }

        Ok(processed)
    }
}
4.2 结构化文章生成
rust
// models/structure_note.rs
#[derive(Serialize, Deserialize)]
pub struct StructureNote {
    pub id: String,
    pub title: String,
    pub outline: Vec<OutlineItem>,
    pub metadata: StructureMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct OutlineItem {
    pub zettel_id: String,
    pub heading: Option<String>,
    pub children: Vec<OutlineItem>,
    pub transition_text: Option<String>,
}

impl ZettelPublisher {
    async fn publish_structured_articles(&self, zettels: &[Zettel]) -> Result<()> {
        let structure_notes = zettels.iter()
            .filter(|z| matches!(z.zettel_type, ZettelType::Structure))
            .collect::<Vec<_>>();

        for structure in structure_notes {
            let article = self.build_article_from_structure(structure).await?;
            let path = format!("public/threads/{}.html", slugify(&structure.title));
            fs::write(path, article).await?;
        }

        Ok(())
    }

    async fn build_article_from_structure(&self, structure: &Zettel) -> Result<String> {
        let outline = self.parse_structure_outline(&structure.content)?;
        let mut article_content = String::new();

        article_content.push_str(&format!("# {}\n\n", structure.title));

        for item in outline.items {
            self.append_outline_item(&mut article_content, &item, 2).await?;
        }

        // 渲染为HTML
        let mut context = Context::new();
        context.insert("title", &structure.title);
        context.insert("content", &markdown_to_html(&article_content)?);
        context.insert("structure_id", &structure.id);
        context.insert("toc", &self.generate_toc(&outline)?);

        self.template_engine.render("article.html", &context)
            .map_err(Into::into)
    }
}
4.3 索引页面生成
rust
// services/index_generator.rs
pub struct IndexGenerator {
    template_engine: Tera,
}

impl IndexGenerator {
    pub async fn generate_all_indices(&self, zettels: &[Zettel]) -> Result<()> {
        // 主索引
        self.generate_main_index(zettels).await?;

        // 时间线索引
        self.generate_chronological_index(zettels).await?;

        // 标签索引
        self.generate_tag_index(zettels).await?;

        // 类型索引
        self.generate_type_index(zettels).await?;

        Ok(())
    }

    async fn generate_main_index(&self, zettels: &[Zettel]) -> Result<()> {
        let index_notes = zettels.iter()
            .filter(|z| matches!(z.zettel_type, ZettelType::Index))
            .collect::<Vec<_>>();

        let mut context = Context::new();
        context.insert("indices", &index_notes);
        context.insert("total_zettels", &zettels.len());
        context.insert("public_zettels", &zettels.iter()
            .filter(|z| matches!(z.publish_status, PublishStatus::Public))
            .count());

        let html = self.template_engine.render("index.html", &context)?;
        fs::write("public/index.html", html).await?;

        Ok(())
    }
}
5. 知识图谱可视化
5.1 图谱数据API
rust
// commands/graph.rs
#[tauri::command]
pub async fn get_graph_data(
    filter: Option<GraphFilter>,
    state: State<'_, AppState>,
) -> Result<GraphData, String> {
    let zettels = state.zettel_service.get_all_zettels().await?;
    let graph = state.link_manager.build_graph(&zettels)?;

    let nodes = zettels.iter()
        .filter(|z| filter.as_ref().map_or(true, |f| f.matches(z)))
        .map(|z| GraphNode {
            id: z.id.clone(),
            title: z.title.clone(),
            type: z.zettel_type.clone(),
            publish_status: z.publish_status.clone(),
            size: calculate_node_size(z),
            color: get_node_color(z),
        })
        .collect();

    let edges = graph.edges()
        .map(|e| GraphEdge {
            source: e.source().to_string(),
            target: e.target().to_string(),
            link_type: e.weight().clone(),
        })
        .collect();

    Ok(GraphData { nodes, edges })
}
5.2 前端图谱组件
typescript
// components/KnowledgeGraph/index.tsx
import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { useGraphData } from '@/hooks/useGraphData';

interface KnowledgeGraphProps {
  filter?: GraphFilter;
  onNodeClick?: (nodeId: string) => void;
  height?: number;
}

export function KnowledgeGraph({ filter, onNodeClick, height = 600 }: KnowledgeGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const { data, loading } = useGraphData(filter);

  useEffect(() => {
    if (!data || !svgRef.current) return;

    const svg = d3.select(svgRef.current);
    const width = svgRef.current.clientWidth;

    // 力导向图布局
    const simulation = d3.forceSimulation(data.nodes)
      .force('link', d3.forceLink(data.edges)
        .id((d: any) => d.id)
        .distance(d => getLinkDistance(d)))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(d => d.size + 5));

    // 缩放功能
    const zoom = d3.zoom()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        container.attr('transform', event.transform);
      });

    svg.call(zoom);

    const container = svg.append('g');

    // 绘制边
    const links = container.append('g')
      .selectAll('line')
      .data(data.edges)
      .enter().append('line')
      .attr('class', d => `link link-${d.link_type}`)
      .attr('stroke-width', 2);

    // 绘制节点
    const nodes = container.append('g')
      .selectAll('g')
      .data(data.nodes)
      .enter().append('g')
      .attr('class', 'node')
      .call(d3.drag()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended));

    nodes.append('circle')
      .attr('r', d => d.size)
      .attr('fill', d => d.color)
      .attr('class', d => `node-${d.publish_status}`);

    nodes.append('text')
      .text(d => d.title)
      .attr('x', 0)
      .attr('y', -15)
      .attr('text-anchor', 'middle')
      .attr('font-size', '12px');

    // 点击事件
    nodes.on('click', (event, d) => {
      if (onNodeClick) {
        onNodeClick(d.id);
      }
    });

    // 高亮相关节点
    nodes.on('mouseover', function(event, d) {
      const connectedNodes = getConnectedNodes(d.id, data.edges);

      nodes.classed('dimmed', n => !connectedNodes.has(n.id));
      links.classed('dimmed', l =>
        l.source.id !== d.id && l.target.id !== d.id
      );
    });

    nodes.on('mouseout', function() {
      nodes.classed('dimmed', false);
      links.classed('dimmed', false);
    });

    simulation.on('tick', () => {
      links
        .attr('x1', d => d.source.x)
        .attr('y1', d => d.source.y)
        .attr('x2', d => d.target.x)
        .attr('y2', d => d.target.y);

      nodes.attr('transform', d => `translate(${d.x},${d.y})`);
    });

    return () => {
      simulation.stop();
    };
  }, [data, height, onNodeClick]);

  if (loading) return <div>加载中...</div>;

  return (
    <div className="knowledge-graph">
      <svg ref={svgRef} width="100%" height={height}>
        <defs>
          <marker id="arrowhead" markerWidth="10" markerHeight="7"
                  refX="0" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#999" />
          </marker>
        </defs>
      </svg>
      <GraphLegend />
    </div>
  );
}
6. 静态网站模板
6.1 单卡片页面模板
html
<!-- templates/zettel.html -->
<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <title>{{ zettel.title }} - Zeno</title>
    <link rel="stylesheet" href="/css/zettel.css">
</head>
<body>
    <nav class="breadcrumb">
        <a href="/">首页</a> >
        <a href="/zettel">卡片</a> >
        <span>{{ zettel.id }}</span>
    </nav>

    <article class="zettel">
        <header>
            <h1>{{ zettel.title }}</h1>
            <div class="metadata">
                <span class="id">ID: {{ zettel.id }}</span>
                <span class="type">类型: {{ zettel.type }}</span>
                <time>创建: {{ zettel.created_at | date }}</time>
            </div>
            <div class="tags">
                {% for tag in zettel.tags %}
                <a href="/tags/{{ tag }}" class="tag">#{{ tag }}</a>
                {% endfor %}
            </div>
        </header>

        <div class="content">
            {{ content | safe }}
        </div>

        <aside class="connections">
            <section class="backlinks">
                <h2>反向链接</h2>
                {% if backlinks %}
                <ul>
                    {% for link in backlinks %}
                    <li>
                        <a href="/zettel/{{ link.id }}.html">
                            {{ link.title }}
                        </a>
                        <span class="context">{{ link.context }}</span>
                    </li>
                    {% endfor %}
                </ul>
                {% else %}
                <p class="empty">暂无反向链接</p>
                {% endif %}
            </section>

            <section class="related">
                <h2>相关卡片</h2>
                <div class="graph-preview" data-zettel-id="{{ zettel.id }}">
                    <!-- 小型知识图谱 -->
                </div>
            </section>
        </aside>
    </article>

    <script src="/js/zettel.js"></script>
</body>
</html>
6.2 知识图谱页面
html
<!-- templates/graph.html -->
<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <title>知识图谱 - Zeno</title>
    <link rel="stylesheet" href="/css/graph.css">
</head>
<body>
    <div class="graph-container">
        <header>
            <h1>知识图谱</h1>
            <div class="controls">
                <div class="filter-group">
                    <label>类型筛选:</label>
                    <select id="type-filter" multiple>
                        <option value="permanent">永久笔记</option>
                        <option value="literature">文献笔记</option>
                        <option value="structure">结构笔记</option>
                        <option value="index">索引笔记</option>
                    </select>
                </div>

                <div class="filter-group">
                    <label>标签筛选:</label>
                    <input type="text" id="tag-filter" placeholder="输入标签...">
                </div>

                <div class="view-options">
                    <button id="zoom-fit">适应窗口</button>
                    <button id="toggle-labels">切换标签</button>
                    <button id="export-svg">导出图片</button>
                </div>
            </div>
        </header>

        <div class="graph-main">
            <div id="graph-canvas"></div>

            <aside class="graph-sidebar">
                <div class="node-info" id="node-info">
                    <h3>选择一个节点查看详情</h3>
                </div>

                <div class="graph-stats">
                    <h3>图谱统计</h3>
                    <ul>
                        <li>总节点数: <span id="total-nodes">0</span></li>
                        <li>总连接数: <span id="total-edges">0</span></li>
                        <li>平均连接度: <span id="avg-degree">0</span></li>
                        <li>最大连接节点: <span id="max-degree-node">-</span></li>
                    </ul>
                </div>
            </aside>
        </div>
    </div>

    <script src="/js/d3.min.js"></script>
    <script src="/js/graph.js"></script>
</body>
</html>
7. 配置文件
7.1 Zeno Zettelkasten 配置
toml
# ~/.zeno/config.toml
[zeno]
version = "0.2.0"
mode = "zettelkasten"
zettel_dir = "~/Documents/Zettelkasten"

[zettel]
# ID 生成设置
id_format = "timestamp"  # timestamp 或 custom
id_timezone = "Asia/Shanghai"
id_precision = "minute"  # minute 或 second

# 默认设置
default_type = "permanent"
default_publish_status = "private"

# 文件命名
filename_format = "{id}.md"  # 或 "{id}-{slug}.md"

[links]
# 链接格式
wiki_links = true  # 支持 [[id]] 格式
markdown_links = true  # 支持 [text](id) 格式

# 链接类型
enable_typed_links = true
link_types = ["reference", "continuation", "contradiction", "branch"]

[publish]
# 发布设置
base_url = "https://notes.example.com"
theme = "zettel-garden"

# 页面生成
generate_individual_pages = true
generate_structure_pages = true
generate_indices = true
generate_graph = true

# 占位符设置
placeholder_template = "placeholder.html"
show_private_as_placeholder = true

[graph]
# 图谱设置
max_nodes = 500
default_depth = 2
layout = "force-directed"  # force-directed, hierarchical, circular

# 节点样式
node_size_metric = "connections"  # connections, word_count, age
color_scheme = "type"  # type, tag, publish_status

[search]
# 搜索设置
enable_fuzzy_search = true
search_content = true
search_titles = true
search_tags = true
min_search_length = 2

[templates]
# 模板设置
template_dir = "~/.zeno/templates"
default_zettel_template = "zettel.md"
default_structure_template = "structure.md"
7.2 发布配置
toml
# ~/.zeno/publish.toml
[site]
title = "我的数字花园"
description = "一个基于 Zettelkasten 的个人知识库"
author = "Your Name"
language = "zh-CN"

[navigation]
# 导航菜单
[[navigation.items]]
name = "首页"
url = "/"

[[navigation.items]]
name = "索引"
url = "/indices/"

[[navigation.items]]
name = "图谱"
url = "/graph/"

[[navigation.items]]
name = "时间线"
url = "/timeline/"

[[navigation.items]]
name = "关于"
url = "/about/"

[features]
# 功能开关
enable_search = true
enable_rss = true
enable_sitemap = true
enable_comments = false
enable_analytics = true

# 图谱设置
[features.graph]
enable_minimap = true
enable_zoom = true
enable_fullscreen = true
save_position = true

# 阅读体验
[features.reading]
enable_toc = true
enable_reading_time = true
enable_progress_bar = true
enable_dark_mode = true

[seo]
# SEO 设置
enable_og_tags = true
enable_twitter_cards = true
default_image = "/images/og-default.png"
8. 核心工作流
8.1 日常使用流程
bash
# 1. 创建新卡片
zeno new                    # 交互式创建
zeno new "想法标题"         # 快速创建

# 2. 快速捕获
zeno quick "临时想法"       # 创建日常笔记
zeno capture <url>          # 从网页创建文献笔记

# 3. 链接和发展
zeno link 202401201430 202401201445  # 建立链接
zeno continue 202401201430           # 创建延续卡片
zeno branch 202401201430             # 创建分支卡片

# 4. 组织和结构
zeno structure create "主题名"       # 创建结构笔记
zeno index update                    # 更新索引

# 5. 搜索和探索
zeno search "关键词"                 # 全文搜索
zeno related 202401201430            # 查找相关卡片
zeno path 202401201430 202401201845  # 查找连接路径

# 6. 发布
zeno publish                         # 发布所有公开卡片
zeno publish --preview               # 预览发布效果
zeno publish --incremental           # 增量发布
8.2 Tauri 命令接口
rust
// commands/zettel.rs
#[tauri::command]
pub async fn create_zettel(
    title: String,
    content: Option<String>,
    zettel_type: Option<ZettelType>,
    state: State<'_, AppState>,
) -> Result<Zettel, String> {
    let id = generate_timestamp_id();
    let zettel = Zettel {
        id: id.clone(),
        title,
        content: content.unwrap_or_default(),
        zettel_type: zettel_type.unwrap_or(ZettelType::Permanent),
        tags: vec![],
        links: vec![],
        created_at: Utc::now(),
        modified_at: Utc::now(),
        publish_status: PublishStatus::Private,
    };

    state.zettel_service.create_zettel(zettel).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn link_zettels(
    source_id: String,
    target_id: String,
    link_type: LinkType,
    context: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.zettel_service
        .add_link(&source_id, &target_id, link_type, context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_paths(
    from_id: String,
    to_id: String,
    max_depth: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<String>>, String> {
    state.link_manager
        .find_paths(&from_id, &to_id, max_depth.unwrap_or(5))
        .await
        .map_err(|e| e.to_string())
}
9. 优势总结
真正的 Zettelkasten：
原子化笔记
时间戳 ID 系统
多种链接类型
支持思维演化
强大的发布系统：
混合发布模式
保留双向链接
占位符机制
SEO 友好
直观的知识探索：
交互式图谱
多维度索引
路径发现
聚类分析
灵活的工作流：
快速捕获
渐进发展
结构化组织
批量操作
这个基于 Zettelkasten 的架构让 Zeno 成为一个真正的"第二大脑"工具，既适合个人知识管理，又能优雅地分享知识。
