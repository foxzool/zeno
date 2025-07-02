import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface WeChatConfig {
  account_name: string;
  app_id: string;
  app_secret: string;
  access_token?: string;
  token_expires_at?: string;
  enabled: boolean;
  default_settings: WeChatPublishSettings;
}

interface WeChatPublishSettings {
  publish_immediately: boolean;
  open_comment: boolean;
  only_fans_can_comment: boolean;
  thumb_media_id?: string;
  author: string;
  content_source_url?: string;
  digest?: string;
  show_cover_pic: boolean;
  extra_fields: { [key: string]: string };
}

interface WeChatPublishResult {
  success: boolean;
  media_id?: string;
  published_at: string;
  preview_url?: string;
  error_message?: string;
  note_title: string;
  processing_time_ms: number;
}

interface WeChatStats {
  total_articles: number;
  monthly_articles: number;
  daily_articles: number;
  total_media_files: number;
  total_media_size: number;
  average_publish_time: number;
  last_publish_time?: string;
  token_status: 'Valid' | 'Expiring' | 'Expired' | 'NotConfigured' | 'ConfigError';
}

interface ValidationResult {
  is_valid: boolean;
  warnings: string[];
  errors: string[];
  suggestions: string[];
  content_stats: {
    character_count: number;
    word_count: number;
    image_count: number;
    link_count: number;
    estimated_read_time: number;
  };
}

interface WeChatPublisherProps {
  onConfigChange?: (config: WeChatConfig) => void;
}

const WeChatPublisher: React.FC<WeChatPublisherProps> = ({ onConfigChange }) => {
  const [config, setConfig] = useState<WeChatConfig | null>(null);
  const [isConfiguring, setIsConfiguring] = useState(false);
  const [isPublishing, setIsPublishing] = useState(false);
  const [publishResults, setPublishResults] = useState<WeChatPublishResult[]>([]);
  const [stats, setStats] = useState<WeChatStats | null>(null);
  const [previewContent, setPreviewContent] = useState<string>('');
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [activeTab, setActiveTab] = useState<'config' | 'publish' | 'preview' | 'stats'>('config');

  useEffect(() => {
    loadConfig();
  }, []);

  useEffect(() => {
    if (config && config.enabled) {
      loadStats();
    }
  }, [config]);

  const loadConfig = async () => {
    try {
      const wechatConfig = await invoke<WeChatConfig>('get_wechat_config');
      setConfig(wechatConfig);
    } catch (error) {
      console.error('Failed to load WeChat config:', error);
      const defaultConfig = await invoke<WeChatConfig>('create_default_wechat_config');
      setConfig(defaultConfig);
    }
  };

  const loadStats = async () => {
    if (!config) return;
    
    try {
      const wechatStats = await invoke<WeChatStats>('get_wechat_stats', { config });
      setStats(wechatStats);
    } catch (error) {
      console.error('Failed to load WeChat stats:', error);
    }
  };

  const saveConfig = async () => {
    if (!config) return;

    try {
      setIsConfiguring(true);
      await invoke('save_wechat_config', { newConfig: config });
      onConfigChange?.(config);
      alert('微信配置保存成功！');
    } catch (error) {
      console.error('Failed to save WeChat config:', error);
      alert(`保存配置失败: ${error}`);
    } finally {
      setIsConfiguring(false);
    }
  };

  const testConfig = async () => {
    if (!config) return;

    try {
      setIsConfiguring(true);
      const isValid = await invoke<boolean>('test_wechat_config', { config });
      
      if (isValid) {
        alert('微信配置测试成功！');
        loadStats();
      } else {
        alert('微信配置测试失败，请检查 AppID 和 AppSecret');
      }
    } catch (error) {
      console.error('Failed to test WeChat config:', error);
      alert(`配置测试失败: ${error}`);
    } finally {
      setIsConfiguring(false);
    }
  };

  const publishNote = async (notePath: string) => {
    if (!config) return;

    try {
      setIsPublishing(true);
      const result = await invoke<WeChatPublishResult>('publish_note_to_wechat', {
        notePath,
        config,
        settings: config.default_settings,
      });
      
      setPublishResults(prev => [result, ...prev]);
      
      if (result.success) {
        alert(`笔记 "${result.note_title}" 发布成功！`);
      } else {
        alert(`发布失败: ${result.error_message}`);
      }
      
      loadStats();
    } catch (error) {
      console.error('Failed to publish note:', error);
      alert(`发布失败: ${error}`);
    } finally {
      setIsPublishing(false);
    }
  };

  const previewNote = async (notePath: string) => {
    if (!config) return;

    try {
      const content = await invoke<string>('preview_wechat_content', {
        notePath,
        config,
      });
      setPreviewContent(content);
      setActiveTab('preview');
    } catch (error) {
      console.error('Failed to preview note:', error);
      alert(`预览失败: ${error}`);
    }
  };

  const validateContent = async (content: string) => {
    try {
      const result = await invoke<ValidationResult>('validate_wechat_content', { content });
      setValidationResult(result);
    } catch (error) {
      console.error('Failed to validate content:', error);
    }
  };

  const refreshToken = async () => {
    try {
      await invoke('refresh_wechat_token');
      alert('访问令牌刷新成功！');
      loadStats();
    } catch (error) {
      console.error('Failed to refresh token:', error);
      alert(`刷新令牌失败: ${error}`);
    }
  };

  const updateConfigField = (field: keyof WeChatConfig, value: any) => {
    if (!config) return;
    
    setConfig({
      ...config,
      [field]: value,
    });
  };

  const updateSettingsField = (field: keyof WeChatPublishSettings, value: any) => {
    if (!config) return;
    
    setConfig({
      ...config,
      default_settings: {
        ...config.default_settings,
        [field]: value,
      },
    });
  };

  const getTokenStatusColor = (status: string) => {
    switch (status) {
      case 'Valid': return '#52c41a';
      case 'Expiring': return '#faad14';
      case 'Expired': return '#f5222d';
      case 'NotConfigured': return '#d9d9d9';
      case 'ConfigError': return '#f5222d';
      default: return '#d9d9d9';
    }
  };

  const getTokenStatusText = (status: string) => {
    switch (status) {
      case 'Valid': return '有效';
      case 'Expiring': return '即将过期';
      case 'Expired': return '已过期';
      case 'NotConfigured': return '未配置';
      case 'ConfigError': return '配置错误';
      default: return '未知';
    }
  };

  const formatFileSize = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  if (!config) {
    return (
      <div className="wechat-publisher">
        <div className="loading">加载微信配置中...</div>
      </div>
    );
  }

  return (
    <div className="wechat-publisher">
      <div className="publisher-header">
        <h2>微信公众号发布</h2>
        <p>将笔记发布到微信公众号平台</p>
        
        {stats && (
          <div className="token-status" style={{ color: getTokenStatusColor(stats.token_status) }}>
            令牌状态: {getTokenStatusText(stats.token_status)}
            {stats.token_status === 'Expiring' || stats.token_status === 'Expired' ? (
              <button onClick={refreshToken} className="refresh-token-btn">
                刷新令牌
              </button>
            ) : null}
          </div>
        )}
      </div>

      <div className="publisher-tabs">
        <button
          className={`tab ${activeTab === 'config' ? 'active' : ''}`}
          onClick={() => setActiveTab('config')}
        >
          配置
        </button>
        <button
          className={`tab ${activeTab === 'publish' ? 'active' : ''}`}
          onClick={() => setActiveTab('publish')}
        >
          发布
        </button>
        <button
          className={`tab ${activeTab === 'preview' ? 'active' : ''}`}
          onClick={() => setActiveTab('preview')}
        >
          预览
        </button>
        <button
          className={`tab ${activeTab === 'stats' ? 'active' : ''}`}
          onClick={() => setActiveTab('stats')}
        >
          统计
        </button>
      </div>

      {activeTab === 'config' && (
        <div className="config-section">
          <h3>基本配置</h3>
          
          <div className="form-group">
            <label>公众号名称:</label>
            <input
              type="text"
              value={config.account_name}
              onChange={(e) => updateConfigField('account_name', e.target.value)}
              placeholder="输入公众号名称"
            />
          </div>

          <div className="form-group">
            <label>AppID:</label>
            <input
              type="text"
              value={config.app_id}
              onChange={(e) => updateConfigField('app_id', e.target.value)}
              placeholder="输入微信公众号 AppID"
            />
          </div>

          <div className="form-group">
            <label>AppSecret:</label>
            <input
              type="password"
              value={config.app_secret}
              onChange={(e) => updateConfigField('app_secret', e.target.value)}
              placeholder="输入微信公众号 AppSecret"
            />
          </div>

          <div className="form-group">
            <label>
              <input
                type="checkbox"
                checked={config.enabled}
                onChange={(e) => updateConfigField('enabled', e.target.checked)}
              />
              启用微信发布
            </label>
          </div>

          <h3>默认发布设置</h3>

          <div className="form-group">
            <label>作者:</label>
            <input
              type="text"
              value={config.default_settings.author}
              onChange={(e) => updateSettingsField('author', e.target.value)}
              placeholder="输入作者名称"
            />
          </div>

          <div className="form-group">
            <label>原文链接:</label>
            <input
              type="url"
              value={config.default_settings.content_source_url || ''}
              onChange={(e) => updateSettingsField('content_source_url', e.target.value)}
              placeholder="输入原文链接 (可选)"
            />
          </div>

          <div className="checkbox-group">
            <label>
              <input
                type="checkbox"
                checked={config.default_settings.publish_immediately}
                onChange={(e) => updateSettingsField('publish_immediately', e.target.checked)}
              />
              立即发布
            </label>

            <label>
              <input
                type="checkbox"
                checked={config.default_settings.open_comment}
                onChange={(e) => updateSettingsField('open_comment', e.target.checked)}
              />
              开启评论
            </label>

            <label>
              <input
                type="checkbox"
                checked={config.default_settings.only_fans_can_comment}
                onChange={(e) => updateSettingsField('only_fans_can_comment', e.target.checked)}
              />
              仅粉丝可评论
            </label>

            <label>
              <input
                type="checkbox"
                checked={config.default_settings.show_cover_pic}
                onChange={(e) => updateSettingsField('show_cover_pic', e.target.checked)}
              />
              显示封面图
            </label>
          </div>

          <div className="config-actions">
            <button
              onClick={testConfig}
              disabled={isConfiguring}
              className="btn-secondary"
            >
              {isConfiguring ? '测试中...' : '测试配置'}
            </button>

            <button
              onClick={saveConfig}
              disabled={isConfiguring}
              className="btn-primary"
            >
              {isConfiguring ? '保存中...' : '保存配置'}
            </button>
          </div>
        </div>
      )}

      {activeTab === 'publish' && (
        <div className="publish-section">
          <h3>发布管理</h3>
          
          <div className="publish-note-form">
            <input
              type="text"
              placeholder="输入笔记路径"
              id="note-path-input"
            />
            <button
              onClick={() => {
                const input = document.getElementById('note-path-input') as HTMLInputElement;
                if (input.value) {
                  publishNote(input.value);
                }
              }}
              disabled={isPublishing || !config.enabled}
              className="btn-primary"
            >
              {isPublishing ? '发布中...' : '发布笔记'}
            </button>
            <button
              onClick={() => {
                const input = document.getElementById('note-path-input') as HTMLInputElement;
                if (input.value) {
                  previewNote(input.value);
                }
              }}
              disabled={!config.enabled}
              className="btn-secondary"
            >
              预览转换
            </button>
          </div>

          {publishResults.length > 0 && (
            <div className="publish-results">
              <h4>发布历史</h4>
              {publishResults.slice(0, 10).map((result, index) => (
                <div key={index} className={`publish-result ${result.success ? 'success' : 'error'}`}>
                  <div className="result-header">
                    <span className="note-title">{result.note_title}</span>
                    <span className="publish-time">
                      {new Date(result.published_at).toLocaleString()}
                    </span>
                  </div>
                  <div className="result-details">
                    <span className="status">
                      {result.success ? '✅ 成功' : '❌ 失败'}
                    </span>
                    <span className="processing-time">
                      {result.processing_time_ms}ms
                    </span>
                    {result.error_message && (
                      <span className="error-message">{result.error_message}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {activeTab === 'preview' && (
        <div className="preview-section">
          <h3>内容预览</h3>
          
          {previewContent ? (
            <div className="preview-content">
              <div className="preview-actions">
                <button
                  onClick={() => validateContent(previewContent)}
                  className="btn-secondary"
                >
                  验证内容
                </button>
              </div>
              
              <div 
                className="preview-html"
                dangerouslySetInnerHTML={{ __html: previewContent }}
              />

              {validationResult && (
                <div className="validation-result">
                  <h4>内容验证结果</h4>
                  
                  <div className="content-stats">
                    <div className="stat-item">
                      <span>字符数: {validationResult.content_stats.character_count}</span>
                    </div>
                    <div className="stat-item">
                      <span>词数: {validationResult.content_stats.word_count}</span>
                    </div>
                    <div className="stat-item">
                      <span>图片: {validationResult.content_stats.image_count}</span>
                    </div>
                    <div className="stat-item">
                      <span>链接: {validationResult.content_stats.link_count}</span>
                    </div>
                    <div className="stat-item">
                      <span>预计阅读: {validationResult.content_stats.estimated_read_time}分钟</span>
                    </div>
                  </div>

                  {validationResult.errors.length > 0 && (
                    <div className="validation-errors">
                      <h5>错误:</h5>
                      <ul>
                        {validationResult.errors.map((error, index) => (
                          <li key={index}>{error}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {validationResult.warnings.length > 0 && (
                    <div className="validation-warnings">
                      <h5>警告:</h5>
                      <ul>
                        {validationResult.warnings.map((warning, index) => (
                          <li key={index}>{warning}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {validationResult.suggestions.length > 0 && (
                    <div className="validation-suggestions">
                      <h5>建议:</h5>
                      <ul>
                        {validationResult.suggestions.map((suggestion, index) => (
                          <li key={index}>{suggestion}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              )}
            </div>
          ) : (
            <div className="no-preview">
              <p>暂无预览内容</p>
              <p>请在发布页面选择笔记进行预览</p>
            </div>
          )}
        </div>
      )}

      {activeTab === 'stats' && (
        <div className="stats-section">
          <h3>统计信息</h3>
          
          {stats ? (
            <div className="stats-grid">
              <div className="stat-item">
                <span className="label">总文章数:</span>
                <span className="value">{stats.total_articles}</span>
              </div>
              <div className="stat-item">
                <span className="label">本月发布:</span>
                <span className="value">{stats.monthly_articles}</span>
              </div>
              <div className="stat-item">
                <span className="label">今日发布:</span>
                <span className="value">{stats.daily_articles}</span>
              </div>
              <div className="stat-item">
                <span className="label">媒体文件:</span>
                <span className="value">{stats.total_media_files}</span>
              </div>
              <div className="stat-item">
                <span className="label">媒体大小:</span>
                <span className="value">{formatFileSize(stats.total_media_size)}</span>
              </div>
              <div className="stat-item">
                <span className="label">平均发布时间:</span>
                <span className="value">{stats.average_publish_time}ms</span>
              </div>
              {stats.last_publish_time && (
                <div className="stat-item">
                  <span className="label">最后发布:</span>
                  <span className="value">
                    {new Date(stats.last_publish_time).toLocaleString()}
                  </span>
                </div>
              )}
              <div className="stat-item">
                <span className="label">令牌状态:</span>
                <span 
                  className="value"
                  style={{ color: getTokenStatusColor(stats.token_status) }}
                >
                  {getTokenStatusText(stats.token_status)}
                </span>
              </div>
            </div>
          ) : (
            <div className="no-stats">
              <p>暂无统计数据</p>
              <p>请先配置并启用微信发布</p>
            </div>
          )}

          <button onClick={loadStats} className="btn-secondary">
            刷新统计
          </button>
        </div>
      )}
    </div>
  );
};

export default WeChatPublisher;