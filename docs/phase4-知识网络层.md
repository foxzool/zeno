# Phase 4: 知识网络层开发计划

## 阶段概述

实现Noto的核心特色功能 - 知识网络和智能关联系统。通过双向链接、知识图谱可视化和智能分析，帮助用户构建和发现知识之间的深层联系。

**预计时间**: 3-4周  
**优先级**: 高 (差异化特色)  
**前置条件**: Phase 3用户界面层完成

## 目标与交付物

### 主要目标
- 实现Wiki风格的双向链接系统
- 建立交互式知识图谱可视化
- 提供智能的内容关联分析
- 创建层次化的标签分类系统

### 交付物
- 双向链接解析和管理引擎
- 知识图谱可视化组件
- 智能标签和分类系统
- 关联性分析和推荐算法

## 详细任务清单

### 4.1 双向链接系统

**任务描述**: 实现Wiki风格的链接语法和管理系统

**链接语法设计**:
```markdown
# 支持的链接格式
[[笔记标题]]                    # 基础链接
[[笔记标题|显示文本]]            # 别名链接
[[笔记标题#章节]]               # 锚点链接
[[路径/笔记标题]]               # 路径链接
![[图片文件]]                   # 嵌入资源
```

**链接解析器实现**:
```rust
// services/link_parser.rs
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct WikiLink {
    pub raw: String,           // 原始文本 [[target|alias]]
    pub target: String,        // 目标文件 target
    pub alias: Option<String>, // 显示别名 alias
    pub anchor: Option<String>,// 锚点 #section
    pub is_embed: bool,        // 是否为嵌入 ![[]]
    pub range: Range<usize>,   // 在文档中的位置
}

pub struct LinkParser {
    wiki_link_regex: Regex,
    markdown_link_regex: Regex,
}

impl LinkParser {
    pub fn new() -> Self {
        Self {
            wiki_link_regex: Regex::new(
                r"(!?)\[\[([^\]]+?)(?:\|([^\]]+?))?\]\]"
            ).unwrap(),
            markdown_link_regex: Regex::new(
                r"\[([^\]]+?)\]\(([^)]+?)\)"
            ).unwrap(),
        }
    }
    
    pub fn extract_links(&self, content: &str) -> Vec<WikiLink> {
        let mut links = Vec::new();
        
        // 解析 Wiki 链接
        for cap in self.wiki_link_regex.captures_iter(content) {
            let is_embed = cap.get(1).map_or(false, |m| m.as_str() == "!");
            let target_with_anchor = cap.get(2).unwrap().as_str();
            let alias = cap.get(3).map(|m| m.as_str().to_string());
            
            let (target, anchor) = if let Some(pos) = target_with_anchor.find('#') {
                (
                    target_with_anchor[..pos].to_string(),
                    Some(target_with_anchor[pos+1..].to_string())
                )
            } else {
                (target_with_anchor.to_string(), None)
            };
            
            links.push(WikiLink {
                raw: cap.get(0).unwrap().as_str().to_string(),
                target,
                alias,
                anchor,
                is_embed,
                range: cap.get(0).unwrap().range(),
            });
        }
        
        links
    }
    
    pub fn resolve_link(&self, link: &WikiLink, base_path: &Path) -> Option<PathBuf> {
        // 链接解析逻辑
        // 1. 精确匹配文件名
        // 2. 模糊匹配标题
        // 3. 相对路径解析
        todo!()
    }
}
```

**链接索引和管理**:
```rust
// services/link_index.rs
use std::collections::{HashMap, HashSet};

pub struct LinkIndex {
    db_pool: SqlitePool,
    // 正向链接: 从这个笔记链接到其他笔记
    outgoing_links: HashMap<String, HashSet<String>>,
    // 反向链接: 其他笔记链接到这个笔记
    incoming_links: HashMap<String, HashSet<String>>,
    // 孤立笔记: 没有任何链接的笔记
    orphaned_notes: HashSet<String>,
    // 断链: 指向不存在文件的链接
    broken_links: HashMap<String, Vec<WikiLink>>,
}

impl LinkIndex {
    pub async fn update_note_links(&mut self, note_id: &str, links: Vec<WikiLink>) -> Result<()> {
        // 清理旧链接
        self.remove_note_links(note_id).await?;
        
        // 添加新链接
        for link in links {
            if let Some(target_id) = self.resolve_link_target(&link).await? {
                self.add_link(note_id, &target_id, &link).await?;
            } else {
                // 记录断链
                self.broken_links
                    .entry(note_id.to_string())
                    .or_default()
                    .push(link);
            }
        }
        
        // 更新孤立笔记状态
        self.update_orphaned_status(note_id).await?;
        
        Ok(())
    }
    
    pub async fn get_backlinks(&self, note_id: &str) -> Result<Vec<BacklinkInfo>> {
        let incoming = self.incoming_links.get(note_id)
            .map(|set| set.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        
        let mut backlinks = Vec::new();
        for source_id in incoming {
            let note = self.get_note_info(&source_id).await?;
            let context = self.get_link_context(&source_id, note_id).await?;
            
            backlinks.push(BacklinkInfo {
                source_note: note,
                context,
                link_count: self.count_links_between(&source_id, note_id).await?,
            });
        }
        
        Ok(backlinks)
    }
    
    pub async fn find_similar_notes(&self, note_id: &str) -> Result<Vec<SimilarNote>> {
        // 基于共同链接计算相似度
        let outgoing = self.outgoing_links.get(note_id).unwrap_or(&HashSet::new());
        let incoming = self.incoming_links.get(note_id).unwrap_or(&HashSet::new());
        
        let mut similarity_scores = HashMap::new();
        
        // 计算基于共同出链的相似度
        for other_note in self.outgoing_links.keys() {
            if other_note == note_id { continue; }
            
            let other_outgoing = self.outgoing_links.get(other_note).unwrap();
            let common_links = outgoing.intersection(other_outgoing).count();
            let total_links = outgoing.union(other_outgoing).count();
            
            if total_links > 0 {
                let score = common_links as f64 / total_links as f64;
                similarity_scores.insert(other_note.clone(), score);
            }
        }
        
        // 排序并返回最相似的笔记
        let mut similar_notes: Vec<_> = similarity_scores.into_iter().collect();
        similar_notes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let mut result = Vec::new();
        for (note_id, score) in similar_notes.into_iter().take(10) {
            let note_info = self.get_note_info(&note_id).await?;
            result.push(SimilarNote {
                note: note_info,
                similarity_score: score,
                common_links: self.get_common_links(note_id, &note_id).await?,
            });
        }
        
        Ok(result)
    }
}
```

**验收标准**:
- [ ] 链接解析准确率 > 99%
- [ ] 大量链接 (1k+) 的解析时间 < 100ms
- [ ] 断链检测和提示功能正常
- [ ] 链接重命名和批量更新正确
- [ ] 反向链接实时更新

### 4.2 知识图谱可视化

**任务描述**: 使用图形库实现交互式知识图谱

**图形库选型**:
```typescript
// 技术选型对比
const GRAPH_LIBRARIES = {
  d3: {
    pros: ['高度可定制', '性能优秀', '社区强大'],
    cons: ['学习曲线陡峭', '开发复杂'],
    适用场景: '复杂定制需求'
  },
  cytoscapejs: {
    pros: ['专为图分析设计', 'API简洁', '布局算法丰富'],
    cons: ['包体积较大', '某些定制受限'],
    适用场景: '图分析为主'
  },
  sigmajs: {
    pros: ['大图性能优秀', '美观的默认样式'],
    cons: ['功能相对有限', '定制性一般'],
    适用场景: '大规模图展示'
  },
  'react-force-graph': {
    pros: ['React集成方便', '开箱即用'],
    cons: ['性能一般', '定制有限'],
    适用场景: '快速原型'
  }
};

// 推荐选择: Cytoscape.js (功能完整，性能好)
```

**图谱组件实现**:
```typescript
// components/KnowledgeGraph/index.tsx
import cytoscape, { Core, ElementDefinition } from 'cytoscape';
import cola from 'cytoscape-cola';
import fcose from 'cytoscape-fcose';

cytoscape.use(cola);
cytoscape.use(fcose);

export interface GraphNode {
  id: string;
  label: string;
  type: 'note' | 'tag' | 'category';
  size: number;
  color: string;
  metadata: {
    path: string;
    wordCount: number;
    lastModified: string;
    tags: string[];
  };
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: 'link' | 'tag' | 'similarity';
  weight: number;
  metadata: {
    linkText?: string;
    context?: string;
  };
}

export const KnowledgeGraph: React.FC<KnowledgeGraphProps> = ({
  nodes,
  edges,
  selectedNodeId,
  onNodeSelect,
  onNodeDoubleClick,
  layout = 'fcose',
  style = {},
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<Core | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [filters, setFilters] = useState<GraphFilters>({
    showTags: true,
    showCategories: true,
    minLinkWeight: 0,
    nodeTypes: ['note', 'tag', 'category'],
  });
  
  // 初始化图谱
  useEffect(() => {
    if (!containerRef.current) return;
    
    const cy = cytoscape({
      container: containerRef.current,
      elements: [],
      style: [
        {
          selector: 'node[type="note"]',
          style: {
            'background-color': '#3B82F6',
            'label': 'data(label)',
            'width': 'data(size)',
            'height': 'data(size)',
            'font-size': '12px',
            'text-valign': 'center',
            'text-halign': 'center',
            'color': '#FFFFFF',
          }
        },
        {
          selector: 'node[type="tag"]',
          style: {
            'background-color': '#10B981',
            'shape': 'triangle',
            'label': 'data(label)',
            'width': 'data(size)',
            'height': 'data(size)',
          }
        },
        {
          selector: 'edge',
          style: {
            'width': 'data(weight)',
            'line-color': '#64748B',
            'target-arrow-color': '#64748B',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
          }
        },
        {
          selector: '.highlighted',
          style: {
            'background-color': '#EF4444',
            'line-color': '#EF4444',
            'target-arrow-color': '#EF4444',
          }
        },
        {
          selector: '.selected',
          style: {
            'border-width': 3,
            'border-color': '#FBBF24',
          }
        }
      ],
      layout: {
        name: layout,
        animate: true,
        animationDuration: 1000,
        // fcose 布局参数
        ...(layout === 'fcose' && {
          nodeRepulsion: 4000,
          idealEdgeLength: 100,
          edgeElasticity: 0.1,
          nestingFactor: 0.1,
          gravity: 0.8,
          numIter: 2500,
        }),
      },
    });
    
    cyRef.current = cy;
    
    // 事件监听
    cy.on('tap', 'node', (event) => {
      const nodeId = event.target.id();
      onNodeSelect?.(nodeId);
      highlightNeighborhood(nodeId);
    });
    
    cy.on('dbltap', 'node', (event) => {
      const nodeId = event.target.id();
      onNodeDoubleClick?.(nodeId);
    });
    
    // 鼠标悬停效果
    cy.on('mouseover', 'node', (event) => {
      const node = event.target;
      node.style('cursor', 'pointer');
      showNodeTooltip(node);
    });
    
    cy.on('mouseout', 'node', () => {
      hideNodeTooltip();
    });
    
    setIsLoading(false);
    
    return () => {
      cy.destroy();
    };
  }, []);
  
  // 更新图谱数据
  useEffect(() => {
    if (!cyRef.current) return;
    
    const filteredNodes = nodes.filter(node => 
      filters.nodeTypes.includes(node.type) &&
      (searchQuery === '' || node.label.toLowerCase().includes(searchQuery.toLowerCase()))
    );
    
    const filteredEdges = edges.filter(edge =>
      edge.weight >= filters.minLinkWeight &&
      filteredNodes.some(n => n.id === edge.source) &&
      filteredNodes.some(n => n.id === edge.target)
    );
    
    const elements: ElementDefinition[] = [
      ...filteredNodes.map(node => ({
        data: {
          id: node.id,
          label: node.label,
          type: node.type,
          size: Math.max(20, Math.min(60, node.size)),
          color: node.color,
          ...node.metadata,
        }
      })),
      ...filteredEdges.map(edge => ({
        data: {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          type: edge.type,
          weight: Math.max(1, Math.min(5, edge.weight)),
          ...edge.metadata,
        }
      }))
    ];
    
    cyRef.current.elements().remove();
    cyRef.current.add(elements);
    
    // 重新应用布局
    cyRef.current.layout({ name: layout }).run();
  }, [nodes, edges, filters, searchQuery, layout]);
  
  // 高亮相邻节点
  const highlightNeighborhood = useCallback((nodeId: string) => {
    if (!cyRef.current) return;
    
    const cy = cyRef.current;
    
    // 清除之前的高亮
    cy.elements().removeClass('highlighted selected');
    
    // 选中当前节点
    const selectedNode = cy.$(`#${nodeId}`);
    selectedNode.addClass('selected');
    
    // 高亮相邻节点和边
    const neighborhood = selectedNode.neighborhood();
    neighborhood.addClass('highlighted');
  }, []);
  
  return (
    <div className="knowledge-graph">
      {/* 控制面板 */}
      <div className="graph-controls">
        <div className="search-bar">
          <input
            type="text"
            placeholder="搜索节点..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        
        <div className="layout-controls">
          <select
            value={layout}
            onChange={(e) => setLayout(e.target.value as LayoutName)}
          >
            <option value="fcose">力导向 (推荐)</option>
            <option value="cola">COLA</option>
            <option value="cose">CoSE</option>
            <option value="circle">圆形</option>
            <option value="grid">网格</option>
          </select>
        </div>
        
        <div className="filter-controls">
          <FilterPanel filters={filters} onFiltersChange={setFilters} />
        </div>
      </div>
      
      {/* 图谱容器 */}
      <div 
        ref={containerRef} 
        className="graph-container"
        style={{ height: '600px', width: '100%' }}
      />
      
      {/* 加载状态 */}
      {isLoading && (
        <div className="graph-loading">
          <Spinner /> 加载图谱中...
        </div>
      )}
      
      {/* 工具提示 */}
      <NodeTooltip />
    </div>
  );
};
```

**图谱布局算法**:
```typescript
// utils/graphLayouts.ts
export interface LayoutConfig {
  name: string;
  options: any;
  description: string;
  bestFor: string[];
}

export const GRAPH_LAYOUTS: Record<string, LayoutConfig> = {
  fcose: {
    name: 'fcose',
    options: {
      animate: true,
      animationDuration: 1000,
      nodeRepulsion: 4000,
      idealEdgeLength: 100,
      edgeElasticity: 0.1,
      nestingFactor: 0.1,
      gravity: 0.8,
      numIter: 2500,
      tile: true,
      tilingPaddingVertical: 10,
      tilingPaddingHorizontal: 10,
    },
    description: '快速复合有机布局',
    bestFor: ['大中型图', '一般用途', '美观度优先'],
  },
  
  cola: {
    name: 'cola',
    options: {
      animate: true,
      maxSimulationTime: 2000,
      edgeLength: 80,
      nodeSpacing: 30,
      flow: { axis: 'y', minSeparation: 30 },
    },
    description: '约束导向布局',
    bestFor: ['层次化数据', '流程图', '有向图'],
  },
  
  cose: {
    name: 'cose',
    options: {
      animate: true,
      animationDuration: 1000,
      nodeRepulsion: 400000,
      nodeOverlap: 10,
      idealEdgeLength: 10,
      edgeElasticity: 100,
      nestingFactor: 5,
      gravity: 250,
      numIter: 100,
    },
    description: '复合弹簧嵌入式布局',
    bestFor: ['小型图', '详细查看', '精确定位'],
  },
};

export const getOptimalLayout = (nodeCount: number, edgeCount: number): string => {
  if (nodeCount < 50) return 'cose';
  if (nodeCount < 200) return 'fcose';
  if (edgeCount / nodeCount > 2) return 'cola';
  return 'fcose';
};
```

**验收标准**:
- [ ] 1000节点图谱渲染时间 < 3秒
- [ ] 图谱交互响应流畅 (60fps)
- [ ] 布局算法收敛正确
- [ ] 节点和边的样式可配置
- [ ] 搜索和过滤功能正常

### 4.3 标签和分类系统

**任务描述**: 实现智能化的标签管理和层次分类

**层次化标签结构**:
```rust
// models/tag.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub children: Vec<Tag>,
    pub note_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagHierarchy {
    pub root_tags: Vec<Tag>,
    pub tag_map: HashMap<i32, Tag>,
    pub path_map: HashMap<String, i32>, // 路径 -> ID映射
}

impl TagHierarchy {
    pub fn new(tags: Vec<Tag>) -> Self {
        let mut root_tags = Vec::new();
        let mut tag_map = HashMap::new();
        let mut path_map = HashMap::new();
        
        // 构建树形结构
        for tag in tags {
            if tag.parent_id.is_none() {
                root_tags.push(tag.clone());
            }
            
            // 构建路径映射
            let path = Self::build_tag_path(&tag, &tag_map);
            path_map.insert(path, tag.id);
            tag_map.insert(tag.id, tag);
        }
        
        Self { root_tags, tag_map, path_map }
    }
    
    pub fn find_tag_by_path(&self, path: &str) -> Option<&Tag> {
        self.path_map.get(path)
            .and_then(|id| self.tag_map.get(id))
    }
    
    pub fn get_tag_suggestions(&self, input: &str) -> Vec<TagSuggestion> {
        let input_lower = input.to_lowercase();
        let mut suggestions = Vec::new();
        
        for tag in self.tag_map.values() {
            if tag.name.to_lowercase().contains(&input_lower) {
                let relevance = Self::calculate_relevance(&tag.name, input);
                suggestions.push(TagSuggestion {
                    tag: tag.clone(),
                    relevance,
                    match_type: if tag.name.to_lowercase().starts_with(&input_lower) {
                        MatchType::Prefix
                    } else {
                        MatchType::Contains
                    },
                });
            }
        }
        
        suggestions.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        suggestions.truncate(10);
        suggestions
    }
}
```

**智能标签建议**:
```rust
// services/tag_suggester.rs
pub struct TagSuggester {
    db_pool: SqlitePool,
    nlp_processor: NlpProcessor,
    tag_hierarchy: TagHierarchy,
}

impl TagSuggester {
    pub async fn suggest_tags_for_note(&self, note: &Note) -> Result<Vec<TagSuggestion>> {
        let mut suggestions = Vec::new();
        
        // 1. 基于内容的关键词提取
        let keywords = self.nlp_processor.extract_keywords(&note.content)?;
        for keyword in keywords {
            if let Some(tag) = self.find_matching_tag(&keyword).await? {
                suggestions.push(TagSuggestion {
                    tag,
                    relevance: 0.8,
                    reason: SuggestionReason::KeywordMatch(keyword),
                });
            }
        }
        
        // 2. 基于相似笔记的标签
        let similar_notes = self.find_similar_notes(note).await?;
        for similar_note in similar_notes {
            for tag in &similar_note.tags {
                if !note.tags.contains(tag) {
                    suggestions.push(TagSuggestion {
                        tag: tag.clone(),
                        relevance: 0.6 * similar_note.similarity,
                        reason: SuggestionReason::SimilarNote(similar_note.id.clone()),
                    });
                }
            }
        }
        
        // 3. 基于文件路径的推断
        if let Some(category) = self.infer_category_from_path(&note.path)? {
            suggestions.push(TagSuggestion {
                tag: category,
                relevance: 0.7,
                reason: SuggestionReason::PathInference,
            });
        }
        
        // 4. 基于时间模式的推荐
        let temporal_tags = self.suggest_temporal_tags(note).await?;
        suggestions.extend(temporal_tags);
        
        // 去重和排序
        self.deduplicate_and_rank(suggestions)
    }
    
    pub async fn suggest_tag_cleanup(&self) -> Result<Vec<TagCleanupSuggestion>> {
        let mut suggestions = Vec::new();
        
        // 查找未使用的标签
        let unused_tags = self.find_unused_tags().await?;
        for tag in unused_tags {
            suggestions.push(TagCleanupSuggestion {
                tag,
                action: CleanupAction::Delete,
                reason: "标签未被使用".to_string(),
            });
        }
        
        // 查找重复的标签
        let duplicate_groups = self.find_duplicate_tags().await?;
        for group in duplicate_groups {
            suggestions.push(TagCleanupSuggestion {
                tag: group.canonical_tag,
                action: CleanupAction::Merge(group.duplicates),
                reason: "发现重复标签".to_string(),
            });
        }
        
        // 查找可以合并的相似标签
        let similar_groups = self.find_similar_tags().await?;
        for group in similar_groups {
            suggestions.push(TagCleanupSuggestion {
                tag: group.primary_tag,
                action: CleanupAction::MergeSimilar(group.similar_tags),
                reason: format!("相似度: {:.2}", group.similarity),
            });
        }
        
        Ok(suggestions)
    }
}
```

**标签可视化组件**:
```typescript
// components/TagCloud/index.tsx
export interface TagCloudProps {
  tags: TagWithCount[];
  maxTags?: number;
  colorScheme?: 'blue' | 'green' | 'purple' | 'rainbow';
  layout?: 'cloud' | 'list' | 'hierarchy';
  onTagClick?: (tag: Tag) => void;
  onTagHover?: (tag: Tag | null) => void;
}

export const TagCloud: React.FC<TagCloudProps> = ({
  tags,
  maxTags = 100,
  colorScheme = 'blue',
  layout = 'cloud',
  onTagClick,
  onTagHover,
}) => {
  const [dimensions, setDimensions] = useState({ width: 600, height: 400 });
  const containerRef = useRef<HTMLDivElement>(null);
  
  // 计算标签大小和颜色
  const processedTags = useMemo(() => {
    const maxCount = Math.max(...tags.map(t => t.count));
    const minCount = Math.min(...tags.map(t => t.count));
    
    return tags.slice(0, maxTags).map(tag => ({
      ...tag,
      size: calculateTagSize(tag.count, minCount, maxCount),
      color: calculateTagColor(tag.count, minCount, maxCount, colorScheme),
      weight: tag.count / maxCount,
    }));
  }, [tags, maxTags, colorScheme]);
  
  if (layout === 'hierarchy') {
    return <TagHierarchyView tags={processedTags} onTagClick={onTagClick} />;
  }
  
  if (layout === 'list') {
    return <TagListView tags={processedTags} onTagClick={onTagClick} />;
  }
  
  // 词云布局
  return (
    <div 
      ref={containerRef}
      className="tag-cloud"
      style={{ width: dimensions.width, height: dimensions.height }}
    >
      <svg width="100%" height="100%">
        {processedTags.map((tag, index) => (
          <text
            key={tag.id}
            x={tag.x}
            y={tag.y}
            fontSize={tag.size}
            fill={tag.color}
            textAnchor="middle"
            dominantBaseline="middle"
            style={{ cursor: 'pointer' }}
            onClick={() => onTagClick?.(tag)}
            onMouseEnter={() => onTagHover?.(tag)}
            onMouseLeave={() => onTagHover?.(null)}
          >
            {tag.name}
          </text>
        ))}
      </svg>
    </div>
  );
};
```

**验收标准**:
- [ ] 标签建议准确率 > 70%
- [ ] 层次化标签导航流畅
- [ ] 标签搜索响应时间 < 50ms
- [ ] 标签云渲染性能良好 (1000+ 标签)
- [ ] 批量标签操作正确

### 4.4 关联性分析

**任务描述**: 实现智能的内容关联分析和推荐

**相似度计算算法**:
```rust
// services/similarity_engine.rs
pub struct SimilarityEngine {
    tfidf_vectorizer: TfIdfVectorizer,
    embeddings_cache: LruCache<String, Vec<f32>>,
}

impl SimilarityEngine {
    pub async fn calculate_content_similarity(
        &mut self, 
        note1: &Note, 
        note2: &Note
    ) -> Result<f64> {
        // 1. TF-IDF 向量相似度
        let tfidf_sim = self.calculate_tfidf_similarity(note1, note2)?;
        
        // 2. 标签重叠度
        let tag_sim = self.calculate_tag_similarity(&note1.tags, &note2.tags);
        
        // 3. 链接关系相似度
        let link_sim = self.calculate_link_similarity(note1, note2).await?;
        
        // 4. 结构相似度 (标题、章节等)
        let struct_sim = self.calculate_structural_similarity(note1, note2)?;
        
        // 加权平均
        let final_similarity = 
            tfidf_sim * 0.4 +
            tag_sim * 0.3 +
            link_sim * 0.2 +
            struct_sim * 0.1;
        
        Ok(final_similarity)
    }
    
    fn calculate_tfidf_similarity(&mut self, note1: &Note, note2: &Note) -> Result<f64> {
        let doc1 = self.preprocess_content(&note1.content);
        let doc2 = self.preprocess_content(&note2.content);
        
        let vec1 = self.tfidf_vectorizer.transform(&doc1)?;
        let vec2 = self.tfidf_vectorizer.transform(&doc2)?;
        
        Ok(cosine_similarity(&vec1, &vec2))
    }
    
    fn calculate_tag_similarity(&self, tags1: &[String], tags2: &[String]) -> f64 {
        let set1: HashSet<_> = tags1.iter().collect();
        let set2: HashSet<_> = tags2.iter().collect();
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
    
    async fn calculate_link_similarity(&self, note1: &Note, note2: &Note) -> Result<f64> {
        let links1 = self.get_note_links(&note1.id).await?;
        let links2 = self.get_note_links(&note2.id).await?;
        
        let targets1: HashSet<_> = links1.iter().map(|l| &l.target).collect();
        let targets2: HashSet<_> = links2.iter().map(|l| &l.target).collect();
        
        let common_targets = targets1.intersection(&targets2).count();
        let total_targets = targets1.union(&targets2).count();
        
        if total_targets == 0 {
            0.0
        } else {
            common_targets as f64 / total_targets as f64
        }
    }
    
    pub async fn find_knowledge_clusters(&self) -> Result<Vec<KnowledgeCluster>> {
        let all_notes = self.get_all_notes().await?;
        let similarity_matrix = self.build_similarity_matrix(&all_notes).await?;
        
        // 使用社区检测算法识别知识聚类
        let communities = self.detect_communities(&similarity_matrix)?;
        
        let mut clusters = Vec::new();
        for community in communities {
            let notes = community.iter()
                .map(|&idx| all_notes[idx].clone())
                .collect();
            
            let cluster = KnowledgeCluster {
                id: Uuid::new_v4().to_string(),
                name: self.generate_cluster_name(&notes),
                notes,
                cohesion: self.calculate_cluster_cohesion(&community, &similarity_matrix),
                topics: self.extract_cluster_topics(&notes),
            };
            
            clusters.push(cluster);
        }
        
        clusters.sort_by(|a, b| b.cohesion.partial_cmp(&a.cohesion).unwrap());
        Ok(clusters)
    }
}
```

**推荐系统**:
```rust
// services/recommendation_engine.rs
pub struct RecommendationEngine {
    similarity_engine: SimilarityEngine,
    graph_analyzer: GraphAnalyzer,
    user_behavior: UserBehaviorTracker,
}

impl RecommendationEngine {
    pub async fn get_recommendations_for_note(
        &self, 
        note_id: &str, 
        recommendation_type: RecommendationType
    ) -> Result<Vec<Recommendation>> {
        match recommendation_type {
            RecommendationType::Similar => {
                self.get_similar_notes(note_id).await
            },
            RecommendationType::Related => {
                self.get_related_notes(note_id).await
            },
            RecommendationType::Complementary => {
                self.get_complementary_notes(note_id).await
            },
            RecommendationType::Serendipitous => {
                self.get_serendipitous_notes(note_id).await
            },
        }
    }
    
    async fn get_similar_notes(&self, note_id: &str) -> Result<Vec<Recommendation>> {
        let note = self.get_note(note_id).await?;
        let candidates = self.get_candidate_notes(&note).await?;
        
        let mut recommendations = Vec::new();
        for candidate in candidates {
            let similarity = self.similarity_engine
                .calculate_content_similarity(&note, &candidate).await?;
            
            if similarity > 0.3 {
                recommendations.push(Recommendation {
                    note: candidate,
                    score: similarity,
                    reason: RecommendationReason::ContentSimilarity(similarity),
                    confidence: similarity,
                });
            }
        }
        
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(10);
        Ok(recommendations)
    }
    
    async fn get_complementary_notes(&self, note_id: &str) -> Result<Vec<Recommendation>> {
        let note = self.get_note(note_id).await?;
        
        // 查找引用但未链接的相关内容
        let referenced_concepts = self.extract_concepts(&note.content)?;
        let mut recommendations = Vec::new();
        
        for concept in referenced_concepts {
            let related_notes = self.find_notes_about_concept(&concept).await?;
            
            for related_note in related_notes {
                if related_note.id != note.id && 
                   !self.are_notes_linked(&note.id, &related_note.id).await? {
                    
                    let relevance = self.calculate_concept_relevance(&concept, &related_note)?;
                    
                    recommendations.push(Recommendation {
                        note: related_note,
                        score: relevance,
                        reason: RecommendationReason::ConceptualGap(concept.clone()),
                        confidence: relevance * 0.8, // 稍微降低置信度
                    });
                }
            }
        }
        
        self.deduplicate_and_rank(recommendations)
    }
    
    pub async fn analyze_knowledge_gaps(&self) -> Result<Vec<KnowledgeGap>> {
        let all_notes = self.get_all_notes().await?;
        let link_graph = self.build_link_graph(&all_notes).await?;
        
        let mut gaps = Vec::new();
        
        // 1. 查找孤立的笔记群集
        let components = self.graph_analyzer.find_disconnected_components(&link_graph)?;
        for component in components {
            if component.len() > 1 {
                gaps.push(KnowledgeGap {
                    gap_type: GapType::DisconnectedCluster,
                    notes: component,
                    severity: self.calculate_gap_severity(&component),
                    suggested_connections: self.suggest_bridge_notes(&component).await?,
                });
            }
        }
        
        // 2. 查找概念上相关但未链接的笔记
        let concept_gaps = self.find_conceptual_gaps(&all_notes).await?;
        gaps.extend(concept_gaps);
        
        // 3. 查找缺失的中间概念
        let missing_concepts = self.identify_missing_concepts(&all_notes).await?;
        gaps.extend(missing_concepts);
        
        gaps.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
        Ok(gaps)
    }
}
```

**验收标准**:
- [ ] 相似笔记推荐准确率 > 60%
- [ ] 推荐计算时间 < 200ms
- [ ] 知识聚类结果合理
- [ ] 知识缺口识别有效
- [ ] 推荐多样性和新颖性平衡

## 集成测试

### 4.5 端到端功能测试

**测试场景**:
1. **链接生命周期测试**
   - 创建新链接自动索引
   - 重命名文件更新所有引用
   - 删除文件处理断链
   - 批量重构操作

2. **图谱交互测试**
   - 大规模图谱渲染性能
   - 实时搜索和过滤
   - 多种布局算法切换
   - 节点详情和导航

3. **智能推荐测试**
   - 不同类型内容的推荐质量
   - 推荐系统冷启动处理
   - 用户反馈学习效果
   - 多语言内容支持

## 里程碑和验收

### 第1周里程碑
- 双向链接解析和索引系统
- 基础知识图谱可视化
- 标签系统核心功能

### 第2周里程碑
- 图谱交互和布局算法
- 智能标签建议功能
- 相似度计算引擎

### 第3周里程碑
- 推荐系统实现
- 知识聚类分析
- 前端组件集成

### 最终验收标准
- [ ] 所有链接功能正常工作
- [ ] 图谱可视化性能达标
- [ ] 推荐准确率满足要求
- [ ] 用户界面流畅友好
- [ ] 大数据量场景稳定

## 下一阶段准备

### Phase 5 准备工作
- 确定发布平台API集成方案
- 设计内容转换和适配规则
- 准备插件系统架构设计

---

**创建时间**: 2025-07-01  
**负责人**: 全栈开发团队  
**状态**: 规划中  
**依赖**: Phase 3 用户界面层完成