import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import Sidebar from './Sidebar';
import FileTree from '../FileTree';
import Search from '../Search';
import { FileNode } from '../FileTree';
import { Menu, X, PanelLeft, PanelRight, Search as SearchIcon } from 'lucide-react';
import useHotkeys from '../../hooks/useHotkeys';

interface LayoutProps {
  children: React.ReactNode;
}

interface NoteFile {
  path: string;
  name: string;
  size: number;
  modified: string | null;
}

// 模拟文件数据
const mockFiles: FileNode[] = [
  {
    id: '1',
    name: '笔记',
    path: '/notes',
    type: 'directory',
    children: [
      {
        id: '2',
        name: '日常思考.md',
        path: '/notes/daily-thoughts.md',
        type: 'file'
      },
      {
        id: '3',
        name: '技术学习',
        path: '/notes/tech',
        type: 'directory',
        children: [
          {
            id: '4',
            name: 'React 最佳实践.md',
            path: '/notes/tech/react-best-practices.md',
            type: 'file'
          },
          {
            id: '5',
            name: 'Rust 学习笔记.md',
            path: '/notes/tech/rust-learning.md',
            type: 'file'
          }
        ]
      }
    ]
  },
  {
    id: '6',
    name: '项目',
    path: '/projects',
    type: 'directory',
    children: [
      {
        id: '7',
        name: 'Zeno 开发日志.md',
        path: '/projects/zeno-dev-log.md',
        type: 'file'
      }
    ]
  }
];

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const navigate = useNavigate();
  const [leftPanelVisible, setLeftPanelVisible] = useState(true);
  const [rightPanelVisible, setRightPanelVisible] = useState(true);
  const [leftPanelWidth] = useState(280);
  const [rightPanelWidth] = useState(320);
  const [selectedFile, setSelectedFile] = useState<string>();
  const [searchOpen, setSearchOpen] = useState(false);
  const [fileTreeData, setFileTreeData] = useState<FileNode[]>(mockFiles);

  // 将笔记列表转换为文件树结构
  const buildFileTree = (notes: NoteFile[]): FileNode[] => {
    // 简化版本：只创建一个平面列表，让用户可以直接点击文件
    return notes.map((note) => ({
      id: note.path,
      name: note.name,
      path: note.path,
      type: 'file' as const
    }));
  };

  // 加载真实的文件数据
  useEffect(() => {
    const loadFileTree = async () => {
      try {
        const config = await invoke<{ workspace_path: string | null }>('get_config');
        
        if (!config.workspace_path) {
          return; // 如果没有设置工作空间，使用默认数据
        }

        const notes = await invoke<NoteFile[]>('list_notes', { 
          dirPath: config.workspace_path 
        });

        if (notes.length > 0) {
          const treeData = buildFileTree(notes);
          setFileTreeData(treeData);
        }
      } catch (err) {
        console.error('加载文件树失败:', err);
        // 发生错误时继续使用默认数据
      }
    };

    loadFileTree();
  }, []);

  // 响应式设计：小屏幕时隐藏侧边栏
  useEffect(() => {
    const handleResize = () => {
      const isMobile = window.innerWidth < 768;
      if (isMobile) {
        setLeftPanelVisible(false);
        setRightPanelVisible(false);
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const handleFileSelect = (file: FileNode) => {
    setSelectedFile(file.id);
    // 只有当文件是 Markdown 文件时才导航到编辑器
    if (file.type === 'file' && (file.path.endsWith('.md') || file.path.endsWith('.markdown'))) {
      navigate(`/editor?file=${encodeURIComponent(file.path)}`);
    }
    console.log('Selected file:', file);
  };

  const handleSearchSelect = (result: any) => {
    console.log('Selected search result:', result);
    // TODO: 打开搜索结果文件
  };

  // 设置快捷键
  useHotkeys([
    {
      key: 'k',
      ctrl: true,
      callback: () => setSearchOpen(true),
    },
    {
      key: 'b',
      ctrl: true,
      callback: () => setLeftPanelVisible(!leftPanelVisible),
    },
    {
      key: 'Escape',
      callback: () => setSearchOpen(false),
    },
  ]);

  return (
    <div className="layout">
      {/* 顶部工具栏 */}
      <div className="toolbar">
        <div className="toolbar-left">
          <button
            onClick={() => setLeftPanelVisible(!leftPanelVisible)}
            className="toolbar-button"
            title="切换文件树"
          >
            <PanelLeft size={16} />
          </button>
          <span className="app-title">Zeno</span>
        </div>
        
        <div className="toolbar-right">
          <button
            onClick={() => setSearchOpen(true)}
            className="toolbar-button"
            title="搜索 (Ctrl+K)"
          >
            <SearchIcon size={16} />
          </button>
          <button
            onClick={() => setRightPanelVisible(!rightPanelVisible)}
            className="toolbar-button"
            title="切换右侧面板"
          >
            <PanelRight size={16} />
          </button>
          <button className="toolbar-button" title="菜单">
            <Menu size={16} />
          </button>
        </div>
      </div>

      {/* 主内容区 */}
      <div className="main-container">
        {/* 左侧面板 - 导航和文件树 */}
        {leftPanelVisible && (
          <div 
            className="left-panel"
            style={{ width: leftPanelWidth }}
          >
            <div className="sidebar-section">
              <Sidebar />
            </div>
            <div className="filetree-section">
              <FileTree
                files={fileTreeData}
                selectedFile={selectedFile}
                onFileSelect={handleFileSelect}
              />
            </div>
          </div>
        )}

        {/* 中间面板 - 主内容 */}
        <div className="center-panel">
          <main className="main-content">
            {children}
          </main>
        </div>

        {/* 右侧面板 - 预览/大纲 */}
        {rightPanelVisible && (
          <div 
            className="right-panel"
            style={{ width: rightPanelWidth }}
          >
            <div className="panel-header">
              <h3>大纲</h3>
              <button 
                onClick={() => setRightPanelVisible(false)}
                className="close-button"
              >
                <X size={14} />
              </button>
            </div>
            <div className="panel-content">
              <p className="placeholder-text">文档大纲将在这里显示</p>
            </div>
          </div>
        )}
      </div>

      {/* 搜索组件 */}
      <Search
        isOpen={searchOpen}
        onClose={() => setSearchOpen(false)}
        onSelect={handleSearchSelect}
      />
      
      <style>{`
        .layout {
          display: flex;
          flex-direction: column;
          height: 100vh;
          background-color: var(--bg-secondary, #f8fafc);
        }
        
        .toolbar {
          display: flex;
          justify-content: space-between;
          align-items: center;
          height: 48px;
          padding: 0 16px;
          background-color: var(--bg-primary, white);
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          position: relative;
          z-index: 10;
        }
        
        .toolbar-left,
        .toolbar-right {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        
        .app-title {
          font-weight: 600;
          font-size: 16px;
          color: var(--text-primary, #1e293b);
        }
        
        .toolbar-button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 32px;
          height: 32px;
          border: none;
          background: transparent;
          color: var(--text-secondary, #64748b);
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }
        
        .toolbar-button:hover {
          background-color: var(--bg-tertiary, #f1f5f9);
          color: var(--text-primary, #1e293b);
        }
        
        .main-container {
          display: flex;
          flex: 1;
          overflow: hidden;
        }
        
        .left-panel {
          min-width: 200px;
          max-width: 400px;
          background-color: var(--bg-primary, white);
          border-right: 1px solid var(--border-primary, #e2e8f0);
          display: flex;
          flex-direction: column;
        }
        
        .sidebar-section {
          flex-shrink: 0;
        }
        
        .filetree-section {
          flex: 1;
          overflow-y: auto;
        }
        
        .center-panel {
          flex: 1;
          display: flex;
          flex-direction: column;
          min-width: 400px;
          background-color: var(--bg-primary, white);
        }
        
        .main-content {
          flex: 1;
          padding: 24px;
          overflow-y: auto;
          background-color: var(--bg-primary, white);
        }
        
        .right-panel {
          min-width: 250px;
          max-width: 500px;
          background-color: var(--bg-primary, white);
          border-left: 1px solid var(--border-primary, #e2e8f0);
          display: flex;
          flex-direction: column;
        }
        
        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          background-color: var(--bg-secondary, #f8fafc);
        }
        
        .panel-header h3 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
          color: var(--text-primary, #1e293b);
        }
        
        .close-button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 24px;
          height: 24px;
          border: none;
          background: transparent;
          color: var(--text-secondary, #64748b);
          border-radius: 4px;
          cursor: pointer;
        }
        
        .close-button:hover {
          background-color: var(--bg-tertiary, #f1f5f9);
          color: var(--text-primary, #1e293b);
        }
        
        .panel-content {
          flex: 1;
          padding: 16px;
          overflow-y: auto;
        }
        
        .placeholder-text {
          color: var(--text-tertiary, #94a3b8);
          font-size: 13px;
          text-align: center;
          margin-top: 40px;
        }
        
        @media (max-width: 768px) {
          .toolbar {
            padding: 0 12px;
          }
          
          .main-content {
            padding: 16px;
          }
          
          .left-panel,
          .right-panel {
            position: absolute;
            top: 48px;
            bottom: 0;
            z-index: 20;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
          }
          
          .left-panel {
            left: 0;
          }
          
          .right-panel {
            right: 0;
          }
        }
      `}</style>
    </div>
  );
};

export default Layout;