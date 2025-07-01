import { HashRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/layout/Layout';
import ThemeProvider from './components/ThemeProvider';
import HomePage from './pages/HomePage';
import NotesPage from './pages/NotesPage';
import EditorPage from './pages/EditorPage';
import SettingsPage from './pages/SettingsPage';
import './index.css';

function App() {
  return (
    <ThemeProvider>
      <HashRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/notes" element={<NotesPage />} />
            <Route path="/editor" element={<EditorPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </Layout>
      </HashRouter>
    </ThemeProvider>
  );
}

export default App;