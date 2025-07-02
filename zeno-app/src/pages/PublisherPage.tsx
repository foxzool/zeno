import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import WeChatPublisher from '../components/WeChatPublisher';
import { ImportExportPage, PluginManagerPage } from '../components';
import '../styles/PublisherPage.css';

interface ZolaConfig {
  base_url: string;
  title: string;
  description: string;
  author: string;
  default_language: string;
  theme: string;
  compile_sass: boolean;
  generate_rss: boolean;
  build_search_index: boolean;
  taxonomies: Array<{
    name: string;
    feed?: boolean;
    paginate_by?: number;
  }>;
  markdown: {
    highlight_code: boolean;
    highlight_theme: string;
    render_emoji: boolean;
    external_links_target_blank: boolean;
    external_links_no_follow: boolean;
    external_links_no_referrer: boolean;
    smart_punctuation: boolean;
  };
  extra: { [key: string]: any };
}

interface PublishResult {
  published_count: number;
  failed_count: number;
  build_time_ms: number;
  output_path: string;
  site_size: number;
  errors: string[];
}

interface SiteStats {
  total_pages: number;
  total_words: number;
  total_tags: number;
  total_categories: number;
  last_build: string | null;
  build_time: number | null;
  site_size: number;
}

const PublisherPage: React.FC = () => {
  const [config, setConfig] = useState<ZolaConfig | null>(null);
  const [sitePath, setSitePath] = useState<string>('');
  const [workspacePath, setWorkspacePath] = useState<string>('');
  const [publishResult, setPublishResult] = useState<PublishResult | null>(null);
  const [siteStats, setSiteStats] = useState<SiteStats | null>(null);
  const [isPublishing, setIsPublishing] = useState(false);
  const [zolaInstalled, setZolaInstalled] = useState<boolean | null>(null);
  const [activeTab, setActiveTab] = useState<'zola' | 'wechat' | 'import-export' | 'plugins' | 'stats'>('zola');

  useEffect(() => {
    loadDefaultConfig();
    checkZolaInstallation();
  }, []);

  useEffect(() => {
    if (sitePath) {
      loadSiteStats();
    }
  }, [sitePath]);

  const loadDefaultConfig = async () => {
    try {
      const defaultConfig = await invoke<ZolaConfig>('create_default_zola_config');
      setConfig(defaultConfig);
    } catch (error) {
      console.error('Failed to load default config:', error);
    }
  };

  const checkZolaInstallation = async () => {
    try {
      const installed = await invoke<boolean>('check_zola_installation');
      setZolaInstalled(installed);
    } catch (error) {
      console.error('Failed to check Zola installation:', error);
      setZolaInstalled(false);
    }
  };

  const loadSiteStats = async () => {
    if (!sitePath) return;
    
    try {
      const stats = await invoke<SiteStats>('get_site_stats', { sitePath });
      setSiteStats(stats);
    } catch (error) {
      console.error('Failed to load site stats:', error);
    }
  };

  const selectSitePath = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择发布站点目录',
      });
      
      if (selected && typeof selected === 'string') {
        setSitePath(selected);
      }
    } catch (error) {
      console.error('Failed to select site path:', error);
    }
  };

  const selectWorkspacePath = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择工作空间目录',
      });
      
      if (selected && typeof selected === 'string') {
        setWorkspacePath(selected);
      }
    } catch (error) {
      console.error('Failed to select workspace path:', error);
    }
  };

  const initializeSite = async () => {
    if (!config || !sitePath) {
      alert('请先配置站点路径和 Zola 配置');
      return;
    }

    try {
      setIsPublishing(true);
      await invoke('initialize_zola_site', {
        sitePath,
        config,
      });
      
      alert('站点初始化成功！');
      loadSiteStats();
    } catch (error) {
      console.error('Failed to initialize site:', error);
      alert(`站点初始化失败: ${error}`);
    } finally {
      setIsPublishing(false);
    }
  };

  const publishNotes = async () => {
    if (!config || !sitePath || !workspacePath) {
      alert('请先配置所有必要的路径和配置');
      return;
    }

    try {
      setIsPublishing(true);
      const result = await invoke<PublishResult>('publish_notes_to_site', {
        sitePath,
        config,
        workspacePath,
      });
      
      setPublishResult(result);
      loadSiteStats();
      alert(`发布完成! 成功发布 ${result.published_count} 篇笔记`);
    } catch (error) {
      console.error('Failed to publish notes:', error);
      alert(`发布失败: ${error}`);
    } finally {
      setIsPublishing(false);
    }
  };

  const updateConfigField = (field: keyof ZolaConfig, value: any) => {
    if (!config) return;
    
    setConfig({
      ...config,
      [field]: value,
    });
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
      <div className="publisher-page">
        <div className="loading">加载配置中...</div>
      </div>
    );
  }

  return (
    <div className="publisher-page">
      <div className="publisher-header">
        <h1>静态网站发布</h1>
        <p>使用 Zola 将笔记发布为静态网站</p>
        
        {zolaInstalled === false && (
          <div className="zola-warning">
            <p>⚠️ 未检测到 Zola 安装。请先安装 Zola: <code>brew install zola</code></p>
          </div>
        )}
      </div>

      <div className="publisher-tabs">
        <button
          className={`tab ${activeTab === 'zola' ? 'active' : ''}`}
          onClick={() => setActiveTab('zola')}
        >
          静态网站 (Zola)
        </button>
        <button
          className={`tab ${activeTab === 'wechat' ? 'active' : ''}`}
          onClick={() => setActiveTab('wechat')}
        >
          微信公众号
        </button>
        <button
          className={`tab ${activeTab === 'import-export' ? 'active' : ''}`}
          onClick={() => setActiveTab('import-export')}
        >
          导入导出
        </button>
        <button
          className={`tab ${activeTab === 'plugins' ? 'active' : ''}`}
          onClick={() => setActiveTab('plugins')}
        >
          插件管理
        </button>
        <button
          className={`tab ${activeTab === 'stats' ? 'active' : ''}`}
          onClick={() => setActiveTab('stats')}
        >
          统计
        </button>
      </div>

      {activeTab === 'zola' && (
        <div className="zola-section">
          <h2>Zola 静态网站生成</h2>
          
          <div className="form-group">
            <label>站点路径:</label>
            <div className="path-input">
              <input
                type="text"
                value={sitePath}
                onChange={(e) => setSitePath(e.target.value)}
                placeholder="选择站点目录"
              />
              <button onClick={selectSitePath}>浏览</button>
            </div>
          </div>

          <div className="form-group">
            <label>工作空间路径:</label>
            <div className="path-input">
              <input
                type="text"
                value={workspacePath}
                onChange={(e) => setWorkspacePath(e.target.value)}
                placeholder="选择笔记工作空间"
              />
              <button onClick={selectWorkspacePath}>浏览</button>
            </div>
          </div>

          <div className="config-grid">
            <div className="form-group">
              <label>站点 URL:</label>
              <input
                type="text"
                value={config.base_url}
                onChange={(e) => updateConfigField('base_url', e.target.value)}
              />
            </div>

            <div className="form-group">
              <label>站点标题:</label>
              <input
                type="text"
                value={config.title}
                onChange={(e) => updateConfigField('title', e.target.value)}
              />
            </div>

            <div className="form-group">
              <label>站点描述:</label>
              <textarea
                value={config.description}
                onChange={(e) => updateConfigField('description', e.target.value)}
              />
            </div>

            <div className="form-group">
              <label>作者:</label>
              <input
                type="text"
                value={config.author}
                onChange={(e) => updateConfigField('author', e.target.value)}
              />
            </div>

            <div className="form-group">
              <label>默认语言:</label>
              <select
                value={config.default_language}
                onChange={(e) => updateConfigField('default_language', e.target.value)}
              >
                <option value="zh">中文</option>
                <option value="en">English</option>
              </select>
            </div>

            <div className="form-group">
              <label>主题:</label>
              <input
                type="text"
                value={config.theme}
                onChange={(e) => updateConfigField('theme', e.target.value)}
              />
            </div>
          </div>

          <div className="checkbox-group">
            <label>
              <input
                type="checkbox"
                checked={config.compile_sass}
                onChange={(e) => updateConfigField('compile_sass', e.target.checked)}
              />
              编译 Sass
            </label>

            <label>
              <input
                type="checkbox"
                checked={config.generate_rss}
                onChange={(e) => updateConfigField('generate_rss', e.target.checked)}
              />
              生成 RSS
            </label>

            <label>
              <input
                type="checkbox"
                checked={config.build_search_index}
                onChange={(e) => updateConfigField('build_search_index', e.target.checked)}
              />
              构建搜索索引
            </label>
          </div>
          
          <div className="publish-actions">
            <button
              onClick={initializeSite}
              disabled={isPublishing || !sitePath}
              className="btn-secondary"
            >
              {isPublishing ? '初始化中...' : '初始化站点'}
            </button>

            <button
              onClick={publishNotes}
              disabled={isPublishing || !sitePath || !workspacePath}
              className="btn-primary"
            >
              {isPublishing ? '发布中...' : '发布笔记'}
            </button>
          </div>

          {publishResult && (
            <div className="publish-result">
              <h3>发布结果</h3>
              <div className="result-grid">
                <div className="result-item">
                  <span className="label">发布成功:</span>
                  <span className="value">{publishResult.published_count} 篇</span>
                </div>
                <div className="result-item">
                  <span className="label">发布失败:</span>
                  <span className="value">{publishResult.failed_count} 篇</span>
                </div>
                <div className="result-item">
                  <span className="label">构建时间:</span>
                  <span className="value">{publishResult.build_time_ms} ms</span>
                </div>
                <div className="result-item">
                  <span className="label">站点大小:</span>
                  <span className="value">{formatFileSize(publishResult.site_size)}</span>
                </div>
              </div>

              {publishResult.errors.length > 0 && (
                <div className="errors">
                  <h4>错误信息:</h4>
                  <ul>
                    {publishResult.errors.map((error, index) => (
                      <li key={index}>{error}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {activeTab === 'wechat' && (
        <WeChatPublisher />
      )}

      {activeTab === 'import-export' && (
        <ImportExportPage />
      )}

      {activeTab === 'plugins' && (
        <PluginManagerPage />
      )}

      {activeTab === 'stats' && (
        <div className="stats-section">
          <h2>站点统计</h2>
          
          {siteStats ? (
            <div className="stats-grid">
              <div className="stat-item">
                <span className="label">总页面数:</span>
                <span className="value">{siteStats.total_pages}</span>
              </div>
              <div className="stat-item">
                <span className="label">总字数:</span>
                <span className="value">{siteStats.total_words.toLocaleString()}</span>
              </div>
              <div className="stat-item">
                <span className="label">标签数:</span>
                <span className="value">{siteStats.total_tags}</span>
              </div>
              <div className="stat-item">
                <span className="label">分类数:</span>
                <span className="value">{siteStats.total_categories}</span>
              </div>
              <div className="stat-item">
                <span className="label">站点大小:</span>
                <span className="value">{formatFileSize(siteStats.site_size)}</span>
              </div>
              <div className="stat-item">
                <span className="label">最后构建:</span>
                <span className="value">
                  {siteStats.last_build ? new Date(siteStats.last_build).toLocaleString() : '未构建'}
                </span>
              </div>
              {siteStats.build_time && (
                <div className="stat-item">
                  <span className="label">构建时间:</span>
                  <span className="value">{siteStats.build_time} ms</span>
                </div>
              )}
            </div>
          ) : (
            <div className="no-stats">
              <p>暂无站点统计数据</p>
              <p>请先选择站点路径或初始化站点</p>
            </div>
          )}

          <button onClick={loadSiteStats} className="btn-secondary">
            刷新统计
          </button>
        </div>
      )}
    </div>
  );
};

export default PublisherPage;