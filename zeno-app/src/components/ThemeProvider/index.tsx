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
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #f1f5f9;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --text-tertiary: #94a3b8;
            --border-primary: #e2e8f0;
            --border-secondary: #cbd5e1;
            --accent-primary: #3b82f6;
            --accent-secondary: #dbeafe;
            --success: #10b981;
            --warning: #f59e0b;
            --error: #ef4444;
          }
          
          .theme-root.dark {
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-tertiary: #334155;
            --text-primary: #f1f5f9;
            --text-secondary: #cbd5e1;
            --text-tertiary: #94a3b8;
            --border-primary: #334155;
            --border-secondary: #475569;
            --accent-primary: #60a5fa;
            --accent-secondary: #1e40af;
            --success: #34d399;
            --warning: #fbbf24;
            --error: #f87171;
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