import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface NoteFile {
  path: string
  name: string
  size: number
  modified: string | null
}

export default function NotesPage() {
  const [notes, setNotes] = useState<NoteFile[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const loadNotes = async () => {
      try {
        // 先尝试读取默认笔记目录
        const homeDir = '/Users/' + (process.env.USER || 'user')
        const notesDir = homeDir + '/Documents/Notes'
        const notesList = await invoke<NoteFile[]>('list_notes', { dirPath: notesDir })
        setNotes(notesList)
      } catch (err) {
        console.error('加载笔记失败:', err)
        setError('无法加载笔记目录，请检查路径是否存在')
      } finally {
        setLoading(false)
      }
    }

    loadNotes()
  }, [])

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
        <button className="btn-primary">新建笔记</button>
      </div>
      
      {error ? (
        <div className="error-message">
          <span className="icon">⚠️</span>
          <div>
            <h3>加载失败</h3>
            <p>{error}</p>
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
                  <span>修改: {new Date(note.modified).toLocaleDateString()}</span>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}