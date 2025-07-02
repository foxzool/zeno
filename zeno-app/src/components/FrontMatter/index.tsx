import React from 'react';
import { Calendar, User, Tag, FileText, Hash, ExternalLink } from 'lucide-react';

interface FrontMatterData {
  title?: string;
  author?: string;
  date?: string;
  tags?: string[] | string;
  category?: string;
  description?: string;
  slug?: string;
  draft?: boolean;
  [key: string]: any;
}

interface FrontMatterProps {
  yamlContent: string;
  onEdit?: () => void;
}

const FrontMatter: React.FC<FrontMatterProps> = ({ yamlContent, onEdit }) => {
  const parseYAML = (content: string): FrontMatterData => {
    try {
      const data: FrontMatterData = {};
      
      // 简单的 YAML 解析器（适用于常见的 front matter 格式）
      const lines = content.split('\n').filter(line => line.trim());
      
      for (const line of lines) {
        if (line.includes(':')) {
          const [key, ...valueParts] = line.split(':');
          const value = valueParts.join(':').trim();
          
          const cleanKey = key.trim();
          let cleanValue: any = value;
          
          // 处理引号
          if ((value.startsWith('"') && value.endsWith('"')) || 
              (value.startsWith("'") && value.endsWith("'"))) {
            cleanValue = value.slice(1, -1);
          }
          
          // 处理数组格式 [tag1, tag2] 或 ["tag1", "tag2"]
          if (value.startsWith('[') && value.endsWith(']')) {
            const arrayContent = value.slice(1, -1);
            cleanValue = arrayContent.split(',').map(item => 
              item.trim().replace(/^["']|["']$/g, '')
            ).filter(item => item);
          }
          
          // 处理布尔值
          if (value.toLowerCase() === 'true') cleanValue = true;
          if (value.toLowerCase() === 'false') cleanValue = false;
          
          data[cleanKey] = cleanValue;
        }
      }
      
      return data;
    } catch (error) {
      console.error('YAML 解析失败:', error);
      return {};
    }
  };

  const data = parseYAML(yamlContent);

  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: 'long',
        day: 'numeric'
      });
    } catch {
      return dateStr;
    }
  };

  const formatTags = (tags: string[] | string): string[] => {
    if (Array.isArray(tags)) {
      return tags;
    }
    if (typeof tags === 'string') {
      // 处理逗号分隔的标签
      return tags.split(',').map(tag => tag.trim()).filter(tag => tag);
    }
    return [];
  };

  const renderField = (key: string, value: any, icon: React.ReactNode) => {
    if (!value) return null;

    return (
      <div className="frontmatter-field">
        <div className="field-icon">{icon}</div>
        <div className="field-content">
          <span className="field-label">{key}:</span>
          <span className="field-value">{value}</span>
        </div>
      </div>
    );
  };

  const renderTags = (tags: string[] | string) => {
    const tagList = formatTags(tags);
    if (tagList.length === 0) return null;

    return (
      <div className="frontmatter-field">
        <div className="field-icon"><Tag size={14} /></div>
        <div className="field-content">
          <span className="field-label">标签:</span>
          <div className="tags-container">
            {tagList.map((tag, index) => (
              <span key={index} className="tag-item">
                {tag}
              </span>
            ))}
          </div>
        </div>
      </div>
    );
  };

  // 如果没有解析到任何数据，返回原始内容
  if (Object.keys(data).length === 0) {
    return (
      <div className="frontmatter-raw" onClick={onEdit}>
        <pre className="yaml-content">{yamlContent}</pre>
      </div>
    );
  }

  return (
    <div className="frontmatter-display">
      <div className="frontmatter-header">
        <Hash size={16} />
        <span className="frontmatter-title">文档信息</span>
        {onEdit && (
          <button onClick={onEdit} className="edit-button" title="编辑 Front Matter">
            <FileText size={14} />
          </button>
        )}
      </div>

      <div className="frontmatter-content">
        {/* 标题 */}
        {data.title && (
          <div className="frontmatter-field title-field">
            <h2 className="document-title">{data.title}</h2>
            {data.draft && (
              <span className="draft-badge">草稿</span>
            )}
          </div>
        )}

        {/* 描述 */}
        {data.description && (
          <div className="frontmatter-field">
            <p className="document-description">{data.description}</p>
          </div>
        )}

        {/* 基本信息 */}
        <div className="frontmatter-fields">
          {renderField('作者', data.author, <User size={14} />)}
          {data.date && renderField('日期', formatDate(data.date), <Calendar size={14} />)}
          {renderField('分类', data.category, <FileText size={14} />)}
          {data.slug && renderField('链接', data.slug, <ExternalLink size={14} />)}
          
          {/* 标签 */}
          {(data.tags || data.tag) && renderTags(data.tags || data.tag)}

          {/* 其他自定义字段 */}
          {Object.entries(data)
            .filter(([key]) => !['title', 'author', 'date', 'tags', 'tag', 'category', 'description', 'slug', 'draft'].includes(key))
            .map(([key, value]) => renderField(key, String(value), <Hash size={14} />))
          }
        </div>
      </div>

      <style>{`
        .frontmatter-display {
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.5rem;
          margin-bottom: 1.5rem;
          overflow: hidden;
        }

        .frontmatter-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          background: var(--bg-tertiary, #f1f5f9);
          border-bottom: 1px solid var(--border-primary, #e2e8f0);
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--text-secondary, #64748b);
        }

        .frontmatter-title {
          flex: 1;
        }

        .edit-button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 24px;
          height: 24px;
          border: none;
          background: transparent;
          color: var(--text-tertiary, #94a3b8);
          border-radius: 0.25rem;
          cursor: pointer;
          transition: all 0.2s;
        }

        .edit-button:hover {
          background: var(--bg-primary, white);
          color: var(--text-primary, #1e293b);
        }

        .frontmatter-content {
          padding: 1rem;
        }

        .title-field {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          margin-bottom: 1rem;
        }

        .document-title {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 700;
          color: var(--text-primary, #1e293b);
          flex: 1;
        }

        .draft-badge {
          background: var(--warning-bg, #fef3c7);
          color: var(--warning-text, #92400e);
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-size: 0.75rem;
          font-weight: 600;
          text-transform: uppercase;
        }

        .document-description {
          margin: 0 0 1rem 0;
          color: var(--text-secondary, #64748b);
          font-style: italic;
          line-height: 1.5;
        }

        .frontmatter-fields {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .frontmatter-field {
          display: flex;
          align-items: flex-start;
          gap: 0.5rem;
          min-height: 1.5rem;
        }

        .field-icon {
          display: flex;
          align-items: center;
          margin-top: 0.125rem;
          color: var(--text-tertiary, #94a3b8);
          flex-shrink: 0;
        }

        .field-content {
          display: flex;
          align-items: flex-start;
          gap: 0.5rem;
          flex: 1;
        }

        .field-label {
          font-weight: 500;
          color: var(--text-secondary, #64748b);
          min-width: 4rem;
          font-size: 0.875rem;
        }

        .field-value {
          color: var(--text-primary, #1e293b);
          font-size: 0.875rem;
          line-height: 1.4;
        }

        .tags-container {
          display: flex;
          flex-wrap: wrap;
          gap: 0.375rem;
        }

        .tag-item {
          background: var(--accent-bg, #dbeafe);
          color: var(--accent-text, #1e40af);
          padding: 0.125rem 0.5rem;
          border-radius: 1rem;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .frontmatter-raw {
          background: var(--bg-secondary, #f8fafc);
          border: 1px solid var(--border-primary, #e2e8f0);
          border-radius: 0.5rem;
          margin-bottom: 1.5rem;
          cursor: pointer;
          transition: all 0.2s;
        }

        .frontmatter-raw:hover {
          border-color: var(--accent-primary, #3b82f6);
        }

        .yaml-content {
          margin: 0;
          padding: 1rem;
          font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
          font-size: 0.875rem;
          line-height: 1.5;
          color: var(--text-primary, #1e293b);
          background: transparent;
          border: none;
          white-space: pre-wrap;
          word-wrap: break-word;
        }

        /* 深色主题适配 */
        .typora-editor.dark .frontmatter-display {
          background: var(--bg-secondary, #141925);
          border-color: var(--border-primary, rgba(255, 255, 255, 0.08));
        }

        .typora-editor.dark .frontmatter-header {
          background: var(--bg-tertiary, #1e2531);
          border-color: var(--border-primary, rgba(255, 255, 255, 0.08));
          color: var(--text-secondary, #9aa0a6);
        }

        .typora-editor.dark .document-title {
          color: var(--text-primary, #e8eaed);
        }

        .typora-editor.dark .document-description {
          color: var(--text-secondary, #9aa0a6);
        }

        .typora-editor.dark .field-label {
          color: var(--text-secondary, #9aa0a6);
        }

        .typora-editor.dark .field-value {
          color: var(--text-primary, #e8eaed);
        }

        .typora-editor.dark .tag-item {
          background: var(--accent-bg, #1e3a8a);
          color: var(--accent-text, #93c5fd);
        }

        .typora-editor.dark .frontmatter-raw {
          background: var(--bg-secondary, #141925);
          border-color: var(--border-primary, rgba(255, 255, 255, 0.08));
        }

        .typora-editor.dark .yaml-content {
          color: var(--text-primary, #e8eaed);
        }
      `}</style>
    </div>
  );
};

export default FrontMatter;