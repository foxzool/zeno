import { useEffect, useCallback } from 'react';

type HotkeyCallback = (event: KeyboardEvent) => void;

interface HotkeyConfig {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  callback: HotkeyCallback;
  preventDefault?: boolean;
}

const useHotkeys = (hotkeys: HotkeyConfig[]) => {
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    for (const hotkey of hotkeys) {
      const keyMatches = event.key.toLowerCase() === hotkey.key.toLowerCase();
      const ctrlMatches = !!hotkey.ctrl === (event.ctrlKey || event.metaKey);
      const metaMatches = !!hotkey.meta === event.metaKey;
      const shiftMatches = !!hotkey.shift === event.shiftKey;
      const altMatches = !!hotkey.alt === event.altKey;

      if (keyMatches && ctrlMatches && metaMatches && shiftMatches && altMatches) {
        if (hotkey.preventDefault !== false) {
          event.preventDefault();
        }
        hotkey.callback(event);
        break;
      }
    }
  }, [hotkeys]);

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);
};

export default useHotkeys;

// 预定义的快捷键配置
export const defaultHotkeys = {
  newFile: { key: 'n', ctrl: true },
  openFile: { key: 'o', ctrl: true },
  saveFile: { key: 's', ctrl: true },
  find: { key: 'f', ctrl: true },
  quickOpen: { key: 'p', ctrl: true },
  toggleSidebar: { key: 'b', ctrl: true },
  togglePreview: { key: 'p', ctrl: true, shift: true },
  toggleTheme: { key: 'd', ctrl: true, shift: true },
};