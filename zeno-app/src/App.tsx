import { HashRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/layout/Layout';
import ThemeProvider from './components/ThemeProvider';
import { EditorProvider } from './contexts/EditorContext';
import HomePage from './pages/HomePage';
import NotesPage from './pages/NotesPage';
import EditorPage from './pages/EditorPage';
import GraphPage from './pages/GraphPage';
import PublisherPage from './pages/PublisherPage';
import SettingsPage from './pages/SettingsPage';
import './index.css';

function App() {
  return (
    <ThemeProvider defaultTheme="auto">
      <EditorProvider>
        <HashRouter>
          <Layout>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/notes" element={<NotesPage />} />
              <Route path="/editor" element={<EditorPage />} />
              <Route path="/graph" element={<GraphPage />} />
              <Route path="/publisher" element={<PublisherPage />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Routes>
          </Layout>
        </HashRouter>
      </EditorProvider>
    </ThemeProvider>
  );
}

export default App;