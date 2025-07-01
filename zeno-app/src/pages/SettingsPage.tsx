import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AppConfig {
  workspace_path: string | null;
  theme: string;
  language: string;
  auto_save: boolean;
  sync_enabled: boolean;
}

export default function SettingsPage() {
  const [config, setConfig] = useState<AppConfig>({
    workspace_path: null,
    theme: 'light',
    language: 'zh-CN',
    auto_save: true,
    sync_enabled: false,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [workspacePath, setWorkspacePath] = useState('');
  const [message, setMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const loadedConfig = await invoke<AppConfig>('get_config');
      setConfig(loadedConfig);
      setWorkspacePath(loadedConfig.workspace_path || '');
    } catch (error) {
      console.error('加载配置失败:', error);
      showMessage('error', '加载配置失败');
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    setSaving(true);
    try {
      const updatedConfig = {
        ...config,
        workspace_path: workspacePath || null,
      };
      await invoke('save_config', { config: updatedConfig });
      setConfig(updatedConfig);
      showMessage('success', '设置已保存');
    } catch (error) {
      console.error('保存配置失败:', error);
      showMessage('error', '保存配置失败');
    } finally {
      setSaving(false);
    }
  };

  const selectWorkspaceDirectory = async () => {
    try {
      const selectedPath = await invoke<string | null>('select_workspace_directory');
      if (selectedPath) {
        setWorkspacePath(selectedPath);
      }
    } catch (error) {
      console.error('选择目录失败:', error);
      showMessage('error', '选择目录失败');
    }
  };

  const createWorkspaceDirectory = async () => {
    if (!workspacePath) {
      showMessage('error', '请先设置工作空间路径');
      return;
    }

    try {
      await invoke('create_workspace_directory', { path: workspacePath });
      showMessage('success', '工作空间创建成功');
    } catch (error) {
      console.error('创建工作空间失败:', error);
      showMessage('error', '创建工作空间失败');
    }
  };

  const validateWorkspacePath = async () => {
    if (!workspacePath) return;

    try {
      const isValid = await invoke<boolean>('validate_workspace_path', { path: workspacePath });
      if (!isValid) {
        showMessage('error', '工作空间路径无效或无权限');
      }
    } catch (error) {
      console.error('验证路径失败:', error);
    }
  };

  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 3000);
  };

  if (loading) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center' }}>
        <div style={{ fontSize: '1.25rem' }}>加载中...</div>
      </div>
    );
  }

  return (
    <div style={{ padding: '2rem' }}>
      <div style={{ maxWidth: '64rem', margin: '0 auto' }}>
        <h1 style={{ 
          fontSize: '1.875rem', 
          fontWeight: 'bold', 
          marginBottom: '1.5rem' 
        }}>
          设置
        </h1>

        {message && (
          <div style={{
            padding: '0.75rem 1rem',
            marginBottom: '1rem',
            borderRadius: '0.375rem',
            backgroundColor: message.type === 'success' ? '#d1fae5' : '#fecaca',
            color: message.type === 'success' ? '#065f46' : '#991b1b',
            border: `1px solid ${message.type === 'success' ? '#a7f3d0' : '#fca5a5'}`,
          }}>
            {message.text}
          </div>
        )}

        {/* 工作空间设置 */}
        <div style={{
          backgroundColor: '#ffffff',
          border: '1px solid #e5e7eb',
          borderRadius: '0.5rem',
          padding: '1.5rem',
          marginBottom: '1.5rem',
        }}>
          <h2 style={{
            fontSize: '1.25rem',
            fontWeight: '600',
            marginBottom: '1rem',
            color: '#111827',
          }}>
            工作空间设置
          </h2>
          
          <div style={{ marginBottom: '1rem' }}>
            <label style={{
              display: 'block',
              fontSize: '0.875rem',
              fontWeight: '500',
              marginBottom: '0.5rem',
              color: '#374151',
            }}>
              笔记目录路径
            </label>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <input
                type="text"
                value={workspacePath}
                onChange={(e) => setWorkspacePath(e.target.value)}
                onBlur={validateWorkspacePath}
                placeholder="选择或输入笔记存储目录"
                style={{
                  flex: 1,
                  padding: '0.5rem 0.75rem',
                  border: '1px solid #d1d5db',
                  borderRadius: '0.375rem',
                  fontSize: '0.875rem',
                }}
              />
              <button
                onClick={selectWorkspaceDirectory}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#3b82f6',
                  color: 'white',
                  border: 'none',
                  borderRadius: '0.375rem',
                  fontSize: '0.875rem',
                  cursor: 'pointer',
                }}
              >
                选择目录
              </button>
              <button
                onClick={createWorkspaceDirectory}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#10b981',
                  color: 'white',
                  border: 'none',
                  borderRadius: '0.375rem',
                  fontSize: '0.875rem',
                  cursor: 'pointer',
                }}
              >
                创建工作空间
              </button>
            </div>
            <p style={{
              fontSize: '0.75rem',
              color: '#6b7280',
              marginTop: '0.25rem',
            }}>
              选择一个目录来存储你的笔记。如果目录不存在，可以点击"创建工作空间"自动创建。
            </p>
          </div>
        </div>

        {/* 界面设置 */}
        <div style={{
          backgroundColor: '#ffffff',
          border: '1px solid #e5e7eb',
          borderRadius: '0.5rem',
          padding: '1.5rem',
          marginBottom: '1.5rem',
        }}>
          <h2 style={{
            fontSize: '1.25rem',
            fontWeight: '600',
            marginBottom: '1rem',
            color: '#111827',
          }}>
            界面设置
          </h2>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
            <div>
              <label style={{
                display: 'block',
                fontSize: '0.875rem',
                fontWeight: '500',
                marginBottom: '0.5rem',
                color: '#374151',
              }}>
                主题
              </label>
              <select
                value={config.theme}
                onChange={(e) => setConfig({ ...config, theme: e.target.value })}
                style={{
                  width: '100%',
                  padding: '0.5rem 0.75rem',
                  border: '1px solid #d1d5db',
                  borderRadius: '0.375rem',
                  fontSize: '0.875rem',
                }}
              >
                <option value="light">浅色</option>
                <option value="dark">深色</option>
                <option value="auto">跟随系统</option>
              </select>
            </div>

            <div>
              <label style={{
                display: 'block',
                fontSize: '0.875rem',
                fontWeight: '500',
                marginBottom: '0.5rem',
                color: '#374151',
              }}>
                语言
              </label>
              <select
                value={config.language}
                onChange={(e) => setConfig({ ...config, language: e.target.value })}
                style={{
                  width: '100%',
                  padding: '0.5rem 0.75rem',
                  border: '1px solid #d1d5db',
                  borderRadius: '0.375rem',
                  fontSize: '0.875rem',
                }}
              >
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option>
              </select>
            </div>
          </div>
        </div>

        {/* 功能设置 */}
        <div style={{
          backgroundColor: '#ffffff',
          border: '1px solid #e5e7eb',
          borderRadius: '0.5rem',
          padding: '1.5rem',
          marginBottom: '1.5rem',
        }}>
          <h2 style={{
            fontSize: '1.25rem',
            fontWeight: '600',
            marginBottom: '1rem',
            color: '#111827',
          }}>
            功能设置
          </h2>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <input
                type="checkbox"
                checked={config.auto_save}
                onChange={(e) => setConfig({ ...config, auto_save: e.target.checked })}
                style={{ width: '1rem', height: '1rem' }}
              />
              <span style={{ fontSize: '0.875rem', color: '#374151' }}>
                自动保存
              </span>
            </label>

            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <input
                type="checkbox"
                checked={config.sync_enabled}
                onChange={(e) => setConfig({ ...config, sync_enabled: e.target.checked })}
                style={{ width: '1rem', height: '1rem' }}
              />
              <span style={{ fontSize: '0.875rem', color: '#374151' }}>
                启用云同步 (即将推出)
              </span>
            </label>
          </div>
        </div>

        {/* 保存按钮 */}
        <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
          <button
            onClick={saveConfig}
            disabled={saving}
            style={{
              padding: '0.75rem 1.5rem',
              backgroundColor: saving ? '#9ca3af' : '#3b82f6',
              color: 'white',
              border: 'none',
              borderRadius: '0.375rem',
              fontSize: '0.875rem',
              fontWeight: '500',
              cursor: saving ? 'not-allowed' : 'pointer',
            }}
          >
            {saving ? '保存中...' : '保存设置'}
          </button>
        </div>
      </div>
    </div>
  );
}