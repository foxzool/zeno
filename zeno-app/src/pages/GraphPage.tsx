import React, { useState, useCallback, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import KnowledgeGraph from '../components/KnowledgeGraph';
import TagBrowser from '../components/TagBrowser';
import { useKnowledgeGraph } from '../hooks/useKnowledgeGraph';
import { HierarchicalTag } from '../hooks/useTagHierarchy';
import { 
  RefreshCw, 
  AlertTriangle, 
  TrendingUp, 
  Link as LinkIcon,
  FileText,
  Tag,
  Zap,
  Sidebar,
  X
} from 'lucide-react';

const GraphPage: React.FC = () => {
  const navigate = useNavigate();
  const {
    nodes,
    edges,
    loading,
    error,
    stats,
    buildGraphFromNotes,
    rebuildIndex,
    getNodeDetails,
  } = useKnowledgeGraph();

  const [selectedNodeId, setSelectedNodeId] = useState<string>();
  const [sidebarContent, setSidebarContent] = useState<any>(null);
  const [showStats, setShowStats] = useState(false);
  const [showTagBrowser, setShowTagBrowser] = useState(false);
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  // 移除未使用的状态变量，直接使用 displayNodes 和 displayEdges

  // 根据选中的标签过滤节点和边
  const { displayNodes, displayEdges } = useMemo(() => {
    if (selectedTags.length === 0) {
      return { displayNodes: nodes, displayEdges: edges };
    }

    // 找到所有相关的标签节点 ID
    const selectedTagIds = selectedTags.map(tag => `tag-${tag}`);
    
    // 找到与选中标签相关的笔记节点
    const relatedNoteIds = new Set<string>();
    edges.forEach(edge => {
      if (edge.type === 'tag' && selectedTagIds.includes(edge.target)) {
        relatedNoteIds.add(edge.source);
      }
    });

    // 过滤节点：保留选中的标签节点和相关的笔记节点
    const filteredNodes = nodes.filter(node => {
      if (selectedTagIds.includes(node.id)) return true; // 选中的标签节点
      if (node.type === 'note' && relatedNoteIds.has(node.id)) return true; // 相关的笔记节点
      return false;
    });

    // 过滤边：只保留与过滤后节点相关的边
    const filteredNodeIds = new Set(filteredNodes.map(n => n.id));
    const filteredEdges = edges.filter(edge => 
      filteredNodeIds.has(edge.source) && filteredNodeIds.has(edge.target)
    );

    return { displayNodes: filteredNodes, displayEdges: filteredEdges };
  }, [nodes, edges, selectedTags]);

  // 处理标签选择
  const handleTagSelect = useCallback((tag: HierarchicalTag) => {
    setSelectedTags(prev => {
      const isSelected = prev.includes(tag.name);
      if (isSelected) {
        return prev.filter(t => t !== tag.name);
      } else {
        return [...prev, tag.name];
      }
    });
  }, []);

  // 清除标签过滤
  const handleClearTagFilter = useCallback(() => {
    setSelectedTags([]);
  }, []);

  // 处理节点选择
  const handleNodeSelect = useCallback(async (nodeId: string) => {
    setSelectedNodeId(nodeId);
    
    // 获取节点详细信息
    const details = await getNodeDetails(nodeId);
    setSidebarContent(details);
  }, [getNodeDetails]);

  // 处理节点双击 - 导航到编辑器
  const handleNodeDoubleClick = useCallback((nodeId: string) => {
    if (!nodeId.startsWith('tag-')) {
      // 找到对应的笔记节点
      const node = displayNodes.find(n => n.id === nodeId);
      if (node) {
        navigate(`/editor?file=${encodeURIComponent(node.metadata.path)}`);
      }
    }
  }, [displayNodes, navigate]);

  // 刷新图谱
  const handleRefresh = useCallback(() => {
    buildGraphFromNotes();
  }, [buildGraphFromNotes]);

  // 重建索引
  const handleRebuildIndex = useCallback(() => {
    rebuildIndex();
  }, [rebuildIndex]);

  if (loading) {
    return (
      <div className="graph-page">
        <div className="loading-container">
          <div className="spinner" />
          <p>正在构建知识图谱...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="graph-page">
        <div className="error-container">
          <AlertTriangle size={48} />
          <h2>加载失败</h2>
          <p>{error}</p>
          <button onClick={handleRefresh}>重试</button>
        </div>
      </div>
    );
  }

  return (
    <div className="graph-page">
      {/* 页面头部 */}
      <div className="page-header">
        <div className="header-content">
          <h1 className="page-title">知识图谱</h1>
          <p className="page-description">
            探索笔记之间的关联，发现知识的深层联系
          </p>
        </div>
        
        <div className="header-actions">
          <button 
            className="stats-button"
            onClick={() => setShowStats(!showStats)}
          >
            <TrendingUp size={16} />
            统计信息
          </button>
          <button 
            className={`tag-browser-button ${showTagBrowser ? 'active' : ''}`}
            onClick={() => setShowTagBrowser(!showTagBrowser)}
            title="标签浏览器"
          >
            <Sidebar size={16} />
            标签浏览器
          </button>
          <button 
            className="refresh-button"
            onClick={handleRefresh}
            title="刷新图谱"
          >
            <RefreshCw size={16} />
            刷新
          </button>
          <button 
            className="rebuild-button"
            onClick={handleRebuildIndex}
            title="重建索引"
          >
            <Zap size={16} />
            重建索引
          </button>
        </div>
      </div>

      {/* 统计信息面板 */}
      {showStats && stats && (
        <div className="stats-panel">
          <div className="stat-card">
            <FileText size={20} />
            <div className="stat-content">
              <div className="stat-value">{stats.total_notes}</div>
              <div className="stat-label">笔记总数</div>
            </div>
          </div>
          <div className="stat-card">
            <LinkIcon size={20} />
            <div className="stat-content">
              <div className="stat-value">{stats.total_links}</div>
              <div className="stat-label">链接总数</div>
            </div>
          </div>
          <div className="stat-card">
            <AlertTriangle size={20} />
            <div className="stat-content">
              <div className="stat-value">{stats.total_broken_links}</div>
              <div className="stat-label">断链数量</div>
            </div>
          </div>
          <div className="stat-card">
            <Tag size={20} />
            <div className="stat-content">
              <div className="stat-value">{stats.orphaned_notes}</div>
              <div className="stat-label">孤立笔记</div>
            </div>
          </div>
        </div>
      )}

      {/* 标签过滤状态 */}
      {selectedTags.length > 0 && (
        <div className="filter-status">
          <div className="filter-content">
            <Tag size={16} />
            <span>已选择 {selectedTags.length} 个标签过滤:</span>
            <div className="selected-tags">
              {selectedTags.map(tag => (
                <span key={tag} className="filter-tag">
                  {tag.split('/').pop()}
                  <button 
                    onClick={() => setSelectedTags(prev => prev.filter(t => t !== tag))}
                    className="remove-tag"
                  >
                    <X size={12} />
                  </button>
                </span>
              ))}
            </div>
            <button onClick={handleClearTagFilter} className="clear-filter">
              清除所有
            </button>
          </div>
        </div>
      )}

      {/* 主要内容区域 */}
      <div className="main-content">
        {/* 标签浏览器侧边栏 */}
        {showTagBrowser && (
          <div className="tag-browser-panel">
            <div className="tag-browser-header">
              <h3>标签浏览器</h3>
              <button 
                onClick={() => setShowTagBrowser(false)}
                className="close-button"
              >
                <X size={16} />
              </button>
            </div>
            <div className="tag-browser-content">
              <TagBrowser
                onTagSelect={handleTagSelect}
                selectedTags={selectedTags}
                showStatistics={true}
                showSearch={true}
                className="graph-tag-browser"
              />
            </div>
          </div>
        )}

        {/* 知识图谱 */}
        <div className="graph-container">
          <KnowledgeGraph
            nodes={displayNodes}
            edges={displayEdges}
            selectedNodeId={selectedNodeId}
            onNodeSelect={handleNodeSelect}
            onNodeDoubleClick={handleNodeDoubleClick}
            layout="fcose"
          />
        </div>

        {/* 侧边栏 */}
        {sidebarContent && (
          <div className="sidebar">
            <div className="sidebar-header">
              <h3>节点详情</h3>
              <button onClick={() => setSidebarContent(null)}>×</button>
            </div>
            
            <div className="sidebar-content">
              {sidebarContent.type === 'note' ? (
                <NoteDetails 
                  nodeId={selectedNodeId}
                  details={sidebarContent}
                  onNavigate={(path) => navigate(`/editor?file=${encodeURIComponent(path)}`)}
                />
              ) : (
                <TagDetails 
                  tagName={sidebarContent.name}
                  nodes={displayNodes}
                  edges={displayEdges}
                />
              )}
            </div>
          </div>
        )}
      </div>

      <style>{`
        .graph-page {
          height: 100%;
          display: flex;
          flex-direction: column;
          background: var(--bg-primary, white);
        }

        .page-header {
          padding: 1.5rem;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
        }

        .header-content .page-title {
          margin: 0 0 0.5rem 0;
          font-size: 1.75rem;
          font-weight: 700;
          color: var(--text-primary, #1e293b);
        }

        .page-description {
          margin: 0;
          color: var(--text-secondary, #64748b);
          font-size: 1rem;
        }

        .header-actions {
          display: flex;
          gap: 0.75rem;
        }

        .header-actions button {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.375rem;
          color: var(--text-primary, #1e293b);
          cursor: pointer;
          transition: all 0.2s;
          font-size: 0.875rem;
          font-weight: 500;
        }

        .header-actions button:hover {
          background: var(--bg-tertiary, #f1f5f9);
          border-color: var(--border-secondary, #cbd5e1);
        }

        .tag-browser-button.active {
          background: var(--accent-primary, #3b82f6);
          color: white;
          border-color: var(--accent-primary, #3b82f6);
        }

        .stats-panel {
          display: flex;
          gap: 1rem;
          padding: 1rem 1.5rem;
          background: var(--bg-secondary, #f8fafc);
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .stat-card {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          background: var(--bg-primary, white);
          padding: 1rem;
          border-radius: 0.5rem;
          border: 1px solid var(--border-primary, #e2e8f0);
          flex: 1;
        }

        .stat-content .stat-value {
          font-size: 1.5rem;
          font-weight: 700;
          color: var(--text-primary, #1e293b);
          line-height: 1;
        }

        .stat-content .stat-label {
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
          margin-top: 0.25rem;
        }

        .filter-status {
          padding: 0.75rem 1.5rem;
          background: var(--accent-secondary, #eff6ff);
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .filter-content {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          font-size: 0.875rem;
          color: var(--text-primary, #1e293b);
        }

        .selected-tags {
          display: flex;
          gap: 0.5rem;
          flex-wrap: wrap;
        }

        .filter-tag {
          display: flex;
          align-items: center;
          gap: 0.25rem;
          padding: 0.25rem 0.5rem;
          background: var(--accent-primary, #3b82f6);
          color: white;
          border-radius: 0.75rem;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .remove-tag {
          background: none;
          border: none;
          color: white;
          cursor: pointer;
          padding: 0;
          display: flex;
          align-items: center;
          border-radius: 50%;
          transition: background 0.2s;
        }

        .remove-tag:hover {
          background: rgba(255, 255, 255, 0.2);
        }

        .clear-filter {
          padding: 0.25rem 0.5rem;
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.25rem;
          color: var(--text-secondary, #64748b);
          cursor: pointer;
          font-size: 0.75rem;
          transition: all 0.2s;
        }

        .clear-filter:hover {
          background: var(--bg-tertiary, #f1f5f9);
          color: var(--text-primary, #1e293b);
        }

        .main-content {
          flex: 1;
          display: flex;
          min-height: 0;
        }

        .tag-browser-panel {
          width: 300px;
          background: var(--bg-primary, #ffffff);
          border-right: 1px solid var(--border-primary, #e2e8f0);
          display: flex;
          flex-direction: column;
          min-height: 0;
        }

        .tag-browser-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          background: var(--bg-secondary, #f8fafc);
        }

        .tag-browser-header h3 {
          margin: 0;
          font-size: 1rem;
          font-weight: 600;
          color: var(--text-primary, #1e293b);
        }

        .close-button {
          background: none;
          border: none;
          color: var(--text-secondary, #64748b);
          cursor: pointer;
          padding: 0.25rem;
          border-radius: 0.25rem;
          transition: all 0.2s;
        }

        .close-button:hover {
          background: var(--bg-tertiary, #f1f5f9);
          color: var(--text-primary, #1e293b);
        }

        .tag-browser-content {
          flex: 1;
          min-height: 0;
          overflow: hidden;
        }

        .graph-tag-browser {
          height: 100%;
          border: none;
          border-radius: 0;
        }

        .graph-container {
          flex: 1;
          min-height: 0;
        }

        .sidebar {
          width: 320px;
          background: var(--bg-secondary, #f8fafc);
          border-left: 1px solid var(--border-primary, #e2e8f0);
          display: flex;
          flex-direction: column;
        }

        .sidebar-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .sidebar-header h3 {
          margin: 0;
          font-size: 1rem;
          font-weight: 600;
          color: var(--text-primary, #1e293b);
        }

        .sidebar-header button {
          background: none;
          border: none;
          font-size: 1.25rem;
          color: var(--text-secondary, #64748b);
          cursor: pointer;
          padding: 0.25rem;
        }

        .sidebar-content {
          flex: 1;
          padding: 1rem;
          overflow-y: auto;
        }

        .loading-container,
        .error-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          gap: 1rem;
          color: var(--text-secondary, #64748b);
        }

        .spinner {
          width: 32px;
          height: 32px;
          border: 3px solid var(--border-primary, #e2e8f0);
          border-top: 3px solid var(--accent-primary, #3b82f6);
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        .error-container h2 {
          color: var(--text-primary, #1e293b);
          margin: 0;
        }

        .error-container button {
          padding: 0.5rem 1rem;
          background: var(--accent-primary, #3b82f6);
          color: white;
          border: none;
          border-radius: 0.375rem;
          cursor: pointer;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};

// 笔记详情组件
const NoteDetails: React.FC<{
  nodeId?: string;
  details: any;
  onNavigate: (path: string) => void;
}> = ({ details, onNavigate }) => {
  
  return (
    <div className="note-details">
      <div className="detail-section">
        <h4>反向链接 ({details.backlinks?.length || 0})</h4>
        {details.backlinks?.length > 0 ? (
          <div className="backlinks-list">
            {details.backlinks.map((backlink: any, index: number) => (
              <div 
                key={index} 
                className="backlink-item"
                onClick={() => onNavigate(backlink.source_note_path)}
              >
                <div className="backlink-title">{backlink.source_note_title}</div>
                <div className="backlink-context">{backlink.context}</div>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty-state">暂无反向链接</p>
        )}
      </div>

      <div className="detail-section">
        <h4>相似笔记 ({details.similarNotes?.length || 0})</h4>
        {details.similarNotes?.length > 0 ? (
          <div className="similar-notes-list">
            {details.similarNotes.map((similar: any, index: number) => (
              <div 
                key={index} 
                className="similar-note-item"
                onClick={() => onNavigate(similar.path)}
              >
                <div className="similar-note-title">{similar.title}</div>
                <div className="similar-note-score">
                  相似度: {Math.round(similar.similarity_score * 100)}%
                </div>
                <div className="similar-note-reason">{similar.similarity_reason}</div>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty-state">暂无相似笔记</p>
        )}
      </div>

      <div className="detail-section">
        <h4>正向链接 ({details.outgoingLinks?.length || 0})</h4>
        {details.outgoingLinks?.length > 0 ? (
          <div className="outgoing-links-list">
            {details.outgoingLinks.map((linkId: string, index: number) => (
              <div key={index} className="outgoing-link-item">
                {linkId}
              </div>
            ))}
          </div>
        ) : (
          <p className="empty-state">暂无正向链接</p>
        )}
      </div>

      <style>{`
        .note-details {
          display: flex;
          flex-direction: column;
          gap: 1.5rem;
        }

        .detail-section h4 {
          margin: 0 0 0.75rem 0;
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--text-primary, #1e293b);
        }

        .backlink-item,
        .similar-note-item {
          padding: 0.75rem;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.375rem;
          cursor: pointer;
          transition: all 0.2s;
          margin-bottom: 0.5rem;
        }

        .backlink-item:hover,
        .similar-note-item:hover {
          border-color: var(--accent-primary, #3b82f6);
          background: var(--bg-tertiary, #f1f5f9);
        }

        .backlink-title,
        .similar-note-title {
          font-weight: 500;
          color: var(--text-primary, #1e293b);
          margin-bottom: 0.25rem;
        }

        .backlink-context {
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
          line-height: 1.4;
        }

        .similar-note-score {
          font-size: 0.75rem;
          color: var(--accent-primary, #3b82f6);
          font-weight: 500;
        }

        .similar-note-reason {
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
          margin-top: 0.25rem;
        }

        .outgoing-link-item {
          padding: 0.5rem;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.25rem;
          font-size: 0.875rem;
          margin-bottom: 0.25rem;
        }

        .empty-state {
          color: var(--text-tertiary, #94a3b8);
          font-style: italic;
          text-align: center;
          padding: 1rem;
        }
      `}</style>
    </div>
  );
};

// 标签详情组件
const TagDetails: React.FC<{
  tagName: string;
  nodes: any[];
  edges: any[];
}> = ({ tagName, nodes, edges }) => {
  // 找到使用该标签的所有笔记
  const relatedNotes = nodes.filter(node => 
    node.type === 'note' && 
    edges.some(edge => 
      edge.source === node.id && 
      edge.target === `tag-${tagName}` && 
      edge.type === 'tag'
    )
  );

  return (
    <div className="tag-details">
      <div className="detail-section">
        <h4>标签: #{tagName}</h4>
        <p>包含此标签的笔记: {relatedNotes.length} 篇</p>
      </div>

      <div className="detail-section">
        <h4>相关笔记</h4>
        {relatedNotes.length > 0 ? (
          <div className="related-notes-list">
            {relatedNotes.map(note => (
              <div key={note.id} className="related-note-item">
                <div className="note-title">{note.label}</div>
                <div className="note-meta">
                  字数: {note.metadata.wordCount} | 
                  标签: {note.metadata.tags.length}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty-state">暂无相关笔记</p>
        )}
      </div>

      <style>{`
        .tag-details {
          display: flex;
          flex-direction: column;
          gap: 1.5rem;
        }

        .detail-section h4 {
          margin: 0 0 0.75rem 0;
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--text-primary, #1e293b);
        }

        .related-note-item {
          padding: 0.75rem;
          background: var(--bg-primary, white);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.375rem;
          margin-bottom: 0.5rem;
        }

        .note-title {
          font-weight: 500;
          color: var(--text-primary, #1e293b);
          margin-bottom: 0.25rem;
        }

        .note-meta {
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
        }

        .empty-state {
          color: var(--text-tertiary, #94a3b8);
          font-style: italic;
          text-align: center;
          padding: 1rem;
        }
      `}</style>
    </div>
  );
};

export default GraphPage;