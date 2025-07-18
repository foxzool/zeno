import React, { useState, useEffect, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import TyporaEditor, { TyporaEditorRef } from '../components/TyporaEditor';
import { useEditor } from '../contexts/EditorContext';

const EditorPage: React.FC = () => {
  const [searchParams] = useSearchParams();
  const { currentContent, setCurrentContent, currentFile, setCurrentFile, setEditorRef } = useEditor();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const editorRef = useRef<TyporaEditorRef>(null);

  // 将 editorRef 传递给 context
  useEffect(() => {
    setEditorRef(editorRef);
  }, [setEditorRef]);

  useEffect(() => {
    const loadFile = async () => {
      const filePath = searchParams.get('file');
      
      if (!filePath) {
        // 如果没有文件参数，显示默认内容
        setCurrentContent(getDefaultContent());
        setCurrentFile(null);
        setLoading(false);
        return;
      }

      try {
        setCurrentFile(filePath);
        const fileContent = await invoke<string>('read_file_content', { path: filePath });
        setCurrentContent(fileContent);
        setError(null);
        
        // 检查是否需要跳转到指定行
        const targetLine = searchParams.get('line');
        if (targetLine && editorRef.current) {
          // 延迟跳转，确保内容已经渲染
          setTimeout(() => {
            editorRef.current?.scrollToLine(parseInt(targetLine, 10));
          }, 100);
        }
      } catch (err) {
        console.error('加载文件失败:', err);
        setError(`加载文件失败: ${err}`);
        setCurrentContent(getDefaultContent());
      } finally {
        setLoading(false);
      }
    };

    loadFile();
  }, [searchParams, editorRef]);

  const getDefaultContent = () => {
    return `# 欢迎使用 Zeno 编辑器

这是一个强大的 Markdown 编辑器，支持：

## 功能特性

- **实时编辑**: 流畅的编辑体验
- **语法高亮**: Markdown 语法支持
- **自动保存**: 防止内容丢失
- **快捷键**: Ctrl+S 保存

## 开始写作

你可以开始在这里写作了...

### 代码示例

\`\`\`rust
fn main() {
    println!("Hello, Zeno!");
}
\`\`\`

### 列表

- 项目一
- 项目二
  - 子项目
  - 子项目

### 表格

| 功能 | 状态 |
|------|------|
| 编辑器 | ✅ 完成 |
| 文件树 | ✅ 完成 |
| 主题 | ✅ 完成 |

---

*开始你的知识管理之旅吧！*
`;
  };

  const handleSave = async () => {
    if (!currentFile) {
      console.log('没有当前文件，无法保存');
      return;
    }

    try {
      await invoke('write_file_content', { 
        path: currentFile, 
        content: currentContent 
      });
      console.log('文件保存成功');
      // 可以添加成功提示
    } catch (err) {
      console.error('保存文件失败:', err);
      setError(`保存文件失败: ${err}`);
    }
  };

  if (loading) {
    return (
      <div className="editor-page">
        <div className="loading">加载文件中...</div>
      </div>
    );
  }

  return (
    <div className="editor-page">
      <div className="editor-header">
        <h1 className="page-title">
          {currentFile ? `编辑: ${currentFile.split('/').pop()}` : '编辑器'}
        </h1>
        <p className="page-description">
          {currentFile ? currentFile : '专业的 Markdown 编辑体验'}
        </p>
      </div>
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      <div className="editor-wrapper">
        <TyporaEditor
          ref={editorRef}
          content={currentContent}
          onChange={setCurrentContent}
          onSave={handleSave}
          className="main-editor"
          placeholder="开始你的知识记录之旅..."
        />
      </div>
      
      <style>{`
        .editor-page {
          height: 100%;
          display: flex;
          flex-direction: column;
          gap: 24px;
        }
        
        .editor-header {
          flex-shrink: 0;
        }
        
        .page-title {
          margin: 0 0 8px 0;
          font-size: 28px;
          font-weight: 700;
          color: var(--text-primary, #1e293b);
        }
        
        .page-description {
          margin: 0;
          color: var(--text-secondary, #64748b);
          font-size: 16px;
        }
        
        .editor-wrapper {
          flex: 1;
          min-height: 0;
        }
        
        .editor-wrapper :global(.main-editor) {
          height: 100%;
        }
      `}</style>
    </div>
  );
};

export default EditorPage;