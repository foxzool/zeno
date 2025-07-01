import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface NoteFile {
  path: string
  name: string
  size: number
  modified: string | null
}

interface AppConfig {
  workspace_path: string | null
  theme: string
  language: string
  auto_save: boolean
  sync_enabled: boolean
}

const formatDate = (timestamp: string): string => {
  try {
    const date = new Date(parseInt(timestamp) * 1000)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))
    
    if (diffDays === 0) {
      return '今天'
    } else if (diffDays === 1) {
      return '昨天'
    } else if (diffDays < 7) {
      return `${diffDays} 天前`
    } else {
      return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      })
    }
  } catch (error) {
    return '未知时间'
  }
}

export default function NotesPage() {
  const [notes, setNotes] = useState<NoteFile[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [workspacePath, setWorkspacePath] = useState<string | null>(null)

  useEffect(() => {
    const loadWorkspaceAndNotes = async () => {
      try {
        // 首先读取配置获取工作空间路径
        const config = await invoke<AppConfig>('get_config')
        
        if (!config.workspace_path) {
          setError('未设置工作空间路径。请前往设置页面配置笔记目录。')
          setLoading(false)
          return
        }

        setWorkspacePath(config.workspace_path)

        // 验证工作空间路径是否有效
        const isValid = await invoke<boolean>('validate_workspace_path', { 
          path: config.workspace_path 
        })

        if (!isValid) {
          setError(`工作空间路径无效: ${config.workspace_path}。请检查目录是否存在且有权限访问。`)
          setLoading(false)
          return
        }

        // 加载笔记列表
        const notesList = await invoke<NoteFile[]>('list_notes', { 
          dirPath: config.workspace_path 
        })
        setNotes(notesList)
        setError(null)
      } catch (err) {
        console.error('加载笔记失败:', err)
        setError('无法加载笔记目录，请检查配置和路径是否正确。')
      } finally {
        setLoading(false)
      }
    }

    loadWorkspaceAndNotes()
  }, [])

  const createWorkspace = async () => {
    if (!workspacePath) return

    try {
      setLoading(true)
      await invoke('create_workspace_directory', { path: workspacePath })
      
      // 重新加载笔记
      const notesList = await invoke<NoteFile[]>('list_notes', { 
        dirPath: workspacePath 
      })
      setNotes(notesList)
      setError(null)
    } catch (err) {
      console.error('创建工作空间失败:', err)
      setError('创建工作空间失败，请检查权限。')
    } finally {
      setLoading(false)
    }
  }

  const refreshNotes = async () => {
    if (!workspacePath) return

    try {
      setLoading(true)
      const notesList = await invoke<NoteFile[]>('list_notes', { 
        dirPath: workspacePath 
      })
      setNotes(notesList)
      setError(null)
    } catch (err) {
      console.error('刷新笔记失败:', err)
      setError('刷新失败，请检查工作空间路径。')
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading">加载笔记中...</div>
      </div>
    )
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>笔记管理</h1>
        <div>
          {workspacePath && (
            <button 
              className="btn-secondary" 
              onClick={refreshNotes}
              style={{ marginRight: '0.5rem' }}
              disabled={loading}
            >
              {loading ? '刷新中...' : '🔄 刷新'}
            </button>
          )}
          <button className="btn-primary">新建笔记</button>
        </div>
      </div>
      
      {error ? (
        <div className="error-message">
          <span className="icon">⚠️</span>
          <div>
            <h3>加载失败</h3>
            <p>{error}</p>
            {error.includes('未设置工作空间路径') && (
              <div style={{ marginTop: '1rem' }}>
                <button 
                  className="btn-primary"
                  onClick={() => window.location.href = '#/settings'}
                  style={{ marginRight: '0.5rem' }}
                >
                  前往设置
                </button>
              </div>
            )}
            {error.includes('工作空间路径无效') && workspacePath && (
              <div style={{ marginTop: '1rem' }}>
                <button 
                  className="btn-primary"
                  onClick={createWorkspace}
                  style={{ marginRight: '0.5rem' }}
                >
                  创建工作空间
                </button>
                <button 
                  className="btn-secondary"
                  onClick={() => window.location.href = '#/settings'}
                >
                  修改设置
                </button>
              </div>
            )}
          </div>
        </div>
      ) : notes.length === 0 ? (
        <div className="empty-state">
          <span className="icon">📝</span>
          <h3>还没有笔记</h3>
          <p>点击"新建笔记"开始记录您的想法</p>
        </div>
      ) : (
        <div className="notes-grid">
          {notes.map((note) => (
            <div key={note.path} className="note-card">
              <h3 className="note-title">{note.name}</h3>
              <div className="note-meta">
                <span>大小: {Math.round(note.size / 1024)} KB</span>
                {note.modified && (
                  <span 
                    title={new Date(parseInt(note.modified) * 1000).toLocaleString('zh-CN')}
                    style={{ cursor: 'help' }}
                  >
                    修改: {formatDate(note.modified)}
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}