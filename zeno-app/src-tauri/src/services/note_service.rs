use crate::models::{Note, AppError};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

pub struct NoteService {
    workspace_path: PathBuf,
}

impl NoteService {
    pub fn new(workspace_path: PathBuf) -> Self {
        Self { workspace_path }
    }
    
    pub async fn list_notes(&self) -> Result<Vec<Note>, AppError> {
        let notes_dir = self.workspace_path.join("notes");
        if !notes_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut notes = Vec::new();
        let mut entries = fs::read_dir(&notes_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && is_markdown_file(&path) {
                if let Ok(note) = self.load_note(&path).await {
                    notes.push(note);
                }
            }
        }
        
        Ok(notes)
    }
    
    pub async fn load_note(&self, path: &Path) -> Result<Note, AppError> {
        let content = fs::read_to_string(path).await?;
        let title = extract_title_from_content(&content)
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string()
            });
        
        Ok(Note::new(path.to_path_buf(), title, content))
    }
    
    pub async fn save_note(&self, note: &Note) -> Result<(), AppError> {
        let notes_dir = self.workspace_path.join("notes");
        fs::create_dir_all(&notes_dir).await?;
        
        fs::write(&note.path, &note.content).await?;
        Ok(())
    }
    
    pub async fn create_note(&self, title: &str, content: &str) -> Result<Note, AppError> {
        let filename = slugify(title) + ".md";
        let path = self.workspace_path.join("notes").join(filename);
        
        let note = Note::new(path, title.to_string(), content.to_string());
        self.save_note(&note).await?;
        
        Ok(note)
    }
    
    pub async fn delete_note(&self, id: Uuid) -> Result<(), AppError> {
        let notes = self.list_notes().await?;
        if let Some(note) = notes.iter().find(|n| n.id == id) {
            fs::remove_file(&note.path).await?;
        }
        Ok(())
    }
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md" || ext == "markdown")
        .unwrap_or(false)
}

fn extract_title_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("# ") {
            return Some(line[2..].trim().to_string());
        }
    }
    None
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}