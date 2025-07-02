import React, { useState, useEffect, useRef, useCallback, useImperativeHandle, forwardRef } from 'react';
import ReactMarkdown from 'react-markdown';
import { useTheme } from '../ThemeProvider';

export interface TyporaEditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave?: () => void;
  className?: string;
  placeholder?: string;
}

export interface TyporaEditorRef {
  scrollToLine: (lineNumber: number) => void;
}

// 辅助函数：获取标题在内容中的行号
const getLineNumberForHeading = (content: string, headingText: string): number => {
  const lines = content.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const match = line.match(/^(#{1,6})\s+(.+)$/);
    if (match && match[2].trim() === headingText.trim()) {
      return i + 1;
    }
  }
  return 1;
};

const TyporaEditor = forwardRef<TyporaEditorRef, TyporaEditorProps>(({
  content,
  onChange,
  onSave,
  className = '',
  placeholder = '开始写作...'
}, ref) => {
  const { actualTheme } = useTheme();
  const [isEditing, setIsEditing] = useState(false);
  const [localContent, setLocalContent] = useState(content);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);

  // 同步外部内容
  useEffect(() => {
    setLocalContent(content);
  }, [content]);

  // 滚动到指定行
  const scrollToLine = useCallback((lineNumber: number) => {
    if (isEditing && textareaRef.current) {
      // 编辑模式：滚动到 textarea 中的指定行
      const lines = localContent.split('\n');
      let targetPosition = 0;
      
      for (let i = 0; i < Math.min(lineNumber - 1, lines.length); i++) {
        targetPosition += lines[i].length + 1; // +1 for newline
      }
      
      textareaRef.current.focus();
      textareaRef.current.setSelectionRange(targetPosition, targetPosition);
      textareaRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
    } else if (previewRef.current) {
      // 预览模式：查找对应的标题元素并滚动
      const headingElement = previewRef.current.querySelector(`[data-line="${lineNumber}"]`);
      
      if (headingElement) {
        headingElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
      } else {
        // 如果找不到精确的元素，尝试按比例滚动
        const totalLines = localContent.split('\n').length;
        const scrollPercentage = lineNumber / totalLines;
        const scrollTop = previewRef.current.scrollHeight * scrollPercentage;
        previewRef.current.scrollTo({ top: scrollTop, behavior: 'smooth' });
      }
    }
  }, [isEditing, localContent]);

  // 暴露方法给父组件
  useImperativeHandle(ref, () => ({
    scrollToLine
  }), [scrollToLine]);

  // 处理内容变化
  const handleContentChange = useCallback((newContent: string) => {
    setLocalContent(newContent);
    onChange(newContent);
  }, [onChange]);

  // 切换编辑模式
  const handleClick = useCallback(() => {
    setIsEditing(true);
    // 延迟聚焦以确保 textarea 已渲染
    setTimeout(() => {
      textareaRef.current?.focus();
    }, 0);
  }, []);

  // 失去焦点时切换到预览模式
  const handleBlur = useCallback(() => {
    setIsEditing(false);
  }, []);

  // 处理键盘事件
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      onSave?.();
    }
    
    // ESC 键退出编辑模式
    if (e.key === 'Escape') {
      setIsEditing(false);
    }
  }, [onSave]);

  // 全局键盘监听
  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        onSave?.();
      }
    };

    document.addEventListener('keydown', handleGlobalKeyDown);
    return () => document.removeEventListener('keydown', handleGlobalKeyDown);
  }, [onSave]);

  return (
    <div 
      ref={containerRef}
      className={`typora-editor ${actualTheme} ${className}`}
      data-theme={actualTheme}
    >
      {isEditing ? (
        <textarea
          ref={textareaRef}
          value={localContent}
          onChange={(e) => handleContentChange(e.target.value)}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="editor-textarea"
        />
      ) : (
        <div 
          ref={previewRef}
          className="editor-preview"
          onClick={handleClick}
        >
          {localContent ? (
            <ReactMarkdown 
              components={{
                // 自定义组件渲染，为标题添加行号信息
                h1: ({node, children, ...props}) => {
                  const lineNumber = getLineNumberForHeading(localContent, String(children));
                  return <h1 className="md-h1" data-line={lineNumber} {...props}>{children}</h1>;
                },
                h2: ({node, children, ...props}) => {
                  const lineNumber = getLineNumberForHeading(localContent, String(children));
                  return <h2 className="md-h2" data-line={lineNumber} {...props}>{children}</h2>;
                },
                h3: ({node, children, ...props}) => {
                  const lineNumber = getLineNumberForHeading(localContent, String(children));
                  return <h3 className="md-h3" data-line={lineNumber} {...props}>{children}</h3>;
                },
                h4: ({node, children, ...props}) => {
                  const lineNumber = getLineNumberForHeading(localContent, String(children));
                  return <h4 className="md-h4" data-line={lineNumber} {...props}>{children}</h4>;
                },
                h5: ({node, children, ...props}) => {
                  const lineNumber = getLineNumberForHeading(localContent, String(children));
                  return <h5 className="md-h5" data-line={lineNumber} {...props}>{children}</h5>;
                },
                h6: ({node, children, ...props}) => {
                  const lineNumber = getLineNumberForHeading(localContent, String(children));
                  return <h6 className="md-h6" data-line={lineNumber} {...props}>{children}</h6>;
                },
                p: ({node, ...props}) => <p className="md-p" {...props} />,
                blockquote: ({node, ...props}) => <blockquote className="md-blockquote" {...props} />,
                code: ({node, className, children, ...props}) => {
                  const isInline = !className?.includes('language-');
                  return isInline ? 
                    <code className="md-code-inline" {...props}>{children}</code> : 
                    <code className="md-code-block" {...props}>{children}</code>;
                },
                pre: ({node, ...props}) => <pre className="md-pre" {...props} />,
                ul: ({node, ...props}) => <ul className="md-ul" {...props} />,
                ol: ({node, ...props}) => <ol className="md-ol" {...props} />,
                li: ({node, ...props}) => <li className="md-li" {...props} />,
                table: ({node, ...props}) => <table className="md-table" {...props} />,
                th: ({node, ...props}) => <th className="md-th" {...props} />,
                td: ({node, ...props}) => <td className="md-td" {...props} />,
              }}
            >
              {localContent}
            </ReactMarkdown>
          ) : (
            <div className="editor-placeholder">
              {placeholder}
            </div>
          )}
        </div>
      )}
      
      <style>{`
        .typora-editor {
          height: 100%;
          display: flex;
          flex-direction: column;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
            'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
        }
        
        .editor-textarea {
          flex: 1;
          padding: 20px;
          border: none;
          outline: none;
          background: var(--bg-primary, white);
          color: var(--text-primary, #1e293b);
          font-size: 16px;
          line-height: 1.7;
          resize: none;
          font-family: inherit;
        }
        
        .editor-preview {
          flex: 1;
          padding: 20px;
          cursor: text;
          background: var(--bg-primary, white);
          color: var(--text-primary, #1e293b);
          font-size: 16px;
          line-height: 1.7;
          overflow-y: auto;
          min-height: 400px;
        }
        
        .editor-placeholder {
          color: var(--text-tertiary, #94a3b8);
          font-style: italic;
        }
        
        /* 深色主题适配 */
        .typora-editor.dark .editor-textarea,
        .typora-editor.dark .editor-preview {
          background: var(--bg-primary, #0a0f1b);
          color: var(--text-primary, #e8eaed);
        }
        
        /* Markdown 元素样式 */
        .typora-editor .md-h1 {
          font-size: 2rem;
          font-weight: 700;
          margin: 1.5rem 0 1rem 0;
          color: var(--text-primary, #1e293b);
          border-bottom: 2px solid var(--border-primary, #e2e8f0);
          padding-bottom: 0.5rem;
        }
        
        .typora-editor .md-h2 {
          font-size: 1.5rem;
          font-weight: 600;
          margin: 1.2rem 0 0.8rem 0;
          color: var(--text-primary, #1e293b);
        }
        
        .typora-editor .md-h3 {
          font-size: 1.25rem;
          font-weight: 600;
          margin: 1rem 0 0.6rem 0;
          color: var(--text-primary, #1e293b);
        }
        
        .typora-editor .md-p {
          margin: 0.8rem 0;
          color: var(--text-primary, #1e293b);
        }
        
        .typora-editor .md-blockquote {
          border-left: 4px solid var(--accent-primary, #3b82f6);
          padding-left: 1rem;
          margin: 1rem 0;
          background: var(--bg-secondary, #f8fafc);
          color: var(--text-secondary, #64748b);
          font-style: italic;
        }
        
        .typora-editor .md-code-inline {
          background: var(--bg-tertiary, #f1f5f9);
          color: var(--accent-primary, #3b82f6);
          padding: 0.2rem 0.4rem;
          border-radius: 0.25rem;
          font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
          font-size: 0.875em;
        }
        
        .typora-editor .md-pre {
          background: var(--bg-tertiary, #f1f5f9);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.5rem;
          padding: 1rem;
          margin: 1rem 0;
          overflow-x: auto;
        }
        
        .typora-editor .md-code-block {
          background: none;
          color: var(--text-primary, #1e293b);
          padding: 0;
          font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
        }
        
        .typora-editor .md-ul, 
        .typora-editor .md-ol {
          margin: 0.8rem 0;
          padding-left: 1.5rem;
        }
        
        .typora-editor .md-li {
          margin: 0.3rem 0;
          color: var(--text-primary, #1e293b);
        }
        
        .typora-editor .md-table {
          border-collapse: collapse;
          width: 100%;
          margin: 1rem 0;
        }
        
        .typora-editor .md-th,
        .typora-editor .md-td {
          border: 1px solid var(--border-primary, #e2e8f0);
          padding: 0.5rem 0.75rem;
          text-align: left;
        }
        
        .typora-editor .md-th {
          background: var(--bg-secondary, #f8fafc);
          font-weight: 600;
        }
        
        /* 深色主题下的样式调整 */
        .typora-editor.dark .md-h1,
        .typora-editor.dark .md-h2,
        .typora-editor.dark .md-h3,
        .typora-editor.dark .md-p,
        .typora-editor.dark .md-li {
          color: var(--text-primary, #e8eaed);
        }
        
        .typora-editor.dark .md-h1 {
          border-bottom-color: var(--border-primary, rgba(255, 255, 255, 0.08));
        }
        
        .typora-editor.dark .md-blockquote {
          background: var(--bg-secondary, #141925);
          color: var(--text-secondary, #9aa0a6);
        }
        
        .typora-editor.dark .md-code-inline {
          background: var(--bg-tertiary, #1e2531);
          color: var(--accent-primary, #5b8dee);
        }
        
        .typora-editor.dark .md-pre {
          background: var(--bg-tertiary, #1e2531);
          border-color: var(--border-primary, rgba(255, 255, 255, 0.08));
        }
        
        .typora-editor.dark .md-code-block {
          color: var(--text-primary, #e8eaed);
        }
        
        .typora-editor.dark .md-th {
          background: var(--bg-secondary, #141925);
        }
        
        .typora-editor.dark .md-th,
        .typora-editor.dark .md-td {
          border-color: var(--border-primary, rgba(255, 255, 255, 0.08));
        }
        
        /* 编辑器焦点样式 */
        .typora-editor .editor-textarea:focus {
          outline: none;
        }
        
        /* 选择区域样式 */
        .typora-editor .editor-textarea ::selection,
        .typora-editor .editor-preview ::selection {
          background: var(--accent-primary, #3b82f6);
          background-opacity: 0.3;
          color: var(--text-primary, #1e293b);
        }
        
        .typora-editor.dark .editor-textarea ::selection,
        .typora-editor.dark .editor-preview ::selection {
          background: var(--accent-primary, #5b8dee);
          background-opacity: 0.3;
          color: var(--text-primary, #e8eaed);
        }
        
        /* 滚动条样式 */
        .typora-editor .editor-textarea::-webkit-scrollbar,
        .typora-editor .editor-preview::-webkit-scrollbar {
          width: 8px;
        }
        
        .typora-editor .editor-textarea::-webkit-scrollbar-track,
        .typora-editor .editor-preview::-webkit-scrollbar-track {
          background: var(--bg-secondary, #f8fafc);
        }
        
        .typora-editor .editor-textarea::-webkit-scrollbar-thumb,
        .typora-editor .editor-preview::-webkit-scrollbar-thumb {
          background: var(--border-primary, #e2e8f0);
          border-radius: 4px;
        }
        
        .typora-editor.dark .editor-textarea::-webkit-scrollbar-track,
        .typora-editor.dark .editor-preview::-webkit-scrollbar-track {
          background: var(--bg-secondary, #141925);
        }
        
        .typora-editor.dark .editor-textarea::-webkit-scrollbar-thumb,
        .typora-editor.dark .editor-preview::-webkit-scrollbar-thumb {
          background: var(--border-primary, rgba(255, 255, 255, 0.08));
        }
      `}</style>
    </div>
  );
});

TyporaEditor.displayName = 'TyporaEditor';

export default TyporaEditor;