import React, { useState } from 'react';
import Editor from '../components/Editor';

const EditorPage: React.FC = () => {
  const [content, setContent] = useState(`# 欢迎使用 Zeno 编辑器

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
`);

  const handleSave = () => {
    console.log('保存内容:', content);
    // TODO: 调用 Tauri 命令保存文件
  };

  return (
    <div className="editor-page">
      <div className="editor-header">
        <h1 className="page-title">编辑器</h1>
        <p className="page-description">专业的 Markdown 编辑体验</p>
      </div>
      
      <div className="editor-wrapper">
        <Editor
          content={content}
          onChange={setContent}
          onSave={handleSave}
          className="main-editor"
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