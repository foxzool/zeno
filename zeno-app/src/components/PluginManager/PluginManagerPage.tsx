import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './PluginManagerPage.css';

// 类型定义
interface Plugin {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  categories: string[];
  homepage?: string;
  repository?: string;
  install_path: string;
  installed_at: string;
  last_updated: string;
}

interface PluginRuntimeState {
  plugin_id: string;
  status: string;
  memory_usage: number;
  cpu_usage: number;
  uptime_seconds: number;
  error_count: number;
  warning_count: number;
}

interface PluginMarketInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  downloads: number;
  rating: number;
  rating_count: number;
  size: number;
  screenshots: string[];
  tags: string[];
  verified: boolean;
  download_url: string;
}

interface PluginInstallOptions {
  force: boolean;
  skip_dependencies: boolean;
  dev_mode: boolean;
  custom_install_path?: string;
}

const PluginManagerPage: React.FC = () => {
  // 状态管理
  const [activeTab, setActiveTab] = useState('installed');
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [runtimeStates, setRuntimeStates] = useState<PluginRuntimeState[]>([]);
  const [marketPlugins, setMarketPlugins] = useState<PluginMarketInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [installUrl, setInstallUrl] = useState('');
  const [installOptions, setInstallOptions] = useState<PluginInstallOptions>({
    force: false,
    skip_dependencies: false,
    dev_mode: false,
  });

  // 加载已安装的插件
  const loadInstalledPlugins = async () => {
    try {
      setLoading(true);
      const [allPlugins, runtimeStates] = await Promise.all([
        invoke<Plugin[]>('get_all_plugins'),
        invoke<PluginRuntimeState[]>('get_plugin_runtime_states'),
      ]);
      setPlugins(allPlugins);
      setRuntimeStates(runtimeStates);
    } catch (err) {
      setError(`Failed to load plugins: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 搜索插件市场
  const searchMarketplace = async () => {
    try {
      setLoading(true);
      const results = await invoke<PluginMarketInfo[]>('search_plugin_marketplace', {
        query: searchQuery,
        category: selectedCategory || null,
        limit: 20,
      });
      setMarketPlugins(results);
    } catch (err) {
      setError(`Failed to search marketplace: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 安装插件
  const installPlugin = async (source: string) => {
    try {
      setLoading(true);
      await invoke('install_plugin', {
        source,
        options: installOptions,
      });
      await loadInstalledPlugins();
      setInstallUrl('');
    } catch (err) {
      setError(`Failed to install plugin: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 卸载插件
  const uninstallPlugin = async (pluginId: string) => {
    if (!confirm('Are you sure you want to uninstall this plugin?')) {
      return;
    }

    try {
      setLoading(true);
      await invoke('uninstall_plugin', { pluginId });
      await loadInstalledPlugins();
    } catch (err) {
      setError(`Failed to uninstall plugin: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 启用/禁用插件
  const togglePlugin = async (pluginId: string, enabled: boolean) => {
    try {
      setLoading(true);
      if (enabled) {
        await invoke('disable_plugin', { pluginId });
      } else {
        await invoke('enable_plugin', { pluginId });
      }
      await loadInstalledPlugins();
    } catch (err) {
      setError(`Failed to toggle plugin: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 重启插件运行时
  const restartRuntime = async (pluginId: string) => {
    try {
      setLoading(true);
      await invoke('restart_plugin_runtime', { pluginId });
      await loadInstalledPlugins();
    } catch (err) {
      setError(`Failed to restart plugin runtime: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 获取运行时状态
  const getRuntimeState = (pluginId: string): PluginRuntimeState | undefined => {
    return runtimeStates.find(state => state.plugin_id === pluginId);
  };

  // 格式化文件大小
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // 格式化运行时间
  const formatUptime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}h ${minutes}m ${secs}s`;
  };

  // 初始化加载
  useEffect(() => {
    loadInstalledPlugins();
  }, []);

  // 自动搜索市场
  useEffect(() => {
    if (activeTab === 'marketplace') {
      searchMarketplace();
    }
  }, [activeTab, searchQuery, selectedCategory]);

  // 渲染已安装插件
  const renderInstalledPlugins = () => (
    <div className="plugins-grid">
      {plugins.map(plugin => {
        const runtimeState = getRuntimeState(plugin.id);
        return (
          <div key={plugin.id} className="plugin-card">
            <div className="plugin-header">
              <h3>{plugin.name}</h3>
              <span className="plugin-version">v{plugin.version}</span>
            </div>
            
            <div className="plugin-info">
              <p className="plugin-description">{plugin.description}</p>
              <p className="plugin-author">by {plugin.author}</p>
              
              {plugin.categories.length > 0 && (
                <div className="plugin-categories">
                  {plugin.categories.map(category => (
                    <span key={category} className="category-tag">{category}</span>
                  ))}
                </div>
              )}
            </div>

            {runtimeState && (
              <div className="runtime-info">
                <div className="runtime-status">
                  <span className={`status-indicator ${runtimeState.status.toLowerCase()}`}>
                    {runtimeState.status}
                  </span>
                </div>
                <div className="runtime-stats">
                  <span>Memory: {runtimeState.memory_usage}MB</span>
                  <span>CPU: {runtimeState.cpu_usage.toFixed(1)}%</span>
                  <span>Uptime: {formatUptime(runtimeState.uptime_seconds)}</span>
                  {runtimeState.error_count > 0 && (
                    <span className="error-count">Errors: {runtimeState.error_count}</span>
                  )}
                </div>
              </div>
            )}

            <div className="plugin-actions">
              <button 
                className={`toggle-btn ${plugin.enabled ? 'enabled' : 'disabled'}`}
                onClick={() => togglePlugin(plugin.id, plugin.enabled)}
                disabled={loading}
              >
                {plugin.enabled ? 'Disable' : 'Enable'}
              </button>
              
              {plugin.enabled && runtimeState && (
                <button 
                  className="restart-btn"
                  onClick={() => restartRuntime(plugin.id)}
                  disabled={loading}
                >
                  Restart
                </button>
              )}
              
              <button 
                className="uninstall-btn"
                onClick={() => uninstallPlugin(plugin.id)}
                disabled={loading}
              >
                Uninstall
              </button>
              
              {plugin.homepage && (
                <a 
                  href={plugin.homepage} 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="plugin-link"
                >
                  Homepage
                </a>
              )}
            </div>
          </div>
        );
      })}
      
      {plugins.length === 0 && !loading && (
        <div className="empty-state">
          <p>No plugins installed</p>
          <p>Go to Marketplace to discover and install plugins</p>
        </div>
      )}
    </div>
  );

  // 渲染插件市场
  const renderMarketplace = () => (
    <div className="marketplace-content">
      <div className="marketplace-search">
        <input
          type="text"
          placeholder="Search plugins..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />
        
        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value)}
          className="category-select"
        >
          <option value="">All Categories</option>
          <option value="Editor">Editor</option>
          <option value="Export">Export</option>
          <option value="Import">Import</option>
          <option value="Theme">Theme</option>
          <option value="Workflow">Workflow</option>
          <option value="Integration">Integration</option>
          <option value="Utility">Utility</option>
          <option value="Developer">Developer</option>
        </select>
      </div>

      <div className="plugins-grid">
        {marketPlugins.map(plugin => (
          <div key={plugin.id} className="plugin-card marketplace-card">
            <div className="plugin-header">
              <h3>{plugin.name}</h3>
              <div className="plugin-meta">
                <span className="plugin-version">v{plugin.version}</span>
                {plugin.verified && <span className="verified-badge">✓</span>}
              </div>
            </div>
            
            <div className="plugin-info">
              <p className="plugin-description">{plugin.description}</p>
              <p className="plugin-author">by {plugin.author}</p>
              
              <div className="plugin-stats">
                <span>⭐ {plugin.rating.toFixed(1)} ({plugin.rating_count})</span>
                <span>📥 {plugin.downloads.toLocaleString()}</span>
                <span>📦 {formatFileSize(plugin.size)}</span>
              </div>
              
              {plugin.tags.length > 0 && (
                <div className="plugin-tags">
                  {plugin.tags.map(tag => (
                    <span key={tag} className="tag">{tag}</span>
                  ))}
                </div>
              )}
            </div>

            <div className="plugin-actions">
              <button 
                className="install-btn"
                onClick={() => installPlugin(plugin.download_url)}
                disabled={loading}
              >
                Install
              </button>
            </div>
          </div>
        ))}
        
        {marketPlugins.length === 0 && !loading && (
          <div className="empty-state">
            <p>No plugins found</p>
            <p>Try adjusting your search criteria</p>
          </div>
        )}
      </div>
    </div>
  );

  // 渲染安装界面
  const renderInstall = () => (
    <div className="install-content">
      <div className="install-form">
        <h3>Install Plugin</h3>
        
        <div className="form-group">
          <label>Plugin Source:</label>
          <input
            type="text"
            placeholder="Enter URL, local path, or plugin ID"
            value={installUrl}
            onChange={(e) => setInstallUrl(e.target.value)}
            className="install-input"
          />
        </div>
        
        <div className="install-options">
          <h4>Install Options</h4>
          
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={installOptions.force}
              onChange={(e) => setInstallOptions({
                ...installOptions,
                force: e.target.checked
              })}
            />
            Force install (overwrite existing)
          </label>
          
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={installOptions.skip_dependencies}
              onChange={(e) => setInstallOptions({
                ...installOptions,
                skip_dependencies: e.target.checked
              })}
            />
            Skip dependency check
          </label>
          
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={installOptions.dev_mode}
              onChange={(e) => setInstallOptions({
                ...installOptions,
                dev_mode: e.target.checked
              })}
            />
            Development mode
          </label>
        </div>
        
        <button
          className="install-button"
          onClick={() => installPlugin(installUrl)}
          disabled={loading || !installUrl.trim()}
        >
          {loading ? 'Installing...' : 'Install Plugin'}
        </button>
      </div>
      
      <div className="install-help">
        <h4>Installation Sources</h4>
        <ul>
          <li><strong>URL:</strong> https://github.com/user/plugin.git</li>
          <li><strong>Local:</strong> /path/to/plugin/directory</li>
          <li><strong>ZIP:</strong> /path/to/plugin.zip</li>
          <li><strong>Marketplace:</strong> plugin-id</li>
        </ul>
      </div>
    </div>
  );

  return (
    <div className="plugin-manager-page">
      <div className="page-header">
        <h1>Plugin Manager</h1>
        <p>Manage and discover plugins to extend Zeno's functionality</p>
      </div>

      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="tab-navigation">
        <button
          className={activeTab === 'installed' ? 'active' : ''}
          onClick={() => setActiveTab('installed')}
        >
          Installed ({plugins.length})
        </button>
        <button
          className={activeTab === 'marketplace' ? 'active' : ''}
          onClick={() => setActiveTab('marketplace')}
        >
          Marketplace
        </button>
        <button
          className={activeTab === 'install' ? 'active' : ''}
          onClick={() => setActiveTab('install')}
        >
          Install
        </button>
      </div>

      <div className="tab-content">
        {loading && <div className="loading-indicator">Loading...</div>}
        
        {activeTab === 'installed' && renderInstalledPlugins()}
        {activeTab === 'marketplace' && renderMarketplace()}
        {activeTab === 'install' && renderInstall()}
      </div>
    </div>
  );
};

export default PluginManagerPage;