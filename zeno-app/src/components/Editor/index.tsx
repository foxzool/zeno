import React, { useEffect, useRef, useState } from 'react';

export interface EditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave?: () => void;
  readOnly?: boolean;
  className?: string;
}

export const Editor: React.FC<EditorProps> = ({
  content,
  onChange,
  onSave,
  readOnly = false,
  className = ''
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [isDirty, setIsDirty] = useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = e.target.value;
    onChange(newContent);
    setIsDirty(true);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      if (onSave) {
        onSave();
        setIsDirty(false);
      }
    }
  };

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = textareaRef.current.scrollHeight + 'px';
    }
  }, [content]);

  return (
    <div className={`editor-container ${className}`}>
      <div className="editor-toolbar">
        <div className="editor-actions">
          {isDirty && <span className="dirty-indicator">•</span>}
          <button 
            onClick={onSave}
            disabled={!isDirty}
            className="save-button"
          >
            保存 {isDirty && '(Ctrl+S)'}
          </button>
        </div>
      </div>
      
      <textarea
        ref={textareaRef}
        value={content}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        readOnly={readOnly}
        className="editor-textarea"
        placeholder="开始写作..."
        spellCheck={false}
      />
      
      <style>{`
        .editor-container {
          display: flex;
          flex-direction: column;
          height: 100%;
          border: 1px solid #e2e8f0;
          border-radius: 8px;
          overflow: hidden;
        }
        
        .editor-toolbar {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 16px;
          background: #f8fafc;
          border-bottom: 1px solid #e2e8f0;
        }
        
        .editor-actions {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        
        .dirty-indicator {
          color: #f59e0b;
          font-size: 18px;
          line-height: 1;
        }
        
        .save-button {
          padding: 4px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          background: white;
          color: #374151;
          font-size: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }
        
        .save-button:hover:not(:disabled) {
          background: #f3f4f6;
          border-color: #9ca3af;
        }
        
        .save-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
        
        .editor-textarea {
          flex: 1;
          padding: 16px;
          border: none;
          outline: none;
          resize: none;
          font-family: 'JetBrains Mono', 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
          font-size: 14px;
          line-height: 1.6;
          background: white;
          min-height: 200px;
        }
        
        .editor-textarea:focus {
          background: #fefefe;
        }
        
        @media (prefers-color-scheme: dark) {
          .editor-container {
            border-color: #374151;
          }
          
          .editor-toolbar {
            background: #1f2937;
            border-bottom-color: #374151;
          }
          
          .save-button {
            background: #374151;
            border-color: #6b7280;
            color: #f9fafb;
          }
          
          .save-button:hover:not(:disabled) {
            background: #4b5563;
            border-color: #9ca3af;
          }
          
          .editor-textarea {
            background: #111827;
            color: #f9fafb;
          }
          
          .editor-textarea:focus {
            background: #0f172a;
          }
        }
      `}</style>
    </div>
  );
};

export default Editor;