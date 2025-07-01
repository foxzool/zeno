import React, { useState, useCallback } from 'react';
import { 
  FileText, 
  Folder, 
  FolderOpen, 
  ChevronRight, 
  ChevronDown,
  Search,
  Plus,
  MoreHorizontal
} from 'lucide-react';

export interface FileNode {
  id: string;
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  isExpanded?: boolean;
  level?: number;
}

interface FileTreeProps {
  files: FileNode[];
  selectedFile?: string;
  onFileSelect: (file: FileNode) => void;
  onFileCreate?: (parentPath: string) => void;
  onFolderCreate?: (parentPath: string) => void;
  onFileContextMenu?: (file: FileNode, event: React.MouseEvent) => void;
  className?: string;
}

interface FileItemProps {
  node: FileNode;
  isSelected: boolean;
  onSelect: (node: FileNode) => void;
  onToggle: (nodeId: string) => void;
  onFileContextMenu?: (node: FileNode, e: React.MouseEvent) => void;
}

const FileItem: React.FC<FileItemProps> = ({
  node,
  isSelected,
  onSelect,
  onToggle,
  onFileContextMenu
}) => {
  const handleClick = () => {
    if (node.type === 'directory') {
      onToggle(node.id);
    } else {
      onSelect(node);
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    if (node.type === 'file') {
      onFileContextMenu?.(node, e);
    }
  };

  const paddingLeft = (node.level || 0) * 16 + 8;

  return (
    <div
      className={`file-item ${isSelected ? 'selected' : ''} ${node.type}`}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
      style={{ paddingLeft }}
    >
      <div className="file-item-content">
        {node.type === 'directory' && (
          <span className="expand-icon">
            {node.isExpanded ? (
              <ChevronDown size={14} />
            ) : (
              <ChevronRight size={14} />
            )}
          </span>
        )}
        
        <span className="file-icon">
          {node.type === 'directory' ? (
            node.isExpanded ? (
              <FolderOpen size={16} />
            ) : (
              <Folder size={16} />
            )
          ) : (
            <FileText size={16} />
          )}
        </span>
        
        <span className="file-name">{node.name}</span>
      </div>
      
      <style>{`
        .file-item {
          display: flex;
          align-items: center;
          padding: 4px 8px;
          cursor: pointer;
          user-select: none;
          border-radius: 4px;
          margin: 1px 4px;
          transition: background-color 0.1s;
        }
        
        .file-item:hover {
          background-color: #f1f5f9;
        }
        
        .file-item.selected {
          background-color: #dbeafe;
          color: #1d4ed8;
        }
        
        .file-item-content {
          display: flex;
          align-items: center;
          gap: 4px;
          flex: 1;
          min-width: 0;
        }
        
        .expand-icon {
          display: flex;
          align-items: center;
          width: 14px;
          color: #6b7280;
        }
        
        .file-icon {
          display: flex;
          align-items: center;
          color: #6b7280;
        }
        
        .file-item.directory .file-icon {
          color: #3b82f6;
        }
        
        .file-name {
          font-size: 13px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        
      `}</style>
    </div>
  );
};

export const FileTree: React.FC<FileTreeProps> = ({
  files,
  selectedFile,
  onFileSelect,
  onFileCreate,
  onFolderCreate,
  onFileContextMenu,
  className = ''
}) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');

  const toggleNode = useCallback((nodeId: string) => {
    setExpandedNodes(prev => {
      const newSet = new Set(prev);
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId);
      } else {
        newSet.add(nodeId);
      }
      return newSet;
    });
  }, []);

  const flattenNodes = useCallback((nodes: FileNode[], level = 0): FileNode[] => {
    const result: FileNode[] = [];
    
    for (const node of nodes) {
      const nodeWithLevel = { ...node, level, isExpanded: expandedNodes.has(node.id) };
      
      // Apply search filter
      if (!searchQuery || node.name.toLowerCase().includes(searchQuery.toLowerCase())) {
        result.push(nodeWithLevel);
      }
      
      if (node.children && expandedNodes.has(node.id)) {
        result.push(...flattenNodes(node.children, level + 1));
      }
    }
    
    return result;
  }, [expandedNodes, searchQuery]);

  const flatNodes = flattenNodes(files);

  return (
    <div className={`file-tree ${className}`}>
      <div className="file-tree-header">
        <div className="search-container">
          <Search size={14} className="search-icon" />
          <input
            type="text"
            placeholder="搜索文件..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>
        
        <div className="toolbar">
          <button
            onClick={() => onFileCreate?.('')}
            className="toolbar-button"
            title="新建文件"
          >
            <Plus size={14} />
          </button>
          <button
            onClick={() => onFolderCreate?.('')}
            className="toolbar-button"
            title="新建文件夹"
          >
            <Folder size={14} />
          </button>
          <button className="toolbar-button" title="更多选项">
            <MoreHorizontal size={14} />
          </button>
        </div>
      </div>
      
      <div className="file-tree-content">
        {flatNodes.map((node) => (
          <FileItem
            key={node.id}
            node={node}
            isSelected={selectedFile === node.id}
            onSelect={onFileSelect}
            onToggle={toggleNode}
            onFileContextMenu={onFileContextMenu}
          />
        ))}
        
        {flatNodes.length === 0 && (
          <div className="empty-state">
            {searchQuery ? '未找到匹配的文件' : '暂无文件'}
          </div>
        )}
      </div>
      
      <style>{`
        .file-tree {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: white;
          border-right: 1px solid #e2e8f0;
        }
        
        .file-tree-header {
          padding: 12px;
          border-bottom: 1px solid #e2e8f0;
          background: #f8fafc;
        }
        
        .search-container {
          position: relative;
          margin-bottom: 8px;
        }
        
        .search-icon {
          position: absolute;
          left: 8px;
          top: 50%;
          transform: translateY(-50%);
          color: #6b7280;
        }
        
        .search-input {
          width: 100%;
          padding: 6px 8px 6px 28px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 12px;
          outline: none;
        }
        
        .search-input:focus {
          border-color: #3b82f6;
          box-shadow: 0 0 0 1px #3b82f6;
        }
        
        .toolbar {
          display: flex;
          gap: 4px;
        }
        
        .toolbar-button {
          padding: 4px;
          border: none;
          background: transparent;
          color: #6b7280;
          border-radius: 4px;
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
        }
        
        .toolbar-button:hover {
          background: #e2e8f0;
          color: #374151;
        }
        
        .file-tree-content {
          flex: 1;
          overflow-y: auto;
          padding: 4px 0;
        }
        
        .empty-state {
          padding: 24px;
          text-align: center;
          color: #6b7280;
          font-size: 13px;
        }
        
      `}</style>
    </div>
  );
};

export default FileTree;