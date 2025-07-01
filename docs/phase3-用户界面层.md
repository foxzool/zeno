# Phase 3: 用户界面层开发计划

## 阶段概述

在稳定的数据管理层基础上，开发用户友好的桌面应用界面。重点实现流畅的编辑体验、直观的文件管理和个性化的用户设置。

**预计时间**: 4-5周  
**优先级**: 高 (用户体验核心)  
**前置条件**: Phase 2数据管理层完成

## 目标与交付物

### 主要目标
- 实现专业级的Markdown编辑器
- 建立直观的文件管理界面
- 提供完整的用户个性化设置
- 确保流畅的用户交互体验

### 交付物
- 功能完整的Markdown编辑器
- 三栏式主界面布局
- 文件树和管理组件
- 设置和配置系统
- 主题和个性化系统

## 详细任务清单

### 3.1 核心编辑器实现

**任务描述**: 实现专业级的Markdown编辑和预览功能

**编辑器选型考虑**:
```typescript
// 技术选型对比
const EDITOR_OPTIONS = {
  monaco: {
    pros: ['VS Code同款', '强大的语言支持', '丰富的API'],
    cons: ['体积较大', 'Tauri集成复杂度'],
    适用场景: '代码重度用户'
  },
  codemirror6: {
    pros: ['轻量级', '高度可定制', '性能优秀'],
    cons: ['学习曲线', '生态相对小'],
    适用场景: '注重性能和定制'
  },
  tiptap: {
    pros: ['所见即所得', '用户友好', 'Vue/React生态'],
    cons: ['Markdown原生支持有限'],
    适用场景: '普通用户为主'
  }
};

// 推荐选择: CodeMirror 6 + 自定义扩展
```

**编辑器组件结构**:
```typescript
// components/Editor/index.tsx
import { EditorView } from '@codemirror/view';
import { EditorState } from '@codemirror/state';

export interface EditorProps {
  note: Note | null;
  onSave: (content: string) => void;
  onAutoSave: (content: string) => void;
  readOnly?: boolean;
  theme?: 'light' | 'dark';
}

export const Editor: React.FC<EditorProps> = ({ note, onSave, onAutoSave }) => {
  const [editorView, setEditorView] = useState<EditorView | null>(null);
  const [isDirty, setIsDirty] = useState(false);
  
  // 编辑器初始化
  useEffect(() => {
    const state = EditorState.create({
      doc: note?.content || '',
      extensions: [
        basicSetup,
        markdown(),
        oneDark,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            setIsDirty(true);
            debouncedAutoSave(update.state.doc.toString());
          }
        }),
        // 自定义扩展
        wikiLinkExtension(),
        frontmatterExtension(),
        mathExtension(),
      ],
    });
    
    const view = new EditorView({
      state,
      parent: editorRef.current!,
    });
    
    setEditorView(view);
    return () => view.destroy();
  }, [note?.id]);
  
  return (
    <div className="editor-container">
      <EditorToolbar 
        editor={editorView}
        isDirty={isDirty}
        onSave={handleSave}
      />
      <div ref={editorRef} className="editor-content" />
      <EditorStatusBar 
        note={note}
        wordCount={wordCount}
        selection={selection}
      />
    </div>
  );
};
```

**编辑器特性**:
- 语法高亮和智能补全
- 实时预览 (分栏或同屏)
- 自动保存 (防丢失)
- 快捷键支持
- 查找替换功能
- 代码折叠
- 行号显示
- Vim模式 (可选)

**自定义扩展**:
```typescript
// extensions/wikiLink.ts
export const wikiLinkExtension = () => {
  return ViewPlugin.fromClass(class {
    decorations: DecorationSet;
    
    constructor(view: EditorView) {
      this.decorations = this.buildDecorations(view);
    }
    
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = this.buildDecorations(update.view);
      }
    }
    
    buildDecorations(view: EditorView) {
      const builder = new RangeSetBuilder<Decoration>();
      
      for (let { from, to } of view.visibleRanges) {
        syntaxTree(view.state).iterate({
          from, to,
          enter: (node) => {
            if (node.name === "WikiLink") {
              builder.add(
                node.from, 
                node.to,
                Decoration.mark({ class: "wiki-link" })
              );
            }
          }
        });
      }
      
      return builder.finish();
    }
  }, {
    decorations: v => v.decorations
  });
};
```

**验收标准**:
- [ ] 编辑器启动时间 < 500ms
- [ ] 大文件 (1MB+) 编辑流畅
- [ ] 自动保存功能正常
- [ ] 快捷键响应及时
- [ ] 语法高亮正确
- [ ] 查找替换功能完整

### 3.2 主界面布局

**任务描述**: 实现三栏式主界面和响应式布局

**布局结构**:
```typescript
// components/Layout/MainLayout.tsx
export const MainLayout: React.FC = () => {
  const [leftPanelWidth, setLeftPanelWidth] = useState(280);
  const [rightPanelWidth, setRightPanelWidth] = useState(320);
  const [leftPanelVisible, setLeftPanelVisible] = useState(true);
  const [rightPanelVisible, setRightPanelVisible] = useState(true);
  
  return (
    <div className="main-layout">
      {/* 标题栏 */}
      <TitleBar />
      
      {/* 工具栏 */}
      <Toolbar />
      
      {/* 主内容区 */}
      <div className="main-content">
        {/* 左侧面板 - 文件树 */}
        {leftPanelVisible && (
          <ResizablePanel
            width={leftPanelWidth}
            onResize={setLeftPanelWidth}
            minWidth={200}
            maxWidth={500}
          >
            <FileExplorer />
          </ResizablePanel>
        )}
        
        {/* 中间面板 - 编辑器 */}
        <div className="editor-panel">
          <Editor />
        </div>
        
        {/* 右侧面板 - 预览/大纲 */}
        {rightPanelVisible && (
          <ResizablePanel
            width={rightPanelWidth}
            onResize={setRightPanelWidth}
            minWidth={250}
            maxWidth={600}
          >
            <RightPanel />
          </ResizablePanel>
        )}
      </div>
      
      {/* 状态栏 */}
      <StatusBar />
    </div>
  );
};
```

**响应式设计**:
```typescript
// hooks/useResponsiveLayout.ts
export const useResponsiveLayout = () => {
  const [breakpoint, setBreakpoint] = useState<'sm' | 'md' | 'lg' | 'xl'>('lg');
  
  useEffect(() => {
    const updateBreakpoint = () => {
      const width = window.innerWidth;
      if (width < 768) setBreakpoint('sm');
      else if (width < 1024) setBreakpoint('md');
      else if (width < 1440) setBreakpoint('lg');
      else setBreakpoint('xl');
    };
    
    window.addEventListener('resize', updateBreakpoint);
    updateBreakpoint();
    
    return () => window.removeEventListener('resize', updateBreakpoint);
  }, []);
  
  return {
    breakpoint,
    isMobile: breakpoint === 'sm',
    isTablet: breakpoint === 'md',
    isDesktop: breakpoint === 'lg' || breakpoint === 'xl',
  };
};
```

**主题系统**:
```typescript
// context/ThemeContext.tsx
export type Theme = 'light' | 'dark' | 'auto';

export interface ThemeContextType {
  theme: Theme;
  actualTheme: 'light' | 'dark';
  setTheme: (theme: Theme) => void;
  colors: ThemeColors;
}

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme, setTheme] = useState<Theme>('auto');
  const [actualTheme, setActualTheme] = useState<'light' | 'dark'>('light');
  
  useEffect(() => {
    if (theme === 'auto') {
      // 监听系统主题变化
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      setActualTheme(mediaQuery.matches ? 'dark' : 'light');
      
      const handleChange = (e: MediaQueryListEvent) => {
        setActualTheme(e.matches ? 'dark' : 'light');
      };
      
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    } else {
      setActualTheme(theme);
    }
  }, [theme]);
  
  return (
    <ThemeContext.Provider value={{ theme, actualTheme, setTheme, colors }}>
      <div data-theme={actualTheme}>
        {children}
      </div>
    </ThemeContext.Provider>
  );
};
```

**验收标准**:
- [ ] 布局在不同屏幕尺寸下正常显示
- [ ] 面板拖拽调整大小流畅
- [ ] 主题切换无闪烁
- [ ] 键盘导航完整支持
- [ ] 高对比度和无障碍支持

### 3.3 文件管理界面

**任务描述**: 实现直观的文件树和管理功能

**文件树组件**:
```typescript
// components/FileExplorer/index.tsx
export interface FileTreeNode {
  id: string;
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileTreeNode[];
  isExpanded?: boolean;
  isSelected?: boolean;
}

export const FileExplorer: React.FC = () => {
  const [fileTree, setFileTree] = useState<FileTreeNode[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [draggedItem, setDraggedItem] = useState<FileTreeNode | null>(null);
  
  return (
    <div className="file-explorer">
      {/* 搜索栏 */}
      <div className="search-bar">
        <SearchInput
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="搜索文件..."
        />
      </div>
      
      {/* 工具栏 */}
      <div className="toolbar">
        <IconButton icon="plus" onClick={handleCreateFile} />
        <IconButton icon="folder-plus" onClick={handleCreateFolder} />
        <IconButton icon="refresh" onClick={handleRefresh} />
      </div>
      
      {/* 文件树 */}
      <div className="file-tree">
        <VirtualizedTree
          nodes={filteredFileTree}
          selectedId={selectedFile}
          onSelect={setSelectedFile}
          onExpand={handleExpand}
          onRename={handleRename}
          onDelete={handleDelete}
          onDragStart={handleDragStart}
          onDragOver={handleDragOver}
          onDrop={handleDrop}
        />
      </div>
      
      {/* 上下文菜单 */}
      <ContextMenu
        items={contextMenuItems}
        onSelect={handleContextMenuSelect}
      />
    </div>
  );
};
```

**虚拟化树组件**:
```typescript
// components/VirtualizedTree/index.tsx
export const VirtualizedTree: React.FC<VirtualizedTreeProps> = ({
  nodes,
  selectedId,
  onSelect,
  onExpand,
}) => {
  const [visibleNodes, setVisibleNodes] = useState<FileTreeNode[]>([]);
  const virtualizerRef = useRef<FixedSizeList>(null);
  
  // 计算可见节点 (展开状态下的扁平化列表)
  useEffect(() => {
    const flattenNodes = (nodes: FileTreeNode[], level = 0): FileTreeNode[] => {
      const result: FileTreeNode[] = [];
      
      for (const node of nodes) {
        result.push({ ...node, level });
        
        if (node.isExpanded && node.children) {
          result.push(...flattenNodes(node.children, level + 1));
        }
      }
      
      return result;
    };
    
    setVisibleNodes(flattenNodes(nodes));
  }, [nodes]);
  
  const Row = ({ index, style }: { index: number; style: CSSProperties }) => {
    const node = visibleNodes[index];
    
    return (
      <div style={style}>
        <FileTreeItem
          node={node}
          isSelected={node.id === selectedId}
          onSelect={() => onSelect(node.id)}
          onExpand={() => onExpand(node.id)}
        />
      </div>
    );
  };
  
  return (
    <FixedSizeList
      ref={virtualizerRef}
      height={400}
      itemCount={visibleNodes.length}
      itemSize={28}
      overscanCount={10}
    >
      {Row}
    </FixedSizeList>
  );
};
```

**文件操作功能**:
- 新建文件/文件夹
- 重命名
- 删除 (回收站支持)
- 复制/粘贴
- 拖拽移动
- 批量操作
- 收藏夹/书签

**搜索和过滤**:
```typescript
// hooks/useFileSearch.ts
export const useFileSearch = (files: FileTreeNode[], query: string) => {
  return useMemo(() => {
    if (!query.trim()) return files;
    
    const searchTerms = query.toLowerCase().split(' ');
    
    const matchesSearch = (node: FileTreeNode): boolean => {
      const searchText = `${node.name} ${node.path}`.toLowerCase();
      return searchTerms.every(term => searchText.includes(term));
    };
    
    const filterTree = (nodes: FileTreeNode[]): FileTreeNode[] => {
      return nodes.reduce<FileTreeNode[]>((acc, node) => {
        if (matchesSearch(node)) {
          acc.push(node);
        } else if (node.children) {
          const filteredChildren = filterTree(node.children);
          if (filteredChildren.length > 0) {
            acc.push({
              ...node,
              children: filteredChildren,
              isExpanded: true, // 搜索时展开匹配的目录
            });
          }
        }
        return acc;
      }, []);
    };
    
    return filterTree(files);
  }, [files, query]);
};
```

**验收标准**:
- [ ] 大量文件 (1k+) 的渲染性能流畅
- [ ] 搜索响应时间 < 100ms
- [ ] 拖拽操作体验良好
- [ ] 键盘导航完整
- [ ] 文件操作撤销/重做支持

### 3.4 设置和配置界面

**任务描述**: 建立完整的用户设置和配置系统

**设置界面结构**:
```typescript
// components/Settings/index.tsx
export const SettingsDialog: React.FC<SettingsDialogProps> = ({ isOpen, onClose }) => {
  const [activeTab, setActiveTab] = useState('general');
  
  const tabs = [
    { id: 'general', label: '通用', icon: 'settings' },
    { id: 'editor', label: '编辑器', icon: 'edit' },
    { id: 'appearance', label: '外观', icon: 'palette' },
    { id: 'shortcuts', label: '快捷键', icon: 'keyboard' },
    { id: 'plugins', label: '插件', icon: 'plugin' },
    { id: 'sync', label: '同步', icon: 'sync' },
  ];
  
  return (
    <Dialog isOpen={isOpen} onClose={onClose} size="large">
      <div className="settings-dialog">
        {/* 左侧标签页 */}
        <div className="settings-sidebar">
          {tabs.map(tab => (
            <button
              key={tab.id}
              className={`tab-button ${activeTab === tab.id ? 'active' : ''}`}
              onClick={() => setActiveTab(tab.id)}
            >
              <Icon name={tab.icon} />
              {tab.label}
            </button>
          ))}
        </div>
        
        {/* 右侧设置内容 */}
        <div className="settings-content">
          {activeTab === 'general' && <GeneralSettings />}
          {activeTab === 'editor' && <EditorSettings />}
          {activeTab === 'appearance' && <AppearanceSettings />}
          {activeTab === 'shortcuts' && <ShortcutsSettings />}
          {activeTab === 'plugins' && <PluginsSettings />}
          {activeTab === 'sync' && <SyncSettings />}
        </div>
      </div>
    </Dialog>
  );
};
```

**设置数据管理**:
```typescript
// stores/settingsStore.ts
export interface UserSettings {
  general: {
    language: string;
    autoSave: boolean;
    autoSaveInterval: number;
    defaultTemplate: string;
    knowledgeBasePath: string;
  };
  editor: {
    fontSize: number;
    fontFamily: string;
    lineHeight: number;
    tabSize: number;
    wordWrap: boolean;
    showLineNumbers: boolean;
    enableVimMode: boolean;
    enableSpellCheck: boolean;
  };
  appearance: {
    theme: 'light' | 'dark' | 'auto';
    accentColor: string;
    sidebarWidth: number;
    showStatusBar: boolean;
    compactMode: boolean;
  };
  shortcuts: Record<string, string>;
  plugins: {
    enabled: string[];
    config: Record<string, any>;
  };
  sync: {
    enabled: boolean;
    provider: 'git' | 'dropbox' | 'onedrive';
    autoSync: boolean;
    conflictResolution: 'manual' | 'latest' | 'merge';
  };
}

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set, get) => ({
      settings: defaultSettings,
      
      updateSettings: (updates: Partial<UserSettings>) => 
        set(state => ({
          settings: deepMerge(state.settings, updates)
        })),
      
      resetSettings: () => set({ settings: defaultSettings }),
      
      importSettings: (settings: UserSettings) => 
        set({ settings }),
      
      exportSettings: () => get().settings,
    }),
    {
      name: 'zeno-settings',
      version: 1,
    }
  )
);
```

**快捷键配置**:
```typescript
// components/Settings/ShortcutsSettings.tsx
export const ShortcutsSettings: React.FC = () => {
  const { settings, updateSettings } = useSettingsStore();
  const [editingShortcut, setEditingShortcut] = useState<string | null>(null);
  
  const shortcuts = [
    { id: 'newFile', label: '新建文件', default: 'Ctrl+N' },
    { id: 'openFile', label: '打开文件', default: 'Ctrl+O' },
    { id: 'saveFile', label: '保存文件', default: 'Ctrl+S' },
    { id: 'search', label: '搜索', default: 'Ctrl+F' },
    { id: 'quickOpen', label: '快速打开', default: 'Ctrl+P' },
    { id: 'togglePreview', label: '切换预览', default: 'Ctrl+Shift+V' },
    { id: 'toggleSidebar', label: '切换侧边栏', default: 'Ctrl+B' },
  ];
  
  const handleShortcutChange = (id: string, newShortcut: string) => {
    updateSettings({
      shortcuts: {
        ...settings.shortcuts,
        [id]: newShortcut,
      },
    });
  };
  
  return (
    <div className="shortcuts-settings">
      <h3>快捷键配置</h3>
      
      <div className="shortcuts-list">
        {shortcuts.map(shortcut => (
          <div key={shortcut.id} className="shortcut-item">
            <span className="shortcut-label">{shortcut.label}</span>
            <ShortcutInput
              value={settings.shortcuts[shortcut.id] || shortcut.default}
              onChange={(value) => handleShortcutChange(shortcut.id, value)}
              isEditing={editingShortcut === shortcut.id}
              onEdit={() => setEditingShortcut(shortcut.id)}
            />
          </div>
        ))}
      </div>
      
      <div className="settings-actions">
        <Button variant="outline" onClick={handleResetToDefaults}>
          恢复默认
        </Button>
        <Button onClick={handleImportShortcuts}>
          导入配置
        </Button>
        <Button onClick={handleExportShortcuts}>
          导出配置
        </Button>
      </div>
    </div>
  );
};
```

**验收标准**:
- [ ] 所有设置项正确保存和恢复
- [ ] 快捷键冲突检测和提示
- [ ] 设置导入导出功能正常
- [ ] 实时预览设置变化效果
- [ ] 设置搜索和分类清晰

## 集成和测试

### 3.5 用户体验测试

**可用性测试场景**:
1. **新用户上手流程**
   - 首次启动引导
   - 创建第一个笔记
   - 基础功能学习

2. **日常使用流程**
   - 快速创建和编辑笔记
   - 文件搜索和导航
   - 设置个性化配置

3. **高级功能流程**
   - 批量文件操作
   - 快捷键使用
   - 自定义主题

**性能测试**:
```typescript
// 性能指标监控
const PERFORMANCE_METRICS = {
  appStartup: 3000,        // 应用启动时间 < 3秒
  editorLoad: 500,         // 编辑器加载 < 500ms
  fileTreeRender: 200,     // 文件树渲染 < 200ms
  searchResponse: 100,     // 搜索响应 < 100ms
  themeSwitch: 100,        // 主题切换 < 100ms
};
```

## 里程碑和验收

### 第1周里程碑
- 编辑器基础功能实现
- 主界面布局完成
- 主题系统基础搭建

### 第2周里程碑
- 文件树组件完成
- 编辑器高级功能实现
- 响应式布局优化

### 第3周里程碑
- 设置系统完成
- 快捷键系统实现
- 用户体验优化

### 第4周里程碑
- 完整功能集成测试
- 性能优化和调试
- 无障碍和兼容性测试

### 最终验收标准
- [ ] 所有核心功能正常工作
- [ ] 用户界面响应流畅
- [ ] 跨平台兼容性良好
- [ ] 无障碍支持完整
- [ ] 用户测试反馈积极
- [ ] 性能指标达到要求

## 技术选型和依赖

### 前端技术栈
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "@codemirror/state": "^6.2.0",
    "@codemirror/view": "^6.9.0",
    "@codemirror/lang-markdown": "^6.1.0",
    "react-window": "^1.8.8",
    "framer-motion": "^10.12.0",
    "react-dnd": "^16.0.1",
    "react-hotkeys-hook": "^4.4.0",
    "zustand": "^4.3.8",
    "@radix-ui/react-dialog": "^1.0.3",
    "@radix-ui/react-dropdown-menu": "^2.0.4"
  }
}
```

### UI组件设计系统
- 基于Radix UI无头组件
- Tailwind CSS样式系统
- 自定义设计令牌
- 暗黑模式原生支持

## 下一阶段准备

### Phase 4 准备工作
- 确定知识图谱可视化库
- 设计双向链接交互规范
- 准备图形渲染性能优化

### 用户反馈收集
- 建立用户反馈收集机制
- 设置使用数据分析
- 准备A/B测试框架

---

**创建时间**: 2025-07-01  
**负责人**: 前端开发团队  
**状态**: 规划中  
**依赖**: Phase 2 数据管理层完成