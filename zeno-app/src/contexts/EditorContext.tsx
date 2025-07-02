import React, { createContext, useContext, useState, ReactNode, RefObject } from 'react';
import { TyporaEditorRef } from '../components/TyporaEditor';

interface EditorContextType {
  currentContent: string;
  setCurrentContent: (content: string) => void;
  currentFile: string | null;
  setCurrentFile: (file: string | null) => void;
  editorRef: RefObject<TyporaEditorRef> | null;
  setEditorRef: (ref: RefObject<TyporaEditorRef>) => void;
}

const EditorContext = createContext<EditorContextType | undefined>(undefined);

export const useEditor = () => {
  const context = useContext(EditorContext);
  if (context === undefined) {
    throw new Error('useEditor must be used within an EditorProvider');
  }
  return context;
};

interface EditorProviderProps {
  children: ReactNode;
}

export const EditorProvider: React.FC<EditorProviderProps> = ({ children }) => {
  const [currentContent, setCurrentContent] = useState('');
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const [editorRef, setEditorRef] = useState<RefObject<TyporaEditorRef> | null>(null);

  const value = {
    currentContent,
    setCurrentContent,
    currentFile,
    setCurrentFile,
    editorRef,
    setEditorRef,
  };

  return (
    <EditorContext.Provider value={value}>
      {children}
    </EditorContext.Provider>
  );
};