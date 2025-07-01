import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface AppInfo {
  name: string
  version: string
  description: string
}

export default function HomePage() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const loadAppInfo = async () => {
      try {
        const info = await invoke<AppInfo>('get_app_info')
        setAppInfo(info)
      } catch (error) {
        console.error('获取应用信息失败:', error)
      } finally {
        setIsLoading(false)
      }
    }

    loadAppInfo()
  }, [])

  if (isLoading) {
    return (
      <div style={{ 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center', 
        height: '100%' 
      }}>
        <div style={{ color: '#6c757d' }}>加载中...</div>
      </div>
    )
  }

  return (
    <div style={{ padding: '2rem' }}>
      <div style={{ maxWidth: '48rem', margin: '0 auto' }}>
        <h1 style={{ 
          fontSize: '1.875rem', 
          fontWeight: 'bold', 
          marginBottom: '1rem' 
        }}>
          欢迎使用 Zeno
        </h1>
        
        {appInfo && (
          <div style={{ 
            backgroundColor: '#f8f9fa', 
            border: '1px solid #e9ecef', 
            borderRadius: '0.5rem', 
            padding: '1.5rem', 
            marginBottom: '1.5rem' 
          }}>
            <h2 style={{ 
              fontSize: '1.25rem', 
              fontWeight: '600', 
              marginBottom: '0.5rem' 
            }}>
              应用信息
            </h2>
            <div style={{ fontSize: '0.875rem' }}>
              <div style={{ marginBottom: '0.5rem' }}>
                <span style={{ color: '#6c757d' }}>名称：</span>
                <span>{appInfo.name}</span>
              </div>
              <div style={{ marginBottom: '0.5rem' }}>
                <span style={{ color: '#6c757d' }}>版本：</span>
                <span>{appInfo.version}</span>
              </div>
              <div>
                <span style={{ color: '#6c757d' }}>描述：</span>
                <span>{appInfo.description}</span>
              </div>
            </div>
          </div>
        )}

        <div>
          <h2 style={{ fontSize: '1.5rem', marginBottom: '1rem' }}>功能特色</h2>
          <ul style={{ marginBottom: '1.5rem' }}>
            <li>📝 Markdown 编辑和解析</li>
            <li>🔗 双向链接支持</li>
            <li>🏷️ 标签和分类管理</li>
            <li>🔍 全文搜索</li>
            <li>📊 知识图谱可视化</li>
            <li>🚀 多平台发布</li>
          </ul>

          <h2 style={{ fontSize: '1.5rem', marginBottom: '1rem' }}>开始使用</h2>
          <p>点击左侧导航栏的"笔记"开始管理你的知识库。</p>
        </div>
      </div>
    </div>
  )
}