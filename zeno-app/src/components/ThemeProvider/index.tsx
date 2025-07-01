import React, { createContext, useContext, useEffect, useState } from 'react';

export type Theme = 'light' | 'dark' | 'auto';

interface ThemeContextType {
  theme: Theme;
  actualTheme: 'light' | 'dark';
  setTheme: (theme: Theme) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: Theme;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({
  children,
  defaultTheme = 'auto'
}) => {
  const [theme, setTheme] = useState<Theme>(() => {
    const stored = localStorage.getItem('zeno-theme');
    return (stored as Theme) || defaultTheme;
  });
  
  // 避免 TypeScript 警告
  console.log('defaultTheme:', defaultTheme);
  
  const [actualTheme, setActualTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    const updateActualTheme = () => {
      if (theme === 'auto') {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        setActualTheme(mediaQuery.matches ? 'dark' : 'light');
        
        const handleChange = (e: MediaQueryListEvent) => {
          setActualTheme(e.matches ? 'dark' : 'light');
        };
        
        mediaQuery.addEventListener('change', handleChange);
        return () => mediaQuery.removeEventListener('change', handleChange);
      } else {
        setActualTheme(theme);
      }
    };

    updateActualTheme();
  }, [theme]);

  useEffect(() => {
    localStorage.setItem('zeno-theme', theme);
    
    // Update document class for CSS
    document.documentElement.classList.remove('light', 'dark');
    document.documentElement.classList.add(actualTheme);
    
    // Update meta theme-color for mobile browsers
    const metaThemeColor = document.querySelector('meta[name="theme-color"]');
    if (metaThemeColor) {
      metaThemeColor.setAttribute('content', actualTheme === 'dark' ? '#1f2937' : '#ffffff');
    }
  }, [actualTheme, theme]);

  const handleSetTheme = (newTheme: Theme) => {
    setTheme(newTheme);
  };

  return (
    <ThemeContext.Provider value={{ theme, actualTheme, setTheme: handleSetTheme }}>
      <div className={`theme-root ${actualTheme}`} data-theme={actualTheme}>
        {children}
        
        <style>{`
          :root {
            color-scheme: ${actualTheme};
          }
          
          .theme-root.light {
            /* 清爽的浅色背景 */
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #f1f5f9;
            
            /* 清晰的文字层次 */
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --text-tertiary: #94a3b8;
            
            /* 精致的边框和分割线 */
            --border-primary: #e2e8f0;
            --border-secondary: #cbd5e1;
            
            /* 品牌色调 */
            --accent-primary: #3b82f6;
            --accent-secondary: #2563eb;
            
            /* 状态颜色 */
            --success: #10b981;
            --warning: #f59e0b;
            --error: #ef4444;
            
            /* 特殊用途颜色 */
            --error-bg: #fef2f2;
            --error-border: #fecaca;
            --error-text: #7f1d1d;
            
            /* 阴影效果 */
            --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
            --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
            --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.15);
          }
          
          .theme-root.dark {
            /* 温暖的深色背景 - 减少冷感 */
            --bg-primary: #0a0f1b;
            --bg-secondary: #141925;
            --bg-tertiary: #1e2531;
            
            /* 高对比度文字 - 确保可读性 */
            --text-primary: #e8eaed;
            --text-secondary: #9aa0a6;
            --text-tertiary: #73787e;
            
            /* 微妙的边框和分割线 */
            --border-primary: rgba(255, 255, 255, 0.08);
            --border-secondary: rgba(255, 255, 255, 0.16);
            
            /* 品牌色调 - 适配深色模式 */
            --accent-primary: #5b8dee;
            --accent-secondary: #4285f4;
            
            /* 状态颜色 - 深色模式优化 */
            --success: #34d399;
            --warning: #fbbf24;
            --error: #f87171;
            
            /* 特殊用途颜色 */
            --error-bg: rgba(239, 68, 68, 0.1);
            --error-border: rgba(239, 68, 68, 0.2);
            --error-text: #fca5a5;
            
            /* 阴影效果 */
            --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
            --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);
            --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.5);
          }
          
          * {
            transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
          }
          
          body {
            background-color: var(--bg-primary);
            color: var(--text-primary);
          }
        `}</style>
      </div>
    </ThemeContext.Provider>
  );
};

export default ThemeProvider;