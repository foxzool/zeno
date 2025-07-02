import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface HierarchicalTag {
  name: string;
  level: number;
  parent?: string;
  children: string[];
  note_count: number;
  color?: string;
}

export interface TagStatistics {
  total_tags: number;
  root_tags: number;
  max_depth: number;
  avg_children: number;
  most_used_tag?: string;
}

export const useTagHierarchy = () => {
  const [allTags, setAllTags] = useState<HierarchicalTag[]>([]);
  const [rootTags, setRootTags] = useState<HierarchicalTag[]>([]);
  const [popularTags, setPopularTags] = useState<HierarchicalTag[]>([]);
  const [statistics, setStatistics] = useState<TagStatistics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 获取所有标签
  const fetchAllTags = useCallback(async () => {
    try {
      const tags = await invoke<HierarchicalTag[]>('get_all_tags');
      setAllTags(tags);
      return tags;
    } catch (err) {
      console.error('Failed to fetch all tags:', err);
      throw err;
    }
  }, []);

  // 获取根标签
  const fetchRootTags = useCallback(async () => {
    try {
      const tags = await invoke<HierarchicalTag[]>('get_root_tags');
      setRootTags(tags);
      return tags;
    } catch (err) {
      console.error('Failed to fetch root tags:', err);
      throw err;
    }
  }, []);

  // 获取热门标签
  const fetchPopularTags = useCallback(async (limit = 10) => {
    try {
      const tags = await invoke<HierarchicalTag[]>('get_popular_tags', { limit });
      setPopularTags(tags);
      return tags;
    } catch (err) {
      console.error('Failed to fetch popular tags:', err);
      throw err;
    }
  }, []);

  // 获取标签统计信息
  const fetchStatistics = useCallback(async () => {
    try {
      const stats = await invoke<TagStatistics>('get_tag_statistics');
      setStatistics(stats);
      return stats;
    } catch (err) {
      console.error('Failed to fetch tag statistics:', err);
      throw err;
    }
  }, []);

  // 搜索标签
  const searchTags = useCallback(async (query: string) => {
    try {
      return await invoke<HierarchicalTag[]>('search_tags', { query });
    } catch (err) {
      console.error('Failed to search tags:', err);
      return [];
    }
  }, []);

  // 获取标签的子标签
  const getTagChildren = useCallback(async (tagName: string) => {
    try {
      return await invoke<HierarchicalTag[]>('get_tag_children', { tagName });
    } catch (err) {
      console.error('Failed to get tag children:', err);
      return [];
    }
  }, []);

  // 获取标签的祖先标签
  const getTagAncestors = useCallback(async (tagName: string) => {
    try {
      return await invoke<HierarchicalTag[]>('get_tag_ancestors', { tagName });
    } catch (err) {
      console.error('Failed to get tag ancestors:', err);
      return [];
    }
  }, []);

  // 获取标签的后代标签
  const getTagDescendants = useCallback(async (tagName: string) => {
    try {
      return await invoke<HierarchicalTag[]>('get_tag_descendants', { tagName });
    } catch (err) {
      console.error('Failed to get tag descendants:', err);
      return [];
    }
  }, []);

  // 获取相关标签建议
  const getSuggestedTags = useCallback(async (tagName: string) => {
    try {
      return await invoke<HierarchicalTag[]>('suggest_related_tags', { tagName });
    } catch (err) {
      console.error('Failed to get suggested tags:', err);
      return [];
    }
  }, []);

  // 获取标签信息
  const getTagInfo = useCallback(async (tagName: string) => {
    try {
      return await invoke<HierarchicalTag | null>('get_tag_info', { tagName });
    } catch (err) {
      console.error('Failed to get tag info:', err);
      return null;
    }
  }, []);

  // 解析和添加标签
  const parseAndAddTags = useCallback(async (tagStrings: string[]) => {
    try {
      const result = await invoke<string[]>('parse_and_add_tags', { tagStrings });
      // 重新获取标签数据
      await Promise.all([fetchAllTags(), fetchRootTags(), fetchPopularTags(), fetchStatistics()]);
      return result;
    } catch (err) {
      console.error('Failed to parse and add tags:', err);
      throw err;
    }
  }, [fetchAllTags, fetchRootTags, fetchPopularTags, fetchStatistics]);

  // 从内容中提取标签
  const extractTagsFromContent = useCallback(async (content: string) => {
    try {
      return await invoke<string[]>('extract_tags_from_content', { content });
    } catch (err) {
      console.error('Failed to extract tags from content:', err);
      return [];
    }
  }, []);

  // 为内容建议标签
  const suggestTagsForContent = useCallback(async (content: string, existingTags: string[] = []) => {
    try {
      return await invoke<string[]>('suggest_tags_for_content', { content, existingTags });
    } catch (err) {
      console.error('Failed to suggest tags for content:', err);
      return [];
    }
  }, []);

  // 更新标签使用计数
  const updateTagUsage = useCallback(async (tagName: string, increment: boolean) => {
    try {
      await invoke('update_tag_usage', { tagName, increment });
      // 刷新相关数据
      await Promise.all([fetchAllTags(), fetchPopularTags(), fetchStatistics()]);
    } catch (err) {
      console.error('Failed to update tag usage:', err);
      throw err;
    }
  }, [fetchAllTags, fetchPopularTags, fetchStatistics]);

  // 重建标签层次结构
  const rebuildTagHierarchy = useCallback(async (notesTags: [string, string[]][]) => {
    try {
      setLoading(true);
      await invoke('rebuild_tag_hierarchy', { notesTags });
      // 重新获取所有数据
      await Promise.all([fetchAllTags(), fetchRootTags(), fetchPopularTags(), fetchStatistics()]);
    } catch (err) {
      console.error('Failed to rebuild tag hierarchy:', err);
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  }, [fetchAllTags, fetchRootTags, fetchPopularTags, fetchStatistics]);

  // 清理未使用的标签
  const cleanupUnusedTags = useCallback(async () => {
    try {
      await invoke('cleanup_unused_tags');
      // 重新获取数据
      await Promise.all([fetchAllTags(), fetchRootTags(), fetchPopularTags(), fetchStatistics()]);
    } catch (err) {
      console.error('Failed to cleanup unused tags:', err);
      throw err;
    }
  }, [fetchAllTags, fetchRootTags, fetchPopularTags, fetchStatistics]);

  // 初始化数据
  const initializeData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      await Promise.all([
        fetchAllTags(),
        fetchRootTags(),
        fetchPopularTags(),
        fetchStatistics(),
      ]);
    } catch (err) {
      console.error('Failed to initialize tag data:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [fetchAllTags, fetchRootTags, fetchPopularTags, fetchStatistics]);

  // 刷新所有数据
  const refreshData = useCallback(async () => {
    await initializeData();
  }, [initializeData]);

  // 自动从笔记重建标签层次结构
  const rebuildFromNotes = useCallback(async () => {
    try {
      setLoading(true);
      
      // 获取配置和工作空间路径
      const config = await invoke<any>('get_config');
      if (!config.workspace_path) {
        throw new Error('未设置工作空间路径');
      }

      // 获取所有笔记
      const noteFiles = await invoke<any[]>('list_notes', { dirPath: config.workspace_path });
      const notesTags: [string, string[]][] = [];

      // 从每个笔记中提取标签
      for (const noteFile of noteFiles) {
        try {
          const content = await invoke<string>('read_file_content', { path: noteFile.path });
          const extractedTags = await extractTagsFromContent(content);
          
          if (extractedTags.length > 0) {
            notesTags.push([noteFile.path, extractedTags]);
          }
        } catch (err) {
          console.warn(`Failed to process note ${noteFile.path}:`, err);
        }
      }

      // 重建标签层次结构
      await rebuildTagHierarchy(notesTags);
    } catch (err) {
      console.error('Failed to rebuild from notes:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [extractTagsFromContent, rebuildTagHierarchy]);

  // 初始化加载
  useEffect(() => {
    initializeData();
  }, [initializeData]);

  return {
    // 数据状态
    allTags,
    rootTags,
    popularTags,
    statistics,
    loading,
    error,

    // 查询方法
    searchTags,
    getTagChildren,
    getTagAncestors,
    getTagDescendants,
    getSuggestedTags,
    getTagInfo,

    // 操作方法
    parseAndAddTags,
    extractTagsFromContent,
    suggestTagsForContent,
    updateTagUsage,
    rebuildTagHierarchy,
    cleanupUnusedTags,

    // 工具方法
    refreshData,
    rebuildFromNotes,
  };
};