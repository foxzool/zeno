import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'

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
  const navigate = useNavigate()
  const [notes, setNotes] = useState<NoteFile[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [workspacePath, setWorkspacePath] = useState<string | null>(null)
  const [showInputDialog, setShowInputDialog] = useState(false)
  const [inputValue, setInputValue] = useState('')

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

  const createNewNote = async () => {
    console.log('createNewNote called!')
    
    if (!workspacePath) {
      console.log('No workspace path set')
      return
    }

    // 显示输入对话框
    setInputValue('新建笔记')
    setShowInputDialog(true)
  }

  const handleCreateNote = async () => {
    const title = inputValue.trim()
    if (!title) return

    try {
      setLoading(true)
      setShowInputDialog(false)
      const args = { 
        'title': title
      }
      console.log('Creating note with args:', args)
      console.log('Args keys:', Object.keys(args))
      
      const result = await invoke('create_note', args)
      
      console.log('Note created:', result)
      
      // 创建成功后刷新笔记列表
      await refreshNotes()
      console.log('Note created successfully!')
    } catch (err) {
      console.error('创建笔记失败:', err)
      setError(`创建笔记失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const handleCancelInput = () => {
    setShowInputDialog(false)
    setInputValue('')
  }

  const handleNoteClick = (note: NoteFile) => {
    // 导航到编辑器页面，传递文件路径作为参数
    navigate(`/editor?file=${encodeURIComponent(note.path)}`)
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
          <button 
            className="btn-primary"
            onClick={createNewNote}
            disabled={loading}
            title={!workspacePath ? "请先设置工作空间路径" : "创建新笔记"}
          >
            新建笔记
          </button>
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
            <div 
              key={note.path} 
              className="note-card"
              onClick={() => handleNoteClick(note)}
              style={{ cursor: 'pointer' }}
            >
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

      {/* 输入对话框 */}
      {showInputDialog && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.5)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 1000
        }}>
          <div style={{
            backgroundColor: 'white',
            padding: '2rem',
            borderRadius: '0.5rem',
            minWidth: '400px',
            boxShadow: '0 10px 25px rgba(0, 0, 0, 0.1)'
          }}>
            <h3 style={{ marginBottom: '1rem' }}>新建笔记</h3>
            <input
              type="text"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder="请输入笔记标题"
              style={{
                width: '100%',
                padding: '0.5rem',
                border: '1px solid #ddd',
                borderRadius: '0.25rem',
                marginBottom: '1rem',
                fontSize: '1rem'
              }}
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  handleCreateNote()
                } else if (e.key === 'Escape') {
                  handleCancelInput()
                }
              }}
            />
            <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
              <button
                className="btn-secondary"
                onClick={handleCancelInput}
              >
                取消
              </button>
              <button
                className="btn-primary"
                onClick={handleCreateNote}
                disabled={!inputValue.trim()}
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}