import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { GraphNode, GraphEdge } from '../components/KnowledgeGraph';

interface BacklinkInfo {
  source_note_id: string;
  source_note_title: string;
  source_note_path: string;
  context: string;
  line_number: number;
  link_type: string;
  occurrence_count: number;
}

interface SimilarNote {
  note_id: string;
  title: string;
  path: string;
  similarity_score: number;
  common_links: string[];
  similarity_reason: string;
}

interface Note {
  id: string;
  title: string;
  path: string;
  content: string;
  tags: string[];
  modified_time?: string;
}

export interface GraphStats {
  total_notes: number;
  total_links: number;
  total_broken_links: number;
  orphaned_notes: number;
}

export const useKnowledgeGraph = () => {
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [stats, setStats] = useState<GraphStats | null>(null);

  // 从笔记数据构建图谱节点
  const buildGraphFromNotes = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // 首先获取配置来获取工作空间路径
      const config = await invoke<any>('get_config');
      if (!config.workspace_path) {
        throw new Error('未设置工作空间路径，请先在设置中配置工作空间');
      }

      // 获取所有笔记文件
      const noteFiles = await invoke<any[]>('list_notes', { dirPath: config.workspace_path });
      const graphNodes: GraphNode[] = [];
      const edgeMap = new Map<string, GraphEdge>();
      const notes: Note[] = [];

      // 读取每个笔记文件的内容并解析
      for (const noteFile of noteFiles) {
        try {
          const content = await invoke<string>('read_file_content', { path: noteFile.path });
          
          // 使用文件名作为标题，去掉 .md 扩展名
          const title = noteFile.name.replace(/\.md$/, '');
          
          // 使用新的标签提取方法
          const tags = await invoke<string[]>('extract_tags_from_content', { content });
          
          const note: Note = {
            id: noteFile.path, // 使用文件路径作为唯一 ID
            title,
            path: noteFile.path,
            content,
            tags,
            modified_time: noteFile.modified,
          };
          
          notes.push(note);
        } catch (err) {
          console.warn(`Failed to read note file ${noteFile.path}:`, err);
          continue;
        }
      }

      // 为每个笔记创建节点
      for (const note of notes) {
        // 计算节点大小（基于字数）
        const wordCount = note.content.length;
        const size = Math.max(20, Math.min(60, Math.sqrt(wordCount) * 2));

        graphNodes.push({
          id: note.id,
          label: note.title,
          type: 'note',
          size,
          color: '#3B82F6',
          metadata: {
            path: note.path,
            wordCount,
            lastModified: note.modified_time || '',
            tags: note.tags || [],
          },
        });

        // 获取笔记的反向链接
        try {
          const backlinks = await invoke<BacklinkInfo[]>('get_backlinks', { noteId: note.id });
          
          for (const backlink of backlinks) {
            const edgeId = `${backlink.source_note_id}-${note.id}`;
            if (!edgeMap.has(edgeId)) {
              edgeMap.set(edgeId, {
                id: edgeId,
                source: backlink.source_note_id,
                target: note.id,
                type: 'link',
                weight: Math.min(5, backlink.occurrence_count),
                metadata: {
                  linkText: backlink.context,
                  context: backlink.context,
                },
              });
            }
          }
        } catch (err) {
          console.warn(`Failed to get backlinks for note ${note.id}:`, err);
        }

        // 获取相似笔记并创建相似性边
        try {
          const similarNotes = await invoke<SimilarNote[]>('find_similar_notes', { 
            noteId: note.id, 
            limit: 5 
          });
          
          for (const similar of similarNotes) {
            if (similar.similarity_score > 0.3) { // 只显示相似度较高的
              const edgeId = `${note.id}-${similar.note_id}-similarity`;
              if (!edgeMap.has(edgeId)) {
                edgeMap.set(edgeId, {
                  id: edgeId,
                  source: note.id,
                  target: similar.note_id,
                  type: 'similarity',
                  weight: Math.round(similar.similarity_score * 5),
                  metadata: {
                    context: similar.similarity_reason,
                  },
                });
              }
            }
          }
        } catch (err) {
          console.warn(`Failed to get similar notes for ${note.id}:`, err);
        }
      }

      // 解析和构建标签层次结构
      const allTagStrings = notes.flatMap(note => note.tags || []);
      const uniqueTagStrings = Array.from(new Set(allTagStrings));
      
      // 解析层次化标签
      await invoke('parse_and_add_tags', { tagStrings: uniqueTagStrings });
      
      // 获取所有标签（包括层次化结构）
      const allHierarchicalTags = await invoke<any[]>('get_all_tags');
      
      // 为每个标签创建节点
      const tagNodes = new Map<string, GraphNode>();
      for (const hierarchicalTag of allHierarchicalTags) {
        const displayName = hierarchicalTag.name.split('/').pop() || hierarchicalTag.name;
        const tagId = `tag-${hierarchicalTag.name}`;
        
        tagNodes.set(hierarchicalTag.name, {
          id: tagId,
          label: `#${displayName}`,
          type: 'tag',
          size: Math.max(20, Math.min(50, hierarchicalTag.note_count * 8 + 20)),
          color: hierarchicalTag.level === 0 ? '#059669' : '#10B981', // 根标签使用深色
          metadata: {
            path: '',
            wordCount: 0,
            lastModified: '',
            tags: [],
            level: hierarchicalTag.level,
            fullName: hierarchicalTag.name,
            noteCount: hierarchicalTag.note_count,
          },
        });
      }

      // 创建笔记到标签的边
      for (const note of notes) {
        for (const tagString of note.tags || []) {
          // 为层次化标签的每一级都创建边
          const parsedTags = await invoke<string[]>('parse_and_add_tags', { tagStrings: [tagString] });
          
          for (const tagName of parsedTags) {
            const tagId = `tag-${tagName}`;
            const edgeId = `${note.id}-${tagId}`;
            
            if (!edgeMap.has(edgeId)) {
              edgeMap.set(edgeId, {
                id: edgeId,
                source: note.id,
                target: tagId,
                type: 'tag',
                weight: 2,
                metadata: {
                  fullTagName: tagName,
                },
              });
            }
          }
        }
      }

      // 创建标签之间的层次关系边
      for (const hierarchicalTag of allHierarchicalTags) {
        if (hierarchicalTag.parent) {
          const parentId = `tag-${hierarchicalTag.parent}`;
          const childId = `tag-${hierarchicalTag.name}`;
          const edgeId = `hierarchy-${parentId}-${childId}`;
          
          edgeMap.set(edgeId, {
            id: edgeId,
            source: parentId,
            target: childId,
            type: 'hierarchy',
            weight: 1,
            metadata: {
              relationshipType: 'parent-child',
            },
          });
        }
      }

      setNodes([...graphNodes, ...Array.from(tagNodes.values())]);
      setEdges(Array.from(edgeMap.values()));

      // 获取统计信息
      try {
        const linkStats = await invoke<GraphStats>('get_link_statistics');
        setStats(linkStats);
      } catch (err) {
        console.warn('Failed to get link statistics:', err);
      }

    } catch (err) {
      console.error('Failed to build knowledge graph:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  // 重建链接索引
  const rebuildIndex = useCallback(async () => {
    try {
      setLoading(true);
      
      // 首先获取配置来获取工作空间路径
      const config = await invoke<any>('get_config');
      if (!config.workspace_path) {
        throw new Error('未设置工作空间路径，请先在设置中配置工作空间');
      }

      // 获取所有笔记文件
      const noteFiles = await invoke<any[]>('list_notes', { dirPath: config.workspace_path });
      const notesData: [string, string, string, string][] = [];

      // 读取每个笔记文件的内容
      for (const noteFile of noteFiles) {
        try {
          const content = await invoke<string>('read_file_content', { path: noteFile.path });
          const title = noteFile.name.replace(/\.md$/, '');
          
          notesData.push([
            noteFile.path, // 使用文件路径作为 ID
            noteFile.path,
            title,
            content
          ]);
        } catch (err) {
          console.warn(`Failed to read note file ${noteFile.path}:`, err);
        }
      }

      // 重建链接索引
      await invoke('rebuild_link_index', { notesData });
      
      // 重新构建图谱
      await buildGraphFromNotes();
    } catch (err) {
      console.error('Failed to rebuild link index:', err);
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [buildGraphFromNotes]);

  // 初始化加载
  useEffect(() => {
    buildGraphFromNotes();
  }, [buildGraphFromNotes]);

  // 获取节点的详细信息
  const getNodeDetails = useCallback(async (nodeId: string) => {
    try {
      if (nodeId.startsWith('tag-')) {
        // 标签节点
        return {
          type: 'tag',
          name: nodeId.replace('tag-', ''),
        };
      } else {
        // 笔记节点 - 获取反向链接和相似笔记
        const [backlinks, similarNotes, outgoingLinks] = await Promise.all([
          invoke<BacklinkInfo[]>('get_backlinks', { noteId: nodeId }),
          invoke<SimilarNote[]>('find_similar_notes', { noteId: nodeId, limit: 10 }),
          invoke<string[]>('get_outgoing_links', { noteId: nodeId }),
        ]);

        return {
          type: 'note',
          backlinks,
          similarNotes,
          outgoingLinks,
        };
      }
    } catch (err) {
      console.error('Failed to get node details:', err);
      return null;
    }
  }, []);

  // 搜索节点
  const searchNodes = useCallback((query: string): GraphNode[] => {
    if (!query) return nodes;
    
    const lowerQuery = query.toLowerCase();
    return nodes.filter(node =>
      node.label.toLowerCase().includes(lowerQuery) ||
      node.metadata.tags.some(tag => tag.toLowerCase().includes(lowerQuery))
    );
  }, [nodes]);

  // 获取断链信息
  const getBrokenLinks = useCallback(async () => {
    try {
      return await invoke('get_broken_links');
    } catch (err) {
      console.error('Failed to get broken links:', err);
      return [];
    }
  }, []);

  // 获取孤立笔记
  const getOrphanedNotes = useCallback(async () => {
    try {
      return await invoke<string[]>('get_orphaned_notes');
    } catch (err) {
      console.error('Failed to get orphaned notes:', err);
      return [];
    }
  }, []);

  return {
    nodes,
    edges,
    loading,
    error,
    stats,
    buildGraphFromNotes,
    rebuildIndex,
    getNodeDetails,
    searchNodes,
    getBrokenLinks,
    getOrphanedNotes,
  };
};