import React, { useState } from 'react';

interface CreateNoteDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (title: string) => void;
  defaultTitle?: string;
  title?: string;
  placeholder?: string;
  confirmText?: string;
}

const CreateNoteDialog: React.FC<CreateNoteDialogProps> = ({
  isOpen,
  onClose,
  onConfirm,
  defaultTitle = '新建笔记',
  title: dialogTitle = '新建笔记',
  placeholder = '请输入笔记标题',
  confirmText = '创建'
}) => {
  const [title, setTitle] = useState(defaultTitle);

  React.useEffect(() => {
    if (isOpen) {
      setTitle(defaultTitle);
    }
  }, [isOpen, defaultTitle]);

  const handleConfirm = () => {
    const trimmedTitle = title.trim();
    if (trimmedTitle) {
      onConfirm(trimmedTitle);
      setTitle('');
    }
  };

  const handleCancel = () => {
    setTitle('');
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleConfirm();
    } else if (e.key === 'Escape') {
      handleCancel();
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
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
        <h3 style={{ marginBottom: '1rem' }}>{dialogTitle}</h3>
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder={placeholder}
          style={{
            width: '100%',
            padding: '0.5rem',
            border: '1px solid #ddd',
            borderRadius: '0.25rem',
            marginBottom: '1rem',
            fontSize: '1rem'
          }}
          autoFocus
          onKeyDown={handleKeyDown}
        />
        <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
          <button
            onClick={handleCancel}
            className="btn-secondary"
          >
            取消
          </button>
          <button
            onClick={handleConfirm}
            disabled={!title.trim()}
            className="btn-primary"
          >
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
};

export default CreateNoteDialog;