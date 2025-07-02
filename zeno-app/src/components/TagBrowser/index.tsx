import React, { useState, useCallback, useMemo } from 'react';
import { Search, ChevronRight, ChevronDown, Tag as TagIcon, Hash, TrendingUp, Layers } from 'lucide-react';
import { useTagHierarchy, HierarchicalTag } from '../../hooks/useTagHierarchy';

interface TagBrowserProps {
  onTagSelect?: (tag: HierarchicalTag) => void;
  selectedTags?: string[];
  showStatistics?: boolean;
  showSearch?: boolean;
  className?: string;
}

interface TagTreeNodeProps {
  tag: HierarchicalTag;
  isSelected: boolean;
  onSelect: (tag: HierarchicalTag) => void;
  allTags: HierarchicalTag[];
  expandedNodes: Set<string>;
  onToggleExpand: (tagName: string) => void;
  level: number;
}

const TagTreeNode: React.FC<TagTreeNodeProps> = ({
  tag,
  isSelected,
  onSelect,
  allTags,
  expandedNodes,
  onToggleExpand,
  level,
}) => {
  const hasChildren = tag.children.length > 0;
  const isExpanded = expandedNodes.has(tag.name);
  const children = allTags.filter(t => tag.children.includes(t.name));

  const handleToggleExpand = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    onToggleExpand(tag.name);
  }, [tag.name, onToggleExpand]);

  const handleSelect = useCallback(() => {
    onSelect(tag);
  }, [tag, onSelect]);

  return (
    <div className="tag-tree-node">
      <div 
        className={`tag-item ${isSelected ? 'selected' : ''}`}
        style={{ paddingLeft: `${level * 16 + 8}px` }}
        onClick={handleSelect}
      >
        <div className="tag-content">
          {hasChildren && (
            <button 
              className="expand-button" 
              onClick={handleToggleExpand}
            >
              {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            </button>
          )}
          {!hasChildren && <div className="expand-spacer" />}
          
          <TagIcon size={14} className="tag-icon" />
          
          <span className="tag-name">
            {tag.name.split('/').pop()} {/* 只显示最后一部分 */}
          </span>
          
          {tag.note_count > 0 && (
            <span className="tag-count">{tag.note_count}</span>
          )}
        </div>
      </div>

      {hasChildren && isExpanded && (
        <div className="tag-children">
          {children.map(child => (
            <TagTreeNode
              key={child.name}
              tag={child}
              isSelected={isSelected}
              onSelect={onSelect}
              allTags={allTags}
              expandedNodes={expandedNodes}
              onToggleExpand={onToggleExpand}
              level={level + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export const TagBrowser: React.FC<TagBrowserProps> = ({
  onTagSelect,
  selectedTags = [],
  showStatistics = true,
  showSearch = true,
  className = '',
}) => {
  const {
    allTags,
    rootTags,
    popularTags,
    statistics,
    loading,
    error,
    searchTags,
  } = useTagHierarchy();

  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<HierarchicalTag[]>([]);
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [activeTab, setActiveTab] = useState<'hierarchy' | 'popular' | 'search'>('hierarchy');

  // 搜索标签
  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query);
    if (query.trim()) {
      const results = await searchTags(query);
      setSearchResults(results);
      setActiveTab('search');
    } else {
      setSearchResults([]);
      setActiveTab('hierarchy');
    }
  }, [searchTags]);

  // 切换节点展开状态
  const handleToggleExpand = useCallback((tagName: string) => {
    setExpandedNodes(prev => {
      const next = new Set(prev);
      if (next.has(tagName)) {
        next.delete(tagName);
      } else {
        next.add(tagName);
      }
      return next;
    });
  }, []);

  // 选择标签
  const handleTagSelect = useCallback((tag: HierarchicalTag) => {
    onTagSelect?.(tag);
  }, [onTagSelect]);

  // 展开到根节点
  const expandToRoot = useCallback(() => {
    setExpandedNodes(new Set(rootTags.map(tag => tag.name)));
  }, [rootTags]);

  // 折叠所有节点
  const collapseAll = useCallback(() => {
    setExpandedNodes(new Set());
  }, []);

  const displayTags = useMemo(() => {
    if (activeTab === 'search') {
      return searchResults;
    } else if (activeTab === 'popular') {
      return popularTags;
    } else {
      return rootTags;
    }
  }, [activeTab, searchResults, popularTags, rootTags]);

  if (loading) {
    return (
      <div className={`tag-browser loading ${className}`}>
        <div className="loading-spinner">
          <div className="spinner" />
          <span>加载标签中...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`tag-browser error ${className}`}>
        <div className="error-message">
          <p>加载标签失败: {error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`tag-browser ${className}`}>
      {/* 搜索栏 */}
      {showSearch && (
        <div className="search-section">
          <div className="search-input">
            <Search size={16} />
            <input
              type="text"
              placeholder="搜索标签..."
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
            />
          </div>
        </div>
      )}

      {/* 统计信息 */}
      {showStatistics && statistics && (
        <div className="statistics-section">
          <div className="stat-item">
            <Hash size={14} />
            <span>总标签: {statistics.total_tags}</span>
          </div>
          <div className="stat-item">
            <Layers size={14} />
            <span>层深: {statistics.max_depth}</span>
          </div>
          {statistics.most_used_tag && (
            <div className="stat-item">
              <TrendingUp size={14} />
              <span>热门: {statistics.most_used_tag.split('/').pop()}</span>
            </div>
          )}
        </div>
      )}

      {/* 标签页导航 */}
      <div className="tab-navigation">
        <button
          className={`tab-button ${activeTab === 'hierarchy' ? 'active' : ''}`}
          onClick={() => setActiveTab('hierarchy')}
        >
          <Layers size={14} />
          层次结构
        </button>
        <button
          className={`tab-button ${activeTab === 'popular' ? 'active' : ''}`}
          onClick={() => setActiveTab('popular')}
        >
          <TrendingUp size={14} />
          热门标签
        </button>
      </div>

      {/* 操作按钮 */}
      {activeTab === 'hierarchy' && (
        <div className="action-buttons">
          <button onClick={expandToRoot} className="action-button">
            展开所有
          </button>
          <button onClick={collapseAll} className="action-button">
            折叠所有
          </button>
        </div>
      )}

      {/* 标签列表 */}
      <div className="tag-list">
        {displayTags.length === 0 ? (
          <div className="empty-state">
            {activeTab === 'search' ? '没有找到匹配的标签' : '暂无标签'}
          </div>
        ) : (
          <div className="tag-tree">
            {activeTab === 'hierarchy' ? (
              displayTags.map(tag => (
                <TagTreeNode
                  key={tag.name}
                  tag={tag}
                  isSelected={selectedTags.includes(tag.name)}
                  onSelect={handleTagSelect}
                  allTags={allTags}
                  expandedNodes={expandedNodes}
                  onToggleExpand={handleToggleExpand}
                  level={0}
                />
              ))
            ) : (
              displayTags.map(tag => (
                <div
                  key={tag.name}
                  className={`tag-item flat ${selectedTags.includes(tag.name) ? 'selected' : ''}`}
                  onClick={() => handleTagSelect(tag)}
                >
                  <TagIcon size={14} className="tag-icon" />
                  <span className="tag-name">{tag.name}</span>
                  <span className="tag-count">{tag.note_count}</span>
                </div>
              ))
            )}
          </div>
        )}
      </div>

      <style>{`
        .tag-browser {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: var(--bg-primary, #ffffff);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.5rem;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .search-section {
          padding: 1rem;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .search-input {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem;
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.375rem;
        }

        .search-input input {
          border: none;
          outline: none;
          flex: 1;
          background: transparent;
          font-size: 0.875rem;
        }

        .statistics-section {
          display: flex;
          gap: 1rem;
          padding: 0.75rem 1rem;
          background: var(--bg-secondary, #f8fafc);
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
        }

        .stat-item {
          display: flex;
          align-items: center;
          gap: 0.25rem;
        }

        .tab-navigation {
          display: flex;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .tab-button {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          background: transparent;
          border: none;
          cursor: pointer;
          font-size: 0.875rem;
          color: var(--text-secondary, #64748b);
          transition: all 0.2s;
          flex: 1;
          justify-content: center;
        }

        .tab-button:hover {
          background: var(--bg-tertiary, #f1f5f9);
        }

        .tab-button.active {
          background: var(--bg-primary, #ffffff);
          color: var(--text-primary, #1e293b);
          border-bottom: 2px solid var(--accent-primary, #3b82f6);
        }

        .action-buttons {
          display: flex;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .action-button {
          padding: 0.25rem 0.5rem;
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.25rem;
          cursor: pointer;
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
          transition: all 0.2s;
        }

        .action-button:hover {
          background: var(--bg-tertiary, #f1f5f9);
          color: var(--text-primary, #1e293b);
        }

        .tag-list {
          flex: 1;
          overflow-y: auto;
          padding: 0.5rem 0;
        }

        .tag-tree {
          display: flex;
          flex-direction: column;
        }

        .tag-item {
          display: flex;
          align-items: center;
          padding: 0.5rem 1rem;
          cursor: pointer;
          transition: background 0.2s;
          font-size: 0.875rem;
        }

        .tag-item:hover {
          background: var(--bg-tertiary, #f1f5f9);
        }

        .tag-item.selected {
          background: var(--accent-secondary, #eff6ff);
          color: var(--accent-primary, #3b82f6);
        }

        .tag-item.flat {
          padding-left: 1rem;
        }

        .tag-content {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          flex: 1;
        }

        .expand-button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 16px;
          height: 16px;
          background: none;
          border: none;
          cursor: pointer;
          color: var(--text-secondary, #64748b);
          transition: color 0.2s;
        }

        .expand-button:hover {
          color: var(--text-primary, #1e293b);
        }

        .expand-spacer {
          width: 16px;
          height: 16px;
        }

        .tag-icon {
          color: var(--text-secondary, #64748b);
        }

        .tag-name {
          flex: 1;
          color: var(--text-primary, #1e293b);
          font-weight: 500;
        }

        .tag-count {
          padding: 0.125rem 0.375rem;
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.75rem;
          font-size: 0.75rem;
          color: var(--text-secondary, #64748b);
          font-weight: 500;
        }

        .empty-state {
          padding: 2rem 1rem;
          text-align: center;
          color: var(--text-tertiary, #94a3b8);
          font-style: italic;
        }

        .loading-spinner,
        .error-message {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
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
      `}</style>
    </div>
  );
};

export default TagBrowser;