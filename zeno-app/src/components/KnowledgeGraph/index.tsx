import React, { useRef, useEffect, useState, useCallback, useMemo } from 'react';
import cytoscape, { Core, ElementDefinition } from 'cytoscape';
import cola from 'cytoscape-cola';
import fcose from 'cytoscape-fcose';
import { Search, Filter, RefreshCw, ZoomIn, ZoomOut, Home, Download } from 'lucide-react';

// 注册布局算法
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
    level?: number;
    fullName?: string;
    noteCount?: number;
  };
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: 'link' | 'tag' | 'similarity' | 'hierarchy';
  weight: number;
  metadata: {
    linkText?: string;
    context?: string;
    fullTagName?: string;
    relationshipType?: string;
  };
}

export interface GraphFilters {
  showTags: boolean;
  showCategories: boolean;
  minLinkWeight: number;
  nodeTypes: ('note' | 'tag' | 'category')[];
  searchQuery: string;
}

export type LayoutName = 'fcose' | 'cola' | 'cose' | 'circle' | 'grid' | 'breadthfirst';

export interface KnowledgeGraphProps {
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNodeId?: string;
  onNodeSelect?: (nodeId: string) => void;
  onNodeDoubleClick?: (nodeId: string) => void;
  layout?: LayoutName;
  style?: React.CSSProperties;
  className?: string;
}

const DEFAULT_FILTERS: GraphFilters = {
  showTags: true,
  showCategories: true,
  minLinkWeight: 0,
  nodeTypes: ['note', 'tag', 'category'],
  searchQuery: '',
};

const LAYOUT_CONFIGS: Record<LayoutName, any> = {
  fcose: {
    name: 'fcose',
    animate: true,
    animationDuration: 1000,
    fit: true,
    padding: 30,
    nodeDimensionsIncludeLabels: false,
    uniformNodeDimensions: false,
    packComponents: true,
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
  cola: {
    name: 'cola',
    animate: true,
    maxSimulationTime: 2000,
    edgeLength: 80,
    nodeSpacing: 30,
    flow: { axis: 'y', minSeparation: 30 },
    fit: true,
    padding: 30,
  },
  cose: {
    name: 'cose',
    animate: true,
    animationDuration: 1000,
    nodeRepulsion: 400000,
    nodeOverlap: 10,
    idealEdgeLength: 10,
    edgeElasticity: 100,
    nestingFactor: 5,
    gravity: 250,
    numIter: 100,
    fit: true,
    padding: 30,
  },
  circle: {
    name: 'circle',
    fit: true,
    padding: 30,
    animate: true,
    animationDuration: 1000,
  },
  grid: {
    name: 'grid',
    fit: true,
    padding: 30,
    animate: true,
    animationDuration: 1000,
  },
  breadthfirst: {
    name: 'breadthfirst',
    fit: true,
    padding: 30,
    animate: true,
    animationDuration: 1000,
    directed: true,
    spacingFactor: 1.75,
  },
};

const CYTOSCAPE_STYLE: any = [
  {
    selector: 'node',
    style: {
      'width': 'data(size)',
      'height': 'data(size)',
      'label': 'data(label)',
      'text-valign': 'center',
      'text-halign': 'center',
      'font-size': '11px',
      'font-weight': '600',
      'font-family': '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      'text-outline-width': 2,
      'text-outline-color': '#FFFFFF',
      'text-outline-opacity': 0.9,
      'text-max-width': '80px',
      'text-wrap': 'wrap',
      'text-overflow-wrap': 'anywhere',
      'border-width': 1,
      'border-color': '#FFFFFF',
      'border-opacity': 0.5,
    }
  },
  {
    selector: 'node[type="note"]',
    style: {
      'background-color': '#2563EB',
      'color': '#FFFFFF',
      'shape': 'roundrectangle',
      'box-shadow': '0 2px 4px rgba(0,0,0,0.1)',
    }
  },
  {
    selector: 'node[type="tag"]',
    style: {
      'background-color': '#059669',
      'color': '#FFFFFF',
      'shape': 'triangle',
      'box-shadow': '0 2px 4px rgba(0,0,0,0.1)',
    }
  },
  {
    selector: 'node[type="category"]',
    style: {
      'background-color': '#F59E0B',
      'color': '#FFFFFF',
      'shape': 'diamond',
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
      'opacity': 0.6,
    }
  },
  {
    selector: 'edge[type="similarity"]',
    style: {
      'line-style': 'dashed',
      'line-color': '#8B5CF6',
      'target-arrow-color': '#8B5CF6',
    }
  },
  {
    selector: 'edge[type="tag"]',
    style: {
      'line-color': '#10B981',
      'target-arrow-color': '#10B981',
    }
  },
  {
    selector: '.highlighted',
    style: {
      'background-color': '#EF4444',
      'line-color': '#EF4444',
      'target-arrow-color': '#EF4444',
      'opacity': 1,
    }
  },
  {
    selector: '.selected',
    style: {
      'border-width': 3,
      'border-color': '#FBBF24',
      'border-opacity': 1,
    }
  },
  {
    selector: '.faded',
    style: {
      'opacity': 0.3,
    }
  }
];

export const KnowledgeGraph: React.FC<KnowledgeGraphProps> = ({
  nodes,
  edges,
  selectedNodeId,
  onNodeSelect,
  onNodeDoubleClick,
  layout = 'fcose',
  style = {},
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<Core | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [filters, setFilters] = useState<GraphFilters>(DEFAULT_FILTERS);
  const [currentLayout, setCurrentLayout] = useState<LayoutName>(layout);
  const [tooltipData, setTooltipData] = useState<{
    node: GraphNode | null;
    position: { x: number; y: number };
  }>({ node: null, position: { x: 0, y: 0 } });
  const [showFilters, setShowFilters] = useState(false);

  // 过滤节点和边
  const filteredData = useMemo(() => {
    const filteredNodes = nodes.filter(node => 
      filters.nodeTypes.includes(node.type) &&
      (filters.searchQuery === '' || 
       node.label.toLowerCase().includes(filters.searchQuery.toLowerCase()) ||
       node.metadata.tags.some(tag => 
         tag.toLowerCase().includes(filters.searchQuery.toLowerCase())
       ))
    );

    const nodeIds = new Set(filteredNodes.map(n => n.id));
    const filteredEdges = edges.filter(edge =>
      edge.weight >= filters.minLinkWeight &&
      nodeIds.has(edge.source) &&
      nodeIds.has(edge.target)
    );

    return { nodes: filteredNodes, edges: filteredEdges };
  }, [nodes, edges, filters]);

  // 初始化 Cytoscape
  useEffect(() => {
    if (!containerRef.current || cyRef.current) return;

    const cy = cytoscape({
      container: containerRef.current,
      elements: [],
      style: CYTOSCAPE_STYLE,
      layout: LAYOUT_CONFIGS[currentLayout],
      wheelSensitivity: 0.5,
      minZoom: 0.1,
      maxZoom: 5,
    });

    cyRef.current = cy;

    // 绑定事件
    cy.on('tap', 'node', (event) => {
      const nodeId = event.target.id();
      onNodeSelect?.(nodeId);
      highlightNeighborhood(nodeId);
    });

    cy.on('dbltap', 'node', (event) => {
      const nodeId = event.target.id();
      onNodeDoubleClick?.(nodeId);
    });

    // 鼠标悬停显示工具提示
    cy.on('mouseover', 'node', (event) => {
      const node = event.target;
      const nodeData = node.data() as GraphNode;
      const renderedPosition = node.renderedPosition();
      
      setTooltipData({
        node: nodeData,
        position: { x: renderedPosition.x, y: renderedPosition.y }
      });
    });

    cy.on('mouseout', 'node', () => {
      setTooltipData({ node: null, position: { x: 0, y: 0 } });
    });

    // 空白区域点击取消选择
    cy.on('tap', (event) => {
      if (event.target === cy) {
        cy.elements().removeClass('selected highlighted faded');
        setTooltipData({ node: null, position: { x: 0, y: 0 } });
      }
    });

    setIsLoading(false);

    return () => {
      if (cyRef.current) {
        cyRef.current.destroy();
        cyRef.current = null;
      }
    };
  }, [onNodeSelect, onNodeDoubleClick]);

  // 更新图谱数据
  useEffect(() => {
    if (!cyRef.current) return;

    const elements: ElementDefinition[] = [
      ...filteredData.nodes.map(node => ({
        data: {
          id: node.id,
          label: node.label,
          type: node.type,
          size: Math.max(20, Math.min(60, node.size)),
          color: node.color,
          ...node.metadata,
        }
      })),
      ...filteredData.edges.map(edge => ({
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
    cyRef.current.layout(LAYOUT_CONFIGS[currentLayout]).run();
  }, [filteredData, currentLayout]);

  // 选中节点效果
  useEffect(() => {
    if (!cyRef.current || !selectedNodeId) return;

    cyRef.current.elements().removeClass('selected');
    const selectedNode = cyRef.current.$(`#${selectedNodeId}`);
    if (selectedNode.length > 0) {
      selectedNode.addClass('selected');
      // 居中显示选中节点
      cyRef.current.center(selectedNode);
    }
  }, [selectedNodeId]);

  // 高亮相邻节点
  const highlightNeighborhood = useCallback((nodeId: string) => {
    if (!cyRef.current) return;

    const cy = cyRef.current;
    const selectedNode = cy.$(`#${nodeId}`);
    
    // 清除之前的样式
    cy.elements().removeClass('highlighted selected faded');
    
    // 选中当前节点
    selectedNode.addClass('selected');
    
    // 高亮相邻节点和边
    const neighborhood = selectedNode.neighborhood();
    neighborhood.addClass('highlighted');
    
    // 淡化其他节点
    const others = cy.elements().not(selectedNode).not(neighborhood);
    others.addClass('faded');
  }, []);

  // 控制函数
  const handleZoomIn = useCallback(() => {
    if (cyRef.current) {
      cyRef.current.zoom(cyRef.current.zoom() * 1.2);
    }
  }, []);

  const handleZoomOut = useCallback(() => {
    if (cyRef.current) {
      cyRef.current.zoom(cyRef.current.zoom() / 1.2);
    }
  }, []);

  const handleFitView = useCallback(() => {
    if (cyRef.current) {
      cyRef.current.fit();
    }
  }, []);

  const handleRefresh = useCallback(() => {
    if (cyRef.current) {
      cyRef.current.layout(LAYOUT_CONFIGS[currentLayout]).run();
    }
  }, [currentLayout]);

  const handleExport = useCallback(() => {
    if (cyRef.current) {
      const png64 = cyRef.current.png({ scale: 2, full: true });
      const link = document.createElement('a');
      link.download = 'knowledge-graph.png';
      link.href = png64;
      link.click();
    }
  }, []);

  const updateFilters = useCallback((updates: Partial<GraphFilters>) => {
    setFilters(prev => ({ ...prev, ...updates }));
  }, []);

  // 点击外部关闭过滤器
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as Element;
      if (showFilters && !target.closest('.filter-section')) {
        setShowFilters(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showFilters]);

  return (
    <div className={`knowledge-graph ${className}`} style={style}>
      {/* 控制面板 */}
      <div className="graph-controls">
        {/* 搜索栏 */}
        <div className="search-section">
          <div className="search-input">
            <Search size={16} />
            <input
              type="text"
              placeholder="搜索节点..."
              value={filters.searchQuery}
              onChange={(e) => updateFilters({ searchQuery: e.target.value })}
            />
          </div>
        </div>

        {/* 布局选择 */}
        <div className="layout-section">
          <label>布局算法:</label>
          <select
            value={currentLayout}
            onChange={(e) => setCurrentLayout(e.target.value as LayoutName)}
          >
            <option value="fcose">力导向 (推荐)</option>
            <option value="cola">COLA</option>
            <option value="cose">CoSE</option>
            <option value="circle">圆形</option>
            <option value="grid">网格</option>
            <option value="breadthfirst">层次</option>
          </select>
        </div>

        {/* 过滤器 */}
        <div className="filter-section">
          <button 
            className="filter-toggle"
            onClick={() => setShowFilters(!showFilters)}
          >
            <Filter size={16} />
            过滤器
          </button>
          {showFilters && (
            <div className="filter-panel">
            <div className="filter-group">
              <label>节点类型:</label>
              <div className="checkbox-group">
                <label>
                  <input
                    type="checkbox"
                    checked={filters.nodeTypes.includes('note')}
                    onChange={(e) => {
                      const nodeTypes = e.target.checked 
                        ? [...filters.nodeTypes, 'note' as const]
                        : filters.nodeTypes.filter(t => t !== 'note');
                      updateFilters({ nodeTypes });
                    }}
                  />
                  笔记
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={filters.nodeTypes.includes('tag')}
                    onChange={(e) => {
                      const nodeTypes = e.target.checked 
                        ? [...filters.nodeTypes, 'tag' as const]
                        : filters.nodeTypes.filter(t => t !== 'tag');
                      updateFilters({ nodeTypes });
                    }}
                  />
                  标签
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={filters.nodeTypes.includes('category')}
                    onChange={(e) => {
                      const nodeTypes = e.target.checked 
                        ? [...filters.nodeTypes, 'category' as const]
                        : filters.nodeTypes.filter(t => t !== 'category');
                      updateFilters({ nodeTypes });
                    }}
                  />
                  分类
                </label>
              </div>
            </div>
            
            <div className="filter-group">
              <label>最小链接权重: {filters.minLinkWeight}</label>
              <input
                type="range"
                min="0"
                max="5"
                step="0.5"
                value={filters.minLinkWeight}
                onChange={(e) => updateFilters({ minLinkWeight: parseFloat(e.target.value) })}
              />
            </div>
          </div>
          )}
        </div>

        {/* 工具按钮 */}
        <div className="tool-buttons">
          <button onClick={handleZoomIn} title="放大">
            <ZoomIn size={16} />
          </button>
          <button onClick={handleZoomOut} title="缩小">
            <ZoomOut size={16} />
          </button>
          <button onClick={handleFitView} title="适应窗口">
            <Home size={16} />
          </button>
          <button onClick={handleRefresh} title="刷新布局">
            <RefreshCw size={16} />
          </button>
          <button onClick={handleExport} title="导出图片">
            <Download size={16} />
          </button>
        </div>
      </div>

      {/* 图谱容器 */}
      <div 
        ref={containerRef} 
        className="graph-container"
      />

      {/* 加载状态 */}
      {isLoading && (
        <div className="graph-loading">
          <div className="spinner" />
          <span>加载图谱中...</span>
        </div>
      )}

      {/* 工具提示 */}
      {tooltipData.node && (
        <div 
          className="node-tooltip"
          style={{
            left: tooltipData.position.x + 10,
            top: tooltipData.position.y - 50,
          }}
        >
          <div className="tooltip-title">{tooltipData.node.label}</div>
          <div className="tooltip-meta">
            <div>类型: {tooltipData.node.type}</div>
            {(tooltipData.node as any).wordCount !== undefined && (
              <div>字数: {(tooltipData.node as any).wordCount}</div>
            )}
            {(tooltipData.node as any).tags && (tooltipData.node as any).tags.length > 0 && (
              <div>标签: {(tooltipData.node as any).tags.join(', ')}</div>
            )}
          </div>
        </div>
      )}

      {/* 样式 */}
      <style>{`
        .knowledge-graph {
          position: relative;
          width: 100%;
          height: 100%;
          display: flex;
          flex-direction: column;
          background: var(--bg-primary, #ffffff);
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .graph-controls {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1rem;
          background: var(--bg-secondary, #f8fafc);
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          flex-wrap: wrap;
        }

        .search-section .search-input {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 6px;
          min-width: 200px;
        }

        .search-input input {
          border: none;
          outline: none;
          flex: 1;
          font-size: 14px;
        }

        .layout-section {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .layout-section select {
          padding: 0.5rem;
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 6px;
          background: var(--bg-primary, white);
        }

        .filter-section {
          position: relative;
        }

        .filter-toggle {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 6px;
          cursor: pointer;
        }

        .filter-panel {
          position: absolute;
          top: 100%;
          left: 0;
          z-index: 10;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 6px;
          padding: 1rem;
          min-width: 250px;
          margin-top: 4px;
          box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
          animation: fadeIn 0.2s ease-out;
        }

        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateY(-10px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        .filter-group {
          margin-bottom: 1rem;
        }

        .filter-group:last-child {
          margin-bottom: 0;
        }

        .filter-group label {
          font-weight: 500;
          margin-bottom: 0.5rem;
          display: block;
        }

        .checkbox-group {
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
        }

        .checkbox-group label {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-weight: normal;
        }

        .tool-buttons {
          display: flex;
          gap: 0.5rem;
        }

        .tool-buttons button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .tool-buttons button:hover {
          background: var(--bg-tertiary, #f1f5f9);
        }

        .graph-container {
          flex: 1;
          position: relative;
          min-height: 400px;
          background: var(--bg-primary, #ffffff);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.5rem;
          margin: 0.5rem;
        }

        .graph-loading {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 1rem;
          color: var(--text-secondary, #64748b);
        }

        .spinner {
          width: 24px;
          height: 24px;
          border: 2px solid var(--border-primary, #e2e8f0);
          border-top: 2px solid var(--accent-primary, #3b82f6);
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .node-tooltip {
          position: absolute;
          z-index: 1000;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 6px;
          padding: 0.75rem;
          box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
          pointer-events: none;
          max-width: 250px;
        }

        .tooltip-title {
          font-weight: 600;
          margin-bottom: 0.5rem;
          color: var(--text-primary, #1e293b);
        }

        .tooltip-meta {
          font-size: 12px;
          color: var(--text-secondary, #64748b);
          line-height: 1.4;
        }

        .tooltip-meta div {
          margin-bottom: 0.25rem;
        }
      `}</style>
    </div>
  );
};

export default KnowledgeGraph;