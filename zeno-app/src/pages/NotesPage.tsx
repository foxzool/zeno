import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import CreateNoteDialog from '../components/CreateNoteDialog'
import ContextMenu, { ContextMenuItem } from '../components/ContextMenu'
import { FileText, Edit3, Trash2, Copy, FolderOpen, Info } from 'lucide-react'

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
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [lastRefreshTime, setLastRefreshTime] = useState(0)
  const [contextMenu, setContextMenu] = useState<{
    isOpen: boolean;
    position: { x: number; y: number };
    note: NoteFile | null;
  }>({
    isOpen: false,
    position: { x: 0, y: 0 },
    note: null
  })

  // 防抖刷新函数
  const debouncedRefresh = async () => {
    const now = Date.now()
    // 如果距离上次刷新不到5秒，则跳过
    if (now - lastRefreshTime < 5000) {
      return
    }
    setLastRefreshTime(now)
    await refreshNotes()
  }

  // 加载工作空间和笔记数据
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

  // 初始加载
  useEffect(() => {
    loadWorkspaceAndNotes()
  }, [])

  // 实时刷新：定时器每30秒刷新一次
  useEffect(() => {
    if (!workspacePath) return

    const interval = setInterval(() => {
      debouncedRefresh()
    }, 30000) // 30秒

    return () => clearInterval(interval)
  }, [workspacePath, lastRefreshTime])

  // 实时刷新：页面获得焦点时刷新
  useEffect(() => {
    const handleFocus = () => {
      if (workspacePath && !loading) {
        debouncedRefresh()
      }
    }

    const handleVisibilityChange = () => {
      if (!document.hidden && workspacePath && !loading) {
        debouncedRefresh()
      }
    }

    window.addEventListener('focus', handleFocus)
    document.addEventListener('visibilitychange', handleVisibilityChange)

    return () => {
      window.removeEventListener('focus', handleFocus)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
  }, [workspacePath, loading, lastRefreshTime])

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

    setShowCreateDialog(true)
  }

  const handleCreateConfirm = async (title: string) => {
    try {
      setLoading(true)
      setShowCreateDialog(false)
      
      const result = await invoke('create_note', { title })
      console.log('Note created:', result)
      
      // 创建成功后刷新笔记列表
      await refreshNotes()
      
      // 导航到新创建的笔记
      navigate(`/editor?file=${encodeURIComponent(result as string)}`)
      
      console.log('Note created successfully!')
    } catch (err) {
      console.error('创建笔记失败:', err)
      setError(`创建笔记失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const handleCreateCancel = () => {
    setShowCreateDialog(false)
  }

  const handleNoteClick = (note: NoteFile) => {
    // 导航到编辑器页面，传递文件路径作为参数
    navigate(`/editor?file=${encodeURIComponent(note.path)}`)
  }

  const handleNoteContextMenu = (note: NoteFile, event: React.MouseEvent) => {
    event.preventDefault()
    event.stopPropagation()
    setContextMenu({
      isOpen: true,
      position: { x: event.clientX, y: event.clientY },
      note
    })
  }

  const handleDeleteNote = async (note: NoteFile) => {
    if (!window.confirm(`确定要删除笔记 "${note.name}" 吗？此操作无法撤销。`)) {
      return
    }

    try {
      setLoading(true)
      await invoke('delete_note', { filePath: note.path })
      
      // 刷新笔记列表
      await refreshNotes()
      
      console.log('笔记删除成功')
    } catch (err) {
      console.error('删除笔记失败:', err)
      setError(`删除笔记失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const handleCopyNotePath = (note: NoteFile) => {
    navigator.clipboard.writeText(note.path).then(() => {
      console.log('路径已复制到剪贴板')
    }).catch(err => {
      console.error('复制路径失败:', err)
    })
  }

  const handleShowNoteInfo = (note: NoteFile) => {
    const info = `文件名: ${note.name}\n路径: ${note.path}\n大小: ${Math.round(note.size / 1024)} KB\n修改时间: ${note.modified ? new Date(parseInt(note.modified) * 1000).toLocaleString('zh-CN') : '未知'}`
    alert(info)
  }

  const getContextMenuItems = (note: NoteFile): ContextMenuItem[] => [
    {
      id: 'open',
      label: '打开',
      icon: <FileText size={14} />,
      onClick: () => handleNoteClick(note)
    },
    {
      id: 'edit',
      label: '编辑',
      icon: <Edit3 size={14} />,
      onClick: () => handleNoteClick(note)
    },
    {
      id: 'separator1',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'showInFolder',
      label: '在文件夹中显示',
      icon: <FolderOpen size={14} />,
      onClick: async () => {
        try {
          await invoke('show_in_folder', { filePath: note.path })
        } catch (err) {
          console.error('打开文件夹失败:', err)
        }
      }
    },
    {
      id: 'copyPath',
      label: '复制路径',
      icon: <Copy size={14} />,
      onClick: () => handleCopyNotePath(note)
    },
    {
      id: 'info',
      label: '属性',
      icon: <Info size={14} />,
      onClick: () => handleShowNoteInfo(note)
    },
    {
      id: 'separator2',
      label: '',
      separator: true,
      onClick: () => {}
    },
    {
      id: 'delete',
      label: '删除',
      icon: <Trash2 size={14} />,
      onClick: () => handleDeleteNote(note),
      danger: true
    }
  ]

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
              onContextMenu={(e) => handleNoteContextMenu(note, e)}
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

      {/* 创建笔记对话框 */}
      <CreateNoteDialog
        isOpen={showCreateDialog}
        onClose={handleCreateCancel}
        onConfirm={handleCreateConfirm}
      />

      {/* 右键菜单 */}
      <ContextMenu
        isOpen={contextMenu.isOpen}
        position={contextMenu.position}
        items={contextMenu.note ? getContextMenuItems(contextMenu.note) : []}
        onClose={() => setContextMenu(prev => ({ ...prev, isOpen: false }))}
      />
    </div>
  )
}