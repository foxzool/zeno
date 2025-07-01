import React, { useState, useEffect, useRef } from 'react';
import { Search as SearchIcon, X, FileText, Calendar } from 'lucide-react';

interface SearchResult {
  id: string;
  title: string;
  path: string;
  content: string;
  score: number;
  lastModified: Date;
}

interface SearchProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (result: SearchResult) => void;
}

// 模拟搜索数据
const mockSearchResults: SearchResult[] = [
  {
    id: '1',
    title: '日常思考.md',
    path: '/notes/daily-thoughts.md',
    content: '今天学习了 React 的新特性...',
    score: 0.95,
    lastModified: new Date('2025-07-01'),
  },
  {
    id: '2',
    title: 'React 最佳实践.md',
    path: '/notes/tech/react-best-practices.md',
    content: 'React 开发中的最佳实践包括...',
    score: 0.87,
    lastModified: new Date('2025-06-30'),
  },
  {
    id: '3',
    title: 'Rust 学习笔记.md',
    path: '/notes/tech/rust-learning.md',
    content: 'Rust 是一门系统编程语言...',
    score: 0.82,
    lastModified: new Date('2025-06-29'),
  },
];

const Search: React.FC<SearchProps> = ({ isOpen, onClose, onSelect }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      return;
    }

    setLoading(true);
    // 模拟搜索延迟
    const timeoutId = setTimeout(() => {
      const filtered = mockSearchResults.filter(result =>
        result.title.toLowerCase().includes(query.toLowerCase()) ||
        result.content.toLowerCase().includes(query.toLowerCase())
      );
      setResults(filtered);
      setSelectedIndex(0);
      setLoading(false);
    }, 200);

    return () => clearTimeout(timeoutId);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'Escape':
        onClose();
        break;
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, results.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (results[selectedIndex]) {
          onSelect(results[selectedIndex]);
          onClose();
        }
        break;
    }
  };

  const formatDate = (date: Date) => {
    const now = new Date();
    const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) return '今天';
    if (diffDays === 1) return '昨天';
    if (diffDays < 7) return `${diffDays} 天前`;
    
    return date.toLocaleDateString('zh-CN');
  };

  if (!isOpen) return null;

  return (
    <div className="search-overlay">
      <div className="search-container" ref={containerRef}>
        <div className="search-header">
          <div className="search-input-container">
            <SearchIcon size={20} className="search-input-icon" />
            <input
              ref={inputRef}
              type="text"
              placeholder="搜索笔记..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              className="search-input"
            />
            {query && (
              <button
                onClick={() => setQuery('')}
                className="clear-button"
              >
                <X size={16} />
              </button>
            )}
          </div>
          <button onClick={onClose} className="close-button">
            <X size={20} />
          </button>
        </div>

        <div className="search-content">
          {loading && (
            <div className="loading-state">
              <div className="loading-spinner"></div>
              <span>搜索中...</span>
            </div>
          )}

          {!loading && query && results.length === 0 && (
            <div className="empty-state">
              <SearchIcon size={48} />
              <h3>未找到相关笔记</h3>
              <p>尝试使用不同的关键词搜索</p>
            </div>
          )}

          {!loading && results.length > 0 && (
            <div className="results-list">
              {results.map((result, index) => (
                <div
                  key={result.id}
                  className={`result-item ${index === selectedIndex ? 'selected' : ''}`}
                  onClick={() => {
                    onSelect(result);
                    onClose();
                  }}
                >
                  <div className="result-icon">
                    <FileText size={16} />
                  </div>
                  <div className="result-content">
                    <div className="result-title">{result.title}</div>
                    <div className="result-path">{result.path}</div>
                    <div className="result-preview">{result.content}</div>
                  </div>
                  <div className="result-meta">
                    <div className="result-date">
                      <Calendar size={12} />
                      <span>{formatDate(result.lastModified)}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {!query && (
            <div className="search-tips">
              <h3>搜索技巧</h3>
              <ul>
                <li>输入关键词搜索笔记标题和内容</li>
                <li>使用 ↑↓ 键选择结果</li>
                <li>按 Enter 键打开选中的笔记</li>
                <li>按 Esc 键关闭搜索</li>
              </ul>
            </div>
          )}
        </div>
      </div>

      <style>{`
        .search-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: flex-start;
          justify-content: center;
          padding-top: 10vh;
          z-index: 1000;
        }

        .search-container {
          width: 90%;
          max-width: 600px;
          background-color: var(--bg-primary, white);
          border-radius: 12px;
          box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
          overflow: hidden;
          max-height: 70vh;
          display: flex;
          flex-direction: column;
        }

        .search-header {
          display: flex;
          align-items: center;
          padding: 16px;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
        }

        .search-input-container {
          flex: 1;
          position: relative;
          display: flex;
          align-items: center;
        }

        .search-input-icon {
          position: absolute;
          left: 12px;
          color: var(--text-tertiary, #94a3b8);
        }

        .search-input {
          width: 100%;
          padding: 12px 16px 12px 44px;
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 8px;
          font-size: 16px;
          outline: none;
          background-color: var(--bg-secondary, #f8fafc);
        }

        .search-input:focus {
          border-color: var(--accent-primary, #3b82f6);
          background-color: var(--bg-primary, white);
        }

        .clear-button {
          position: absolute;
          right: 8px;
          padding: 4px;
          border: none;
          background: transparent;
          color: var(--text-tertiary, #94a3b8);
          border-radius: 4px;
          cursor: pointer;
        }

        .clear-button:hover {
          background-color: var(--bg-tertiary, #f1f5f9);
          color: var(--text-secondary, #64748b);
        }

        .close-button {
          margin-left: 12px;
          padding: 8px;
          border: none;
          background: transparent;
          color: var(--text-tertiary, #94a3b8);
          border-radius: 6px;
          cursor: pointer;
        }

        .close-button:hover {
          background-color: var(--bg-tertiary, #f1f5f9);
          color: var(--text-secondary, #64748b);
        }

        .search-content {
          flex: 1;
          overflow-y: auto;
          min-height: 200px;
        }

        .loading-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 40px 20px;
          color: var(--text-secondary, #64748b);
        }

        .loading-spinner {
          width: 24px;
          height: 24px;
          border: 2px solid var(--bg-tertiary, #f1f5f9);
          border-top: 2px solid var(--accent-primary, #3b82f6);
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 12px;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 40px 20px;
          text-align: center;
          color: var(--text-tertiary, #94a3b8);
        }

        .empty-state h3 {
          margin: 16px 0 8px 0;
          color: var(--text-secondary, #64748b);
        }

        .empty-state p {
          margin: 0;
          font-size: 14px;
        }

        .results-list {
          padding: 8px 0;
        }

        .result-item {
          display: flex;
          align-items: flex-start;
          padding: 12px 16px;
          cursor: pointer;
          transition: background-color 0.1s;
        }

        .result-item:hover,
        .result-item.selected {
          background-color: var(--bg-secondary, #f8fafc);
        }

        .result-icon {
          margin-right: 12px;
          margin-top: 2px;
          color: var(--text-tertiary, #94a3b8);
        }

        .result-content {
          flex: 1;
          min-width: 0;
        }

        .result-title {
          font-weight: 600;
          color: var(--text-primary, #1e293b);
          margin-bottom: 4px;
        }

        .result-path {
          font-size: 12px;
          color: var(--text-tertiary, #94a3b8);
          margin-bottom: 4px;
        }

        .result-preview {
          font-size: 14px;
          color: var(--text-secondary, #64748b);
          line-height: 1.4;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .result-meta {
          margin-left: 12px;
          display: flex;
          flex-direction: column;
          align-items: flex-end;
        }

        .result-date {
          display: flex;
          align-items: center;
          gap: 4px;
          font-size: 12px;
          color: var(--text-tertiary, #94a3b8);
        }

        .search-tips {
          padding: 24px;
          color: var(--text-secondary, #64748b);
        }

        .search-tips h3 {
          margin: 0 0 16px 0;
          color: var(--text-primary, #1e293b);
          font-size: 16px;
        }

        .search-tips ul {
          margin: 0;
          padding-left: 20px;
        }

        .search-tips li {
          margin-bottom: 8px;
          font-size: 14px;
          line-height: 1.5;
        }
      `}</style>
    </div>
  );
};

export default Search;