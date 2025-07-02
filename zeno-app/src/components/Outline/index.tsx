import React, { useState, useEffect } from 'react';
import { Hash } from 'lucide-react';

export interface OutlineItem {
  id: string;
  text: string;
  level: number; // 1-6 对应 h1-h6
  line: number;
}

interface OutlineProps {
  content: string;
  onItemClick?: (item: OutlineItem) => void;
}

const Outline: React.FC<OutlineProps> = ({ content, onItemClick }) => {
  const [outlineItems, setOutlineItems] = useState<OutlineItem[]>([]);

  useEffect(() => {
    if (!content) {
      setOutlineItems([]);
      return;
    }

    const lines = content.split('\n');
    const items: OutlineItem[] = [];

    lines.forEach((line, index) => {
      // 匹配 Markdown 标题格式 (# ## ### 等)
      const match = line.match(/^(#{1,6})\s+(.+)$/);
      if (match) {
        const level = match[1].length;
        const text = match[2].trim();
        const id = `heading-${index}-${text.toLowerCase().replace(/[^\w\u4e00-\u9fa5]+/g, '-')}`;
        
        items.push({
          id,
          text,
          level,
          line: index + 1
        });
      }
    });

    setOutlineItems(items);
  }, [content]);

  const handleItemClick = (item: OutlineItem) => {
    onItemClick?.(item);
  };

  if (outlineItems.length === 0) {
    return (
      <div className="outline-empty">
        <Hash size={20} className="outline-empty-icon" />
        <p className="outline-empty-text">
          在文档中添加标题以生成大纲
        </p>
        <p className="outline-empty-hint">
          使用 # ## ### 来创建标题
        </p>
      </div>
    );
  }

  return (
    <div className="outline">
      <div className="outline-list">
        {outlineItems.map((item) => (
          <div
            key={item.id}
            className={`outline-item level-${item.level}`}
            onClick={() => handleItemClick(item)}
            title={`第 ${item.line} 行: ${item.text}`}
          >
            <div className="outline-item-content">
              <span className="outline-item-text">{item.text}</span>
              <span className="outline-item-line">L{item.line}</span>
            </div>
          </div>
        ))}
      </div>

      <style>{`
        .outline {
          height: 100%;
          overflow-y: auto;
        }

        .outline-empty {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
          text-align: center;
          color: var(--text-tertiary, #94a3b8);
        }

        .outline-empty-icon {
          margin-bottom: 12px;
          opacity: 0.5;
        }

        .outline-empty-text {
          margin: 0 0 4px 0;
          font-size: 13px;
          font-weight: 500;
        }

        .outline-empty-hint {
          margin: 0;
          font-size: 12px;
          opacity: 0.7;
        }

        .outline-list {
          padding: 8px 0;
        }

        .outline-item {
          padding: 6px 12px;
          cursor: pointer;
          border-radius: 4px;
          margin: 2px 8px;
          transition: all 0.2s;
          border-left: 2px solid transparent;
        }

        .outline-item:hover {
          background-color: var(--bg-secondary, #f8fafc);
          border-left-color: var(--accent-primary, #3b82f6);
        }

        .outline-item.level-1 {
          margin-left: 8px;
          font-weight: 600;
          font-size: 14px;
        }

        .outline-item.level-2 {
          margin-left: 20px;
          font-weight: 500;
          font-size: 13px;
        }

        .outline-item.level-3 {
          margin-left: 32px;
          font-size: 13px;
        }

        .outline-item.level-4 {
          margin-left: 44px;
          font-size: 12px;
        }

        .outline-item.level-5 {
          margin-left: 56px;
          font-size: 12px;
          opacity: 0.8;
        }

        .outline-item.level-6 {
          margin-left: 68px;
          font-size: 11px;
          opacity: 0.7;
        }

        .outline-item-content {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 8px;
        }

        .outline-item-text {
          flex: 1;
          color: var(--text-primary, #1e293b);
          line-height: 1.4;
          word-break: break-word;
        }

        .outline-item-line {
          font-size: 10px;
          color: var(--text-tertiary, #94a3b8);
          opacity: 0.6;
          font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
          flex-shrink: 0;
        }

        .outline-item:hover .outline-item-line {
          opacity: 1;
        }

        /* 滚动条样式 */
        .outline::-webkit-scrollbar {
          width: 6px;
        }

        .outline::-webkit-scrollbar-track {
          background: transparent;
        }

        .outline::-webkit-scrollbar-thumb {
          background: var(--border-primary, #e2e8f0);
          border-radius: 3px;
        }

        .outline::-webkit-scrollbar-thumb:hover {
          background: var(--text-tertiary, #94a3b8);
        }
      `}</style>
    </div>
  );
};

export default Outline;