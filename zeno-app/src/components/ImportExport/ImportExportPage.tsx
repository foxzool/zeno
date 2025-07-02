import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './ImportExportPage.css';

interface ImportConfig {
  importer_type: ImporterType;
  source_path: string;
  target_workspace: string;
  options: ImportOptions;
}

interface ExportConfig {
  exporter_type: ExporterType;
  source_workspace: string;
  target_path: string;
  options: ExportOptions;
}

interface ImportOptions {
  preserve_structure: boolean;
  merge_mode: 'Overwrite' | 'Skip' | 'Merge' | 'Rename';
  include_attachments: boolean;
  convert_links: boolean;
  convert_tags: boolean;
  dry_run: boolean;
  skip_duplicates: boolean;
  backup_existing: boolean;
  custom_mappings: { [key: string]: string };
}

interface ExportOptions {
  include_attachments: boolean;
  include_metadata: boolean;
  preserve_structure: boolean;
  convert_links: boolean;
  include_tags: boolean;
  filter_options: FilterOptions;
  format_options: FormatOptions;
  output_options: OutputOptions;
}

interface FilterOptions {
  date_range?: { start: string; end: string };
  tag_filter: string[];
  path_filter: string[];
  content_filter?: string;
  exclude_drafts: boolean;
  include_archived: boolean;
  minimum_word_count?: number;
}

interface FormatOptions {
  page_size?: 'A4' | 'Letter' | 'Legal' | 'A3' | 'A5';
  font_family?: string;
  font_size?: number;
  line_height?: number;
  margin?: { top: number; right: number; bottom: number; left: number };
  header_footer: boolean;
  table_of_contents: boolean;
  syntax_highlighting: boolean;
  math_rendering: boolean;
}

interface OutputOptions {
  compression: boolean;
  encryption: boolean;
  password?: string;
  split_by_size?: number;
  naming_pattern: string;
  custom_metadata: { [key: string]: string };
}

type ImporterType = 'Obsidian' | 'Notion' | 'Markdown' | 'Roam' | 'LogSeq' | 'Generic';
type ExporterType = 'Markdown' | 'Html' | 'Pdf' | 'Epub' | 'Latex' | 'Json' | 'Zip' | 'Obsidian' | 'Notion';

interface ImportPreview {
  total_files: number;
  notes: number;
  attachments: number;
  media_files: number;
  estimated_size: number;
  warnings: string[];
  conflicts: FileConflict[];
  structure: DirectoryNode[];
}

interface ExportPreview {
  total_notes: number;
  total_attachments: number;
  estimated_size: number;
  filtered_notes: number;
  warnings: string[];
  structure: ExportDirectoryNode[];
}

interface FileConflict {
  source_path: string;
  target_path: string;
  conflict_type: 'NameCollision' | 'ContentMismatch' | 'SizeDiscrepancy' | 'TimestampConflict';
  suggested_resolution: string;
}

interface DirectoryNode {
  name: string;
  path: string;
  file_count: number;
  children: DirectoryNode[];
}

interface ExportDirectoryNode {
  name: string;
  path: string;
  note_count: number;
  attachment_count: number;
  children: ExportDirectoryNode[];
}

interface ImportResult {
  success: boolean;
  imported_count: number;
  skipped_count: number;
  failed_count: number;
  warnings: string[];
  errors: string[];
  processing_time_ms: number;
  imported_files: ImportedFile[];
  duplicate_files: string[];
}

interface ExportResult {
  success: boolean;
  exported_count: number;
  skipped_count: number;
  failed_count: number;
  warnings: string[];
  errors: string[];
  processing_time_ms: number;
  output_files: ExportedFile[];
  total_size: number;
  compression_ratio?: number;
}

interface ImportedFile {
  source_path: string;
  target_path: string;
  file_type: 'Note' | 'Attachment' | 'Media' | 'Config';
  size: number;
  modified_time: string;
  status: 'Success' | 'Warning' | 'Failed' | 'Skipped';
  transformations: Transformation[];
}

interface ExportedFile {
  source_path: string;
  output_path: string;
  file_type: 'Note' | 'Index' | 'Attachment' | 'Stylesheet' | 'Metadata' | 'Archive';
  original_size: number;
  exported_size: number;
  status: 'Success' | 'Warning' | 'Failed' | 'Skipped';
  transformations: ExportTransformation[];
}

interface Transformation {
  transformation_type: 'LinkConversion' | 'TagConversion' | 'FrontmatterConversion' | 'PathRewrite' | 'ContentReformat';
  description: string;
  from_value: string;
  to_value: string;
}

interface ExportTransformation {
  transformation_type: 'FormatConversion' | 'LinkRewriting' | 'AssetEmbedding' | 'MetadataExtraction' | 'ContentFiltering' | 'StructureFlattening';
  description: string;
  from_format: string;
  to_format: string;
}

const ImportExportPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'import' | 'export'>('import');
  const [importConfig, setImportConfig] = useState<ImportConfig | null>(null);
  const [exportConfig, setExportConfig] = useState<ExportConfig | null>(null);
  const [importPreview, setImportPreview] = useState<ImportPreview | null>(null);
  const [exportPreview, setExportPreview] = useState<ExportPreview | null>(null);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);

  useEffect(() => {
    loadDefaultConfigs();
  }, []);

  const loadDefaultConfigs = async () => {
    try {
      const defaultImportConfig = await invoke<ImportConfig>('create_default_import_config', {
        importerType: 'Markdown',
        sourcePath: '',
        targetWorkspace: ''
      });
      setImportConfig(defaultImportConfig);

      const defaultExportConfig = await invoke<ExportConfig>('create_default_export_config', {
        exporterType: 'Markdown',
        sourceWorkspace: '',
        targetPath: ''
      });
      setExportConfig(defaultExportConfig);
    } catch (error) {
      console.error('Failed to load default configs:', error);
    }
  };

  const selectImportSource = async () => {
    try {
      console.log('Opening file dialog for import source...');
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择导入源目录',
      });

      console.log('File dialog result:', selected);

      if (selected && typeof selected === 'string' && importConfig) {
        console.log('Setting import source path:', selected);
        setImportConfig({
          ...importConfig,
          source_path: selected
        });
      } else {
        console.log('No file selected or invalid selection');
      }
    } catch (error) {
      console.error('Failed to select import source:', error);
      alert(`选择导入源失败: ${error}`);
    }
  };

  const selectImportTarget = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择目标工作空间',
      });

      if (selected && typeof selected === 'string' && importConfig) {
        setImportConfig({
          ...importConfig,
          target_workspace: selected
        });
      }
    } catch (error) {
      console.error('Failed to select import target:', error);
    }
  };

  const selectExportSource = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择源工作空间',
      });

      if (selected && typeof selected === 'string' && exportConfig) {
        setExportConfig({
          ...exportConfig,
          source_workspace: selected
        });
      }
    } catch (error) {
      console.error('Failed to select export source:', error);
    }
  };

  const selectExportTarget = async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: '选择导出文件路径',
      });

      if (selected && typeof selected === 'string' && exportConfig) {
        setExportConfig({
          ...exportConfig,
          target_path: selected
        });
      }
    } catch (error) {
      console.error('Failed to select export target:', error);
    }
  };

  const previewImport = async () => {
    if (!importConfig || !importConfig.source_path || !importConfig.target_workspace) {
      alert('请先设置导入源和目标路径');
      return;
    }

    try {
      setIsProcessing(true);
      const preview = await invoke<ImportPreview>('preview_import', { config: importConfig });
      setImportPreview(preview);
    } catch (error) {
      console.error('Failed to preview import:', error);
      alert(`预览导入失败: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const executeImport = async () => {
    if (!importConfig || !importConfig.source_path || !importConfig.target_workspace) {
      alert('请先设置导入源和目标路径');
      return;
    }

    try {
      setIsProcessing(true);
      const result = await invoke<ImportResult>('execute_import', { config: importConfig });
      setImportResult(result);
      
      if (result.success) {
        alert(`导入完成! 成功导入 ${result.imported_count} 个文件`);
      } else {
        alert(`导入失败，请查看错误信息`);
      }
    } catch (error) {
      console.error('Failed to execute import:', error);
      alert(`导入失败: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const previewExport = async () => {
    if (!exportConfig || !exportConfig.source_workspace || !exportConfig.target_path) {
      alert('请先设置源工作空间和导出路径');
      return;
    }

    try {
      setIsProcessing(true);
      const preview = await invoke<ExportPreview>('preview_export', { config: exportConfig });
      setExportPreview(preview);
    } catch (error) {
      console.error('Failed to preview export:', error);
      alert(`预览导出失败: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const executeExport = async () => {
    if (!exportConfig || !exportConfig.source_workspace || !exportConfig.target_path) {
      alert('请先设置源工作空间和导出路径');
      return;
    }

    try {
      setIsProcessing(true);
      const result = await invoke<ExportResult>('execute_export', { config: exportConfig });
      setExportResult(result);
      
      if (result.success) {
        alert(`导出完成! 成功导出 ${result.exported_count} 个文件`);
      } else {
        alert(`导出失败，请查看错误信息`);
      }
    } catch (error) {
      console.error('Failed to execute export:', error);
      alert(`导出失败: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  if (!importConfig || !exportConfig) {
    return (
      <div className="import-export-page">
        <div className="loading">加载配置中...</div>
      </div>
    );
  }

  return (
    <div className="import-export-page">
      <div className="page-header">
        <h1>数据导入导出</h1>
        <p>导入外部数据或导出笔记到不同格式</p>
      </div>

      <div className="tabs">
        <button
          className={`tab ${activeTab === 'import' ? 'active' : ''}`}
          onClick={() => setActiveTab('import')}
        >
          数据导入
        </button>
        <button
          className={`tab ${activeTab === 'export' ? 'active' : ''}`}
          onClick={() => setActiveTab('export')}
        >
          数据导出
        </button>
      </div>

      {activeTab === 'import' && (
        <div className="import-section">
          <div className="config-panel">
            <h2>导入配置</h2>
            
            <div className="form-group">
              <label>导入类型:</label>
              <select
                value={importConfig.importer_type}
                onChange={(e) => setImportConfig({
                  ...importConfig,
                  importer_type: e.target.value as ImporterType
                })}
              >
                <option value="Markdown">Markdown 文件</option>
                <option value="Obsidian">Obsidian 库</option>
                <option value="Notion">Notion 导出</option>
                <option value="Roam">Roam Research</option>
                <option value="LogSeq">LogSeq</option>
                <option value="Generic">通用文本</option>
              </select>
            </div>

            <div className="form-group">
              <label>导入源:</label>
              <div className="path-input">
                <input
                  type="text"
                  value={importConfig.source_path}
                  onChange={(e) => setImportConfig({
                    ...importConfig,
                    source_path: e.target.value
                  })}
                  placeholder="选择导入源目录"
                />
                <button onClick={selectImportSource}>浏览</button>
              </div>
            </div>

            <div className="form-group">
              <label>目标工作空间:</label>
              <div className="path-input">
                <input
                  type="text"
                  value={importConfig.target_workspace}
                  onChange={(e) => setImportConfig({
                    ...importConfig,
                    target_workspace: e.target.value
                  })}
                  placeholder="选择目标工作空间"
                />
                <button onClick={selectImportTarget}>浏览</button>
              </div>
            </div>

            <div className="options-section">
              <div className="options-header">
                <h3>导入选项</h3>
                <button
                  className="toggle-advanced"
                  onClick={() => setShowAdvanced(!showAdvanced)}
                >
                  {showAdvanced ? '隐藏高级选项' : '显示高级选项'}
                </button>
              </div>

              <div className="basic-options">
                <div className="checkbox-group">
                  <label>
                    <input
                      type="checkbox"
                      checked={importConfig.options.preserve_structure}
                      onChange={(e) => setImportConfig({
                        ...importConfig,
                        options: {
                          ...importConfig.options,
                          preserve_structure: e.target.checked
                        }
                      })}
                    />
                    保持目录结构
                  </label>

                  <label>
                    <input
                      type="checkbox"
                      checked={importConfig.options.include_attachments}
                      onChange={(e) => setImportConfig({
                        ...importConfig,
                        options: {
                          ...importConfig.options,
                          include_attachments: e.target.checked
                        }
                      })}
                    />
                    包含附件
                  </label>

                  <label>
                    <input
                      type="checkbox"
                      checked={importConfig.options.convert_links}
                      onChange={(e) => setImportConfig({
                        ...importConfig,
                        options: {
                          ...importConfig.options,
                          convert_links: e.target.checked
                        }
                      })}
                    />
                    转换链接格式
                  </label>

                  <label>
                    <input
                      type="checkbox"
                      checked={importConfig.options.convert_tags}
                      onChange={(e) => setImportConfig({
                        ...importConfig,
                        options: {
                          ...importConfig.options,
                          convert_tags: e.target.checked
                        }
                      })}
                    />
                    转换标签格式
                  </label>
                </div>
              </div>

              {showAdvanced && (
                <div className="advanced-options">
                  <div className="form-group">
                    <label>冲突处理:</label>
                    <select
                      value={importConfig.options.merge_mode}
                      onChange={(e) => setImportConfig({
                        ...importConfig,
                        options: {
                          ...importConfig.options,
                          merge_mode: e.target.value as any
                        }
                      })}
                    >
                      <option value="Skip">跳过</option>
                      <option value="Overwrite">覆盖</option>
                      <option value="Merge">合并</option>
                      <option value="Rename">重命名</option>
                    </select>
                  </div>

                  <div className="checkbox-group">
                    <label>
                      <input
                        type="checkbox"
                        checked={importConfig.options.dry_run}
                        onChange={(e) => setImportConfig({
                          ...importConfig,
                          options: {
                            ...importConfig.options,
                            dry_run: e.target.checked
                          }
                        })}
                      />
                      预演模式（不实际导入）
                    </label>

                    <label>
                      <input
                        type="checkbox"
                        checked={importConfig.options.backup_existing}
                        onChange={(e) => setImportConfig({
                          ...importConfig,
                          options: {
                            ...importConfig.options,
                            backup_existing: e.target.checked
                          }
                        })}
                      />
                      备份现有文件
                    </label>

                    <label>
                      <input
                        type="checkbox"
                        checked={importConfig.options.skip_duplicates}
                        onChange={(e) => setImportConfig({
                          ...importConfig,
                          options: {
                            ...importConfig.options,
                            skip_duplicates: e.target.checked
                          }
                        })}
                      />
                      跳过重复文件
                    </label>
                  </div>
                </div>
              )}
            </div>

            <div className="action-buttons">
              <button
                onClick={previewImport}
                disabled={isProcessing || !importConfig.source_path || !importConfig.target_workspace}
                className="btn-secondary"
              >
                {isProcessing ? '预览中...' : '预览导入'}
              </button>

              <button
                onClick={executeImport}
                disabled={isProcessing || !importConfig.source_path || !importConfig.target_workspace}
                className="btn-primary"
              >
                {isProcessing ? '导入中...' : '开始导入'}
              </button>
            </div>
          </div>

          {importPreview && (
            <div className="preview-panel">
              <h3>导入预览</h3>
              
              <div className="preview-stats">
                <div className="stat-item">
                  <span className="label">总文件数:</span>
                  <span className="value">{importPreview.total_files}</span>
                </div>
                <div className="stat-item">
                  <span className="label">笔记:</span>
                  <span className="value">{importPreview.notes}</span>
                </div>
                <div className="stat-item">
                  <span className="label">附件:</span>
                  <span className="value">{importPreview.attachments}</span>
                </div>
                <div className="stat-item">
                  <span className="label">媒体文件:</span>
                  <span className="value">{importPreview.media_files}</span>
                </div>
                <div className="stat-item">
                  <span className="label">估计大小:</span>
                  <span className="value">{formatFileSize(importPreview.estimated_size)}</span>
                </div>
              </div>

              {importPreview.warnings.length > 0 && (
                <div className="warnings">
                  <h4>警告:</h4>
                  <ul>
                    {importPreview.warnings.map((warning, index) => (
                      <li key={index}>{warning}</li>
                    ))}
                  </ul>
                </div>
              )}

              {importPreview.conflicts.length > 0 && (
                <div className="conflicts">
                  <h4>文件冲突:</h4>
                  <ul>
                    {importPreview.conflicts.map((conflict, index) => (
                      <li key={index}>
                        <strong>{conflict.source_path}</strong> → {conflict.target_path}
                        <br />
                        建议: {conflict.suggested_resolution}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {importResult && (
            <div className="result-panel">
              <h3>导入结果</h3>
              
              <div className="result-stats">
                <div className="stat-item">
                  <span className="label">成功:</span>
                  <span className="value">{importResult.imported_count}</span>
                </div>
                <div className="stat-item">
                  <span className="label">跳过:</span>
                  <span className="value">{importResult.skipped_count}</span>
                </div>
                <div className="stat-item">
                  <span className="label">失败:</span>
                  <span className="value">{importResult.failed_count}</span>
                </div>
                <div className="stat-item">
                  <span className="label">处理时间:</span>
                  <span className="value">{importResult.processing_time_ms} ms</span>
                </div>
              </div>

              {importResult.errors.length > 0 && (
                <div className="errors">
                  <h4>错误:</h4>
                  <ul>
                    {importResult.errors.map((error, index) => (
                      <li key={index}>{error}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {activeTab === 'export' && (
        <div className="export-section">
          <div className="config-panel">
            <h2>导出配置</h2>
            
            <div className="form-group">
              <label>导出格式:</label>
              <select
                value={exportConfig.exporter_type}
                onChange={(e) => setExportConfig({
                  ...exportConfig,
                  exporter_type: e.target.value as ExporterType
                })}
              >
                <option value="Markdown">Markdown 格式</option>
                <option value="Html">HTML 格式</option>
                <option value="Pdf">PDF 文档</option>
                <option value="Epub">EPUB 电子书</option>
                <option value="Latex">LaTeX 格式</option>
                <option value="Json">JSON 数据</option>
                <option value="Zip">ZIP 压缩包</option>
              </select>
            </div>

            <div className="form-group">
              <label>源工作空间:</label>
              <div className="path-input">
                <input
                  type="text"
                  value={exportConfig.source_workspace}
                  onChange={(e) => setExportConfig({
                    ...exportConfig,
                    source_workspace: e.target.value
                  })}
                  placeholder="选择源工作空间"
                />
                <button onClick={selectExportSource}>浏览</button>
              </div>
            </div>

            <div className="form-group">
              <label>导出路径:</label>
              <div className="path-input">
                <input
                  type="text"
                  value={exportConfig.target_path}
                  onChange={(e) => setExportConfig({
                    ...exportConfig,
                    target_path: e.target.value
                  })}
                  placeholder="选择导出文件路径"
                />
                <button onClick={selectExportTarget}>浏览</button>
              </div>
            </div>

            <div className="options-section">
              <div className="options-header">
                <h3>导出选项</h3>
                <button
                  className="toggle-advanced"
                  onClick={() => setShowAdvanced(!showAdvanced)}
                >
                  {showAdvanced ? '隐藏高级选项' : '显示高级选项'}
                </button>
              </div>

              <div className="basic-options">
                <div className="checkbox-group">
                  <label>
                    <input
                      type="checkbox"
                      checked={exportConfig.options.include_attachments}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          include_attachments: e.target.checked
                        }
                      })}
                    />
                    包含附件
                  </label>

                  <label>
                    <input
                      type="checkbox"
                      checked={exportConfig.options.include_metadata}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          include_metadata: e.target.checked
                        }
                      })}
                    />
                    包含元数据
                  </label>

                  <label>
                    <input
                      type="checkbox"
                      checked={exportConfig.options.preserve_structure}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          preserve_structure: e.target.checked
                        }
                      })}
                    />
                    保持目录结构
                  </label>

                  <label>
                    <input
                      type="checkbox"
                      checked={exportConfig.options.convert_links}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          convert_links: e.target.checked
                        }
                      })}
                    />
                    转换链接
                  </label>
                </div>
              </div>

              {showAdvanced && (
                <div className="advanced-options">
                  <div className="form-group">
                    <label>标签过滤 (逗号分隔):</label>
                    <input
                      type="text"
                      value={exportConfig.options.filter_options.tag_filter.join(', ')}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          filter_options: {
                            ...exportConfig.options.filter_options,
                            tag_filter: e.target.value.split(',').map(s => s.trim()).filter(s => s)
                          }
                        }
                      })}
                      placeholder="输入要包含的标签"
                    />
                  </div>

                  <div className="form-group">
                    <label>路径过滤 (逗号分隔):</label>
                    <input
                      type="text"
                      value={exportConfig.options.filter_options.path_filter.join(', ')}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          filter_options: {
                            ...exportConfig.options.filter_options,
                            path_filter: e.target.value.split(',').map(s => s.trim()).filter(s => s)
                          }
                        }
                      })}
                      placeholder="输入要包含的路径"
                    />
                  </div>

                  <div className="form-group">
                    <label>最小字数:</label>
                    <input
                      type="number"
                      value={exportConfig.options.filter_options.minimum_word_count || ''}
                      onChange={(e) => setExportConfig({
                        ...exportConfig,
                        options: {
                          ...exportConfig.options,
                          filter_options: {
                            ...exportConfig.options.filter_options,
                            minimum_word_count: e.target.value ? parseInt(e.target.value) : undefined
                          }
                        }
                      })}
                      placeholder="最小字数过滤"
                    />
                  </div>

                  <div className="checkbox-group">
                    <label>
                      <input
                        type="checkbox"
                        checked={exportConfig.options.filter_options.exclude_drafts}
                        onChange={(e) => setExportConfig({
                          ...exportConfig,
                          options: {
                            ...exportConfig.options,
                            filter_options: {
                              ...exportConfig.options.filter_options,
                              exclude_drafts: e.target.checked
                            }
                          }
                        })}
                      />
                      排除草稿
                    </label>

                    <label>
                      <input
                        type="checkbox"
                        checked={exportConfig.options.filter_options.include_archived}
                        onChange={(e) => setExportConfig({
                          ...exportConfig,
                          options: {
                            ...exportConfig.options,
                            filter_options: {
                              ...exportConfig.options.filter_options,
                              include_archived: e.target.checked
                            }
                          }
                        })}
                      />
                      包含归档
                    </label>
                  </div>
                </div>
              )}
            </div>

            <div className="action-buttons">
              <button
                onClick={previewExport}
                disabled={isProcessing || !exportConfig.source_workspace || !exportConfig.target_path}
                className="btn-secondary"
              >
                {isProcessing ? '预览中...' : '预览导出'}
              </button>

              <button
                onClick={executeExport}
                disabled={isProcessing || !exportConfig.source_workspace || !exportConfig.target_path}
                className="btn-primary"
              >
                {isProcessing ? '导出中...' : '开始导出'}
              </button>
            </div>
          </div>

          {exportPreview && (
            <div className="preview-panel">
              <h3>导出预览</h3>
              
              <div className="preview-stats">
                <div className="stat-item">
                  <span className="label">总笔记数:</span>
                  <span className="value">{exportPreview.total_notes}</span>
                </div>
                <div className="stat-item">
                  <span className="label">过滤后:</span>
                  <span className="value">{exportPreview.filtered_notes}</span>
                </div>
                <div className="stat-item">
                  <span className="label">附件数:</span>
                  <span className="value">{exportPreview.total_attachments}</span>
                </div>
                <div className="stat-item">
                  <span className="label">估计大小:</span>
                  <span className="value">{formatFileSize(exportPreview.estimated_size)}</span>
                </div>
              </div>

              {exportPreview.warnings.length > 0 && (
                <div className="warnings">
                  <h4>警告:</h4>
                  <ul>
                    {exportPreview.warnings.map((warning, index) => (
                      <li key={index}>{warning}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {exportResult && (
            <div className="result-panel">
              <h3>导出结果</h3>
              
              <div className="result-stats">
                <div className="stat-item">
                  <span className="label">成功:</span>
                  <span className="value">{exportResult.exported_count}</span>
                </div>
                <div className="stat-item">
                  <span className="label">跳过:</span>
                  <span className="value">{exportResult.skipped_count}</span>
                </div>
                <div className="stat-item">
                  <span className="label">失败:</span>
                  <span className="value">{exportResult.failed_count}</span>
                </div>
                <div className="stat-item">
                  <span className="label">总大小:</span>
                  <span className="value">{formatFileSize(exportResult.total_size)}</span>
                </div>
                <div className="stat-item">
                  <span className="label">处理时间:</span>
                  <span className="value">{exportResult.processing_time_ms} ms</span>
                </div>
                {exportResult.compression_ratio && (
                  <div className="stat-item">
                    <span className="label">压缩率:</span>
                    <span className="value">{(exportResult.compression_ratio * 100).toFixed(1)}%</span>
                  </div>
                )}
              </div>

              {exportResult.errors.length > 0 && (
                <div className="errors">
                  <h4>错误:</h4>
                  <ul>
                    {exportResult.errors.map((error, index) => (
                      <li key={index}>{error}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default ImportExportPage;