import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import Sidebar from './Sidebar';
import FileTree from '../FileTree';
import Search from '../Search';
import CreateNoteDialog from '../CreateNoteDialog';
import ContextMenu, { ContextMenuItem } from '../ContextMenu';
import { FileNode } from '../FileTree';
import { Menu, X, PanelLeft, PanelRight, Search as SearchIcon, Edit3, Trash2, Copy, FolderOpen, Info, FolderPlus, FilePlus, Edit } from 'lucide-react';
import useHotkeys from '../../hooks/useHotkeys';

interface LayoutProps {
  children: React.ReactNode;
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
  const [showCreateFileDialog, setShowCreateFileDialog] = useState(false);
  const [showCreateFolderDialog, setShowCreateFolderDialog] = useState(false);
  const [createParentPath, setCreateParentPath] = useState<string>('');
  const [contextMenu, setContextMenu] = useState<{
    isOpen: boolean;
    position: { x: number; y: number };
    file: FileNode | null;
    type: 'file' | 'folder';
  }>({
    isOpen: false,
    position: { x: 0, y: 0 },
    file: null,
    type: 'file'
  });

  // 将后端返回的文件树转换为前端的 FileNode 结构
  const convertFileTreeNodes = (backendNodes: any[]): FileNode[] => {
    return backendNodes.map((node) => ({
      id: node.id,
      name: node.name,
      path: node.path,
      type: node.type as 'file' | 'directory',
      children: node.children ? convertFileTreeNodes(node.children) : undefined
    }));
  };

  // 加载真实的文件数据
  const loadFileTree = async () => {
    try {
      const config = await invoke<{ workspace_path: string | null }>('get_config');
      
      if (!config.workspace_path) {
        return; // 如果没有设置工作空间，使用默认数据
      }

      const fileTreeNodes = await invoke<any[]>('get_file_tree', { 
        dirPath: config.workspace_path 
      });

      if (fileTreeNodes.length > 0) {
        const treeData = convertFileTreeNodes(fileTreeNodes);
        setFileTreeData(treeData);
      } else {
        setFileTreeData([]); // 如果没有文件，显示空树
      }
    } catch (err) {
      console.error('加载文件树失败:', err);
      // 发生错误时继续使用默认数据
    }
  };

  useEffect(() => {
    loadFileTree();
  }, []);

  // 实时刷新文件树：定时器每30秒刷新一次
  useEffect(() => {
    const interval = setInterval(() => {
      loadFileTree();
    }, 30000); // 30秒

    return () => clearInterval(interval);
  }, []);

  // 实时刷新文件树：页面获得焦点时刷新
  useEffect(() => {
    const handleFocus = () => {
      loadFileTree();
    };

    window.addEventListener('focus', handleFocus);
    document.addEventListener('visibilitychange', () => {
      if (!document.hidden) {
        loadFileTree();
      }
    });

    return () => {
      window.removeEventListener('focus', handleFocus);
      document.removeEventListener('visibilitychange', handleFocus);
    };
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

  const handleFileCreate = (parentPath: string) => {
    setCreateParentPath(parentPath);
    setShowCreateFileDialog(true);
  };

  const handleFolderCreate = (parentPath: string) => {
    setCreateParentPath(parentPath);
    setShowCreateFolderDialog(true);
  };

  const handleCreateConfirm = async (title: string) => {
    try {
      const result = await invoke('create_note', { title, parentPath: createParentPath });
      console.log('文件创建成功:', result);
      
      // 重新加载文件树
      await loadFileTree();
      
      // 导航到新创建的文件
      navigate(`/editor?file=${encodeURIComponent(result as string)}`);
      
      setShowCreateFileDialog(false);
      setCreateParentPath('');
    } catch (err) {
      console.error('创建文件失败:', err);
      alert(`创建文件失败: ${err}`);
    }
  };

  const handleCreateCancel = () => {
    setShowCreateFileDialog(false);
    setCreateParentPath('');
  };

  const handleCreateFolderConfirm = async (name: string) => {
    try {
      await invoke('create_folder', { name, parentPath: createParentPath });
      console.log('文件夹创建成功');
      
      // 重新加载文件树
      await loadFileTree();
      
      setShowCreateFolderDialog(false);
      setCreateParentPath('');
    } catch (err) {
      console.error('创建文件夹失败:', err);
      alert(`创建文件夹失败: ${err}`);
    }
  };

  const handleCreateFolderCancel = () => {
    setShowCreateFolderDialog(false);
    setCreateParentPath('');
  };

  const handleFileContextMenu = (file: FileNode, event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    setContextMenu({
      isOpen: true,
      position: { x: event.clientX, y: event.clientY },
      file,
      type: 'file'
    });
  };

  const handleFolderContextMenu = (folder: FileNode, event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    setContextMenu({
      isOpen: true,
      position: { x: event.clientX, y: event.clientY },
      file: folder,
      type: 'folder'
    });
  };

  const handleDeleteFile = async (file: FileNode) => {
    if (!window.confirm(`确定要删除文件 "${file.name}" 吗？此操作无法撤销。`)) {
      return;
    }

    try {
      await invoke('delete_note', { filePath: file.path });
      
      // 重新加载文件树
      await loadFileTree();
      
      console.log('文件删除成功');
    } catch (err) {
      console.error('删除文件失败:', err);
      alert(`删除文件失败: ${err}`);
    }
  };

  const handleDeleteFolder = async (folder: FileNode) => {
    if (!window.confirm(`确定要删除文件夹 "${folder.name}" 吗？此操作将删除文件夹及其所有内容，无法撤销。`)) {
      return;
    }

    try {
      await invoke('delete_folder', { folderPath: folder.path });
      
      // 重新加载文件树
      await loadFileTree();
      
      console.log('文件夹删除成功');
    } catch (err) {
      console.error('删除文件夹失败:', err);
      alert(`删除文件夹失败: ${err}`);
    }
  };

  const handleRenameFolder = async (folder: FileNode) => {
    const newName = prompt('请输入新的文件夹名称:', folder.name);
    if (!newName || newName === folder.name) {
      return;
    }

    try {
      await invoke('rename_folder', { oldPath: folder.path, newName });
      
      // 重新加载文件树
      await loadFileTree();
      
      console.log('文件夹重命名成功');
    } catch (err) {
      console.error('重命名文件夹失败:', err);
      alert(`重命名文件夹失败: ${err}`);
    }
  };

  const handleCreateFileInFolder = (folder: FileNode) => {
    setCreateParentPath(folder.path);
    setShowCreateFileDialog(true);
  };

  const handleCreateFolderInFolder = (folder: FileNode) => {
    setCreateParentPath(folder.path);
    setShowCreateFolderDialog(true);
  };

  const handleCopyFilePath = (file: FileNode) => {
    navigator.clipboard.writeText(file.path).then(() => {
      console.log('路径已复制到剪贴板');
    }).catch(err => {
      console.error('复制路径失败:', err);
    });
  };

  const handleShowFileInfo = (file: FileNode) => {
    const info = `文件名: ${file.name}\n路径: ${file.path}\n类型: ${file.type === 'file' ? '文件' : '目录'}`;
    alert(info);
  };

  const getFileContextMenuItems = (file: FileNode): ContextMenuItem[] => [
    {
      id: 'open',
      label: '打开编辑',
      icon: <Edit3 size={14} />,
      onClick: () => handleFileSelect(file)
    },
    {
      id: 'separator1',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'showInFolder',
      label: '在文件夹中显示',
      icon: <FolderOpen size={14} />,
      onClick: async () => {
        try {
          await invoke('show_in_folder', { filePath: file.path });
        } catch (err) {
          console.error('打开文件夹失败:', err);
        }
      }
    },
    {
      id: 'copyPath',
      label: '复制路径',
      icon: <Copy size={14} />,
      onClick: () => handleCopyFilePath(file)
    },
    {
      id: 'info',
      label: '属性',
      icon: <Info size={14} />,
      onClick: () => handleShowFileInfo(file)
    },
    {
      id: 'separator2',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'delete',
      label: '删除',
      icon: <Trash2 size={14} />,
      onClick: () => handleDeleteFile(file),
      danger: true
    }
  ];

  const getFolderContextMenuItems = (folder: FileNode): ContextMenuItem[] => [
    {
      id: 'createFile',
      label: '新建文件',
      icon: <FilePlus size={14} />,
      onClick: () => handleCreateFileInFolder(folder)
    },
    {
      id: 'createFolder',
      label: '新建文件夹',
      icon: <FolderPlus size={14} />,
      onClick: () => handleCreateFolderInFolder(folder)
    },
    {
      id: 'separator1',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'rename',
      label: '重命名',
      icon: <Edit size={14} />,
      onClick: () => handleRenameFolder(folder)
    },
    {
      id: 'separator2',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'showInFolder',
      label: '在文件夹中显示',
      icon: <FolderOpen size={14} />,
      onClick: async () => {
        try {
          await invoke('show_in_folder', { filePath: folder.path });
        } catch (err) {
          console.error('打开文件夹失败:', err);
        }
      }
    },
    {
      id: 'copyPath',
      label: '复制路径',
      icon: <Copy size={14} />,
      onClick: () => handleCopyFilePath(folder)
    },
    {
      id: 'info',
      label: '属性',
      icon: <Info size={14} />,
      onClick: () => handleShowFileInfo(folder)
    },
    {
      id: 'separator3',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'delete',
      label: '删除文件夹',
      icon: <Trash2 size={14} />,
      onClick: () => handleDeleteFolder(folder),
      danger: true
    }
  ];

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
                onFileCreate={handleFileCreate}
                onFolderCreate={handleFolderCreate}
                onFileContextMenu={handleFileContextMenu}
                onFolderContextMenu={handleFolderContextMenu}
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

      {/* 创建文件对话框 */}
      <CreateNoteDialog
        isOpen={showCreateFileDialog}
        onClose={handleCreateCancel}
        onConfirm={handleCreateConfirm}
      />

      {/* 创建文件夹对话框 */}
      <CreateNoteDialog
        isOpen={showCreateFolderDialog}
        onClose={handleCreateFolderCancel}
        onConfirm={handleCreateFolderConfirm}
        title="创建新文件夹"
        placeholder="请输入文件夹名称"
        confirmText="创建"
      />

      {/* 右键菜单 */}
      <ContextMenu
        isOpen={contextMenu.isOpen}
        position={contextMenu.position}
        items={contextMenu.file ? 
          (contextMenu.type === 'folder' ? 
            getFolderContextMenuItems(contextMenu.file) : 
            getFileContextMenuItems(contextMenu.file)
          ) : []
        }
        onClose={() => setContextMenu(prev => ({ ...prev, isOpen: false }))}
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