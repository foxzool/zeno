/**
 * Note Statistics Plugin for Zeno
 * 
 * 提供详细的笔记统计功能，包括：
 * - 笔记数量和字数统计
 * - 标签使用分析
 * - 写作模式分析
 * - 链接关系统计
 * - 时间趋势分析
 */

class NoteStatsPlugin {
  constructor() {
    this.isEnabled = false;
    this.config = null;
    this.statistics = {};
    this.updateTimer = null;
    this.cache = new Map();
    
    // 统计数据结构
    this.resetStatistics();
  }

  /**
   * 重置统计数据
   */
  resetStatistics() {
    this.statistics = {
      overview: {
        totalNotes: 0,
        totalWords: 0,
        totalCharacters: 0,
        totalTags: 0,
        totalLinks: 0,
        averageWordsPerNote: 0,
        lastUpdated: null
      },
      tags: {
        mostUsed: [],
        distribution: {},
        hierarchy: {}
      },
      writing: {
        dailyStats: {},
        monthlyStats: {},
        yearlyStats: {},
        mostProductiveDays: [],
        averageSessionLength: 0
      },
      content: {
        longestNote: null,
        shortestNote: null,
        mostLinkedNote: null,
        recentlyModified: [],
        wordFrequency: {}
      },
      relationships: {
        orphanNotes: [],
        hubNotes: [],
        linkDensity: 0,
        clusterAnalysis: {}
      }
    };
  }

  /**
   * 插件启用
   */
  async onEnable(config) {
    console.log('[NoteStats] Plugin enabled');
    this.isEnabled = true;
    this.config = config;
    
    // 注册命令
    await this.registerCommands();
    
    // 注册菜单
    await this.registerMenus();
    
    // 开始统计更新
    await this.startStatisticsUpdates();
    
    // 初始统计计算
    await this.updateStatistics();
    
    await this.sendNotification('success', 'Note Statistics plugin enabled');
  }

  /**
   * 插件停用
   */
  async onDisable() {
    console.log('[NoteStats] Plugin disabled');
    this.isEnabled = false;
    
    // 停止定时更新
    if (this.updateTimer) {
      clearInterval(this.updateTimer);
      this.updateTimer = null;
    }
    
    // 清理缓存
    this.cache.clear();
    
    await this.sendNotification('info', 'Note Statistics plugin disabled');
  }

  /**
   * 配置更新
   */
  async onConfigUpdate(newConfig) {
    console.log('[NoteStats] Config updated');
    this.config = newConfig;
    
    // 重新启动定时器
    await this.startStatisticsUpdates();
  }

  /**
   * 注册命令
   */
  async registerCommands() {
    const commands = [
      {
        id: 'note_stats.show_overview',
        name: 'Note Stats: Show Overview',
        description: 'Display general statistics overview',
        shortcut: 'Ctrl+Shift+S'
      },
      {
        id: 'note_stats.show_tag_analysis',
        name: 'Note Stats: Tag Analysis',
        description: 'Show detailed tag usage analysis'
      },
      {
        id: 'note_stats.show_writing_patterns',
        name: 'Note Stats: Writing Patterns',
        description: 'Analyze writing patterns and productivity'
      },
      {
        id: 'note_stats.export_stats',
        name: 'Note Stats: Export Statistics',
        description: 'Export statistics to file'
      },
      {
        id: 'note_stats.refresh_stats',
        name: 'Note Stats: Refresh',
        description: 'Manually refresh statistics'
      }
    ];

    for (const command of commands) {
      await this.callZenoAPI('register_command', command);
    }
  }

  /**
   * 注册菜单
   */
  async registerMenus() {
    const menu = {
      id: 'note_stats_menu',
      label: 'Statistics',
      items: [
        { id: 'overview', label: 'Overview', command: 'note_stats.show_overview' },
        { id: 'tags', label: 'Tag Analysis', command: 'note_stats.show_tag_analysis' },
        { id: 'writing', label: 'Writing Patterns', command: 'note_stats.show_writing_patterns' },
        { id: 'separator1', type: 'separator' },
        { id: 'export', label: 'Export Statistics', command: 'note_stats.export_stats' },
        { id: 'refresh', label: 'Refresh', command: 'note_stats.refresh_stats' }
      ]
    };

    await this.callZenoAPI('register_menu', menu);
  }

  /**
   * 开始统计更新定时器
   */
  async startStatisticsUpdates() {
    if (this.updateTimer) {
      clearInterval(this.updateTimer);
    }

    const interval = (this.config?.settings?.update_interval || 300) * 1000;
    this.updateTimer = setInterval(() => {
      this.updateStatistics();
    }, interval);
  }

  /**
   * 更新统计数据
   */
  async updateStatistics() {
    try {
      console.log('[NoteStats] Updating statistics...');
      
      // 获取所有笔记
      const notes = await this.getAllNotes();
      const tags = await this.getAllTags();
      const links = await this.getAllLinks();
      
      // 重置统计
      this.resetStatistics();
      
      // 计算基础统计
      await this.calculateOverviewStats(notes);
      
      // 计算标签统计
      await this.calculateTagStats(notes, tags);
      
      // 计算写作模式
      await this.calculateWritingPatterns(notes);
      
      // 计算内容统计
      await this.calculateContentStats(notes);
      
      // 计算关系统计
      await this.calculateRelationshipStats(notes, links);
      
      this.statistics.overview.lastUpdated = new Date().toISOString();
      
      console.log('[NoteStats] Statistics updated');
      
      // 发送更新事件
      await this.sendEvent('stats_updated', this.statistics);
      
    } catch (error) {
      console.error('[NoteStats] Failed to update statistics:', error);
      await this.sendNotification('error', `Failed to update statistics: ${error.message}`);
    }
  }

  /**
   * 计算概览统计
   */
  async calculateOverviewStats(notes) {
    const stats = this.statistics.overview;
    
    stats.totalNotes = notes.length;
    stats.totalWords = 0;
    stats.totalCharacters = 0;
    
    const tagSet = new Set();
    let totalLinks = 0;
    
    for (const note of notes) {
      const wordCount = this.countWords(note.content || '');
      stats.totalWords += wordCount;
      stats.totalCharacters += (note.content || '').length;
      
      // 统计标签
      if (note.tags) {
        note.tags.forEach(tag => tagSet.add(tag));
      }
      
      // 统计链接
      const linkCount = this.countLinks(note.content || '');
      totalLinks += linkCount;
    }
    
    stats.totalTags = tagSet.size;
    stats.totalLinks = totalLinks;
    stats.averageWordsPerNote = notes.length > 0 ? 
      Math.round(stats.totalWords / notes.length) : 0;
  }

  /**
   * 计算标签统计
   */
  async calculateTagStats(notes, allTags) {
    const tagStats = this.statistics.tags;
    const tagCounts = new Map();
    
    // 统计标签使用频率
    for (const note of notes) {
      if (note.tags) {
        note.tags.forEach(tag => {
          tagCounts.set(tag, (tagCounts.get(tag) || 0) + 1);
        });
      }
    }
    
    // 生成最常用标签列表
    tagStats.mostUsed = Array.from(tagCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 20)
      .map(([tag, count]) => ({ tag, count, percentage: (count / notes.length * 100).toFixed(1) }));
    
    // 标签分布
    tagStats.distribution = Object.fromEntries(tagCounts);
    
    // 分析标签层次结构
    tagStats.hierarchy = this.analyzeTagHierarchy(Array.from(tagCounts.keys()));
  }

  /**
   * 计算写作模式
   */
  async calculateWritingPatterns(notes) {
    const patterns = this.statistics.writing;
    const dailyStats = {};
    const monthlyStats = {};
    const yearlyStats = {};
    
    for (const note of notes) {
      if (!note.modified_time && !note.created_time) continue;
      
      const date = new Date(note.modified_time || note.created_time);
      const day = date.toISOString().split('T')[0];
      const month = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
      const year = date.getFullYear().toString();
      
      const wordCount = this.countWords(note.content || '');
      
      // 日统计
      if (!dailyStats[day]) {
        dailyStats[day] = { notes: 0, words: 0 };
      }
      dailyStats[day].notes++;
      dailyStats[day].words += wordCount;
      
      // 月统计
      if (!monthlyStats[month]) {
        monthlyStats[month] = { notes: 0, words: 0 };
      }
      monthlyStats[month].notes++;
      monthlyStats[month].words += wordCount;
      
      // 年统计
      if (!yearlyStats[year]) {
        yearlyStats[year] = { notes: 0, words: 0 };
      }
      yearlyStats[year].notes++;
      yearlyStats[year].words += wordCount;
    }
    
    patterns.dailyStats = dailyStats;
    patterns.monthlyStats = monthlyStats;
    patterns.yearlyStats = yearlyStats;
    
    // 找出最高产的日子
    patterns.mostProductiveDays = Object.entries(dailyStats)
      .sort((a, b) => b[1].words - a[1].words)
      .slice(0, 10)
      .map(([date, stats]) => ({ date, ...stats }));
  }

  /**
   * 计算内容统计
   */
  async calculateContentStats(notes) {
    const content = this.statistics.content;
    
    if (notes.length === 0) return;
    
    let longest = notes[0];
    let shortest = notes[0];
    let mostLinked = notes[0];
    let maxLinks = 0;
    
    const wordFreq = {};
    
    for (const note of notes) {
      const wordCount = this.countWords(note.content || '');
      const linkCount = this.countLinks(note.content || '');
      
      // 最长和最短笔记
      if (wordCount > this.countWords(longest.content || '')) {
        longest = note;
      }
      if (wordCount < this.countWords(shortest.content || '')) {
        shortest = note;
      }
      
      // 最多链接的笔记
      if (linkCount > maxLinks) {
        maxLinks = linkCount;
        mostLinked = note;
      }
      
      // 词频统计
      const words = this.extractWords(note.content || '');
      words.forEach(word => {
        if (word.length > 2) { // 忽略太短的词
          wordFreq[word] = (wordFreq[word] || 0) + 1;
        }
      });
    }
    
    content.longestNote = {
      title: longest.title,
      path: longest.path,
      wordCount: this.countWords(longest.content || '')
    };
    
    content.shortestNote = {
      title: shortest.title,
      path: shortest.path,
      wordCount: this.countWords(shortest.content || '')
    };
    
    content.mostLinkedNote = {
      title: mostLinked.title,
      path: mostLinked.path,
      linkCount: maxLinks
    };
    
    // 最近修改的笔记
    content.recentlyModified = notes
      .filter(note => note.modified_time)
      .sort((a, b) => new Date(b.modified_time) - new Date(a.modified_time))
      .slice(0, 10)
      .map(note => ({
        title: note.title,
        path: note.path,
        modifiedTime: note.modified_time
      }));
    
    // 词频（取前50个）
    content.wordFrequency = Object.entries(wordFreq)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 50)
      .reduce((obj, [word, count]) => {
        obj[word] = count;
        return obj;
      }, {});
  }

  /**
   * 计算关系统计
   */
  async calculateRelationshipStats(notes, links) {
    const relationships = this.statistics.relationships;
    
    const linkMap = new Map();
    const noteMap = new Map();
    
    // 建立笔记映射
    notes.forEach(note => {
      noteMap.set(note.id, note);
      linkMap.set(note.id, { incoming: 0, outgoing: 0 });
    });
    
    // 统计链接
    let totalLinks = 0;
    if (links && Array.isArray(links)) {
      links.forEach(link => {
        totalLinks++;
        if (linkMap.has(link.source)) {
          linkMap.get(link.source).outgoing++;
        }
        if (linkMap.has(link.target)) {
          linkMap.get(link.target).incoming++;
        }
      });
    }
    
    // 孤立笔记（没有任何链接）
    relationships.orphanNotes = Array.from(linkMap.entries())
      .filter(([id, links]) => links.incoming === 0 && links.outgoing === 0)
      .map(([id]) => ({
        title: noteMap.get(id)?.title || id,
        path: noteMap.get(id)?.path || ''
      }))
      .slice(0, 20);
    
    // 中心笔记（链接最多）
    relationships.hubNotes = Array.from(linkMap.entries())
      .map(([id, links]) => ({
        title: noteMap.get(id)?.title || id,
        path: noteMap.get(id)?.path || '',
        totalLinks: links.incoming + links.outgoing,
        incoming: links.incoming,
        outgoing: links.outgoing
      }))
      .sort((a, b) => b.totalLinks - a.totalLinks)
      .slice(0, 10);
    
    // 链接密度
    relationships.linkDensity = notes.length > 1 ? 
      (totalLinks / (notes.length * (notes.length - 1))) : 0;
  }

  /**
   * 处理命令
   */
  async handleCommand(commandData) {
    const { command_id } = commandData;
    
    switch (command_id) {
      case 'note_stats.show_overview':
        await this.showOverview();
        break;
      case 'note_stats.show_tag_analysis':
        await this.showTagAnalysis();
        break;
      case 'note_stats.show_writing_patterns':
        await this.showWritingPatterns();
        break;
      case 'note_stats.export_stats':
        await this.exportStatistics();
        break;
      case 'note_stats.refresh_stats':
        await this.updateStatistics();
        await this.sendNotification('success', 'Statistics refreshed');
        break;
    }
  }

  /**
   * 显示概览
   */
  async showOverview() {
    const stats = this.statistics.overview;
    const message = `
📊 **Note Statistics Overview**

📝 **Notes**: ${stats.totalNotes.toLocaleString()}
📖 **Total Words**: ${stats.totalWords.toLocaleString()}
🔤 **Characters**: ${stats.totalCharacters.toLocaleString()}
🏷️ **Unique Tags**: ${stats.totalTags.toLocaleString()}
🔗 **Links**: ${stats.totalLinks.toLocaleString()}
📏 **Avg Words/Note**: ${stats.averageWordsPerNote}

🕒 **Last Updated**: ${stats.lastUpdated ? new Date(stats.lastUpdated).toLocaleString() : 'Never'}
    `.trim();
    
    await this.sendNotification('info', message);
  }

  /**
   * 显示标签分析
   */
  async showTagAnalysis() {
    const tagStats = this.statistics.tags;
    const topTags = tagStats.mostUsed.slice(0, 10);
    
    let message = '🏷️ **Top Tags**\n\n';
    topTags.forEach((item, index) => {
      message += `${index + 1}. ${item.tag}: ${item.count} (${item.percentage}%)\n`;
    });
    
    await this.sendNotification('info', message);
  }

  /**
   * 工具方法
   */
  countWords(text) {
    if (!text) return 0;
    return text.trim().split(/\s+/).filter(word => word.length > 0).length;
  }

  countLinks(text) {
    if (!text) return 0;
    const wikiLinks = (text.match(/\[\[.*?\]\]/g) || []).length;
    const mdLinks = (text.match(/\[.*?\]\(.*?\)/g) || []).length;
    return wikiLinks + mdLinks;
  }

  extractWords(text) {
    if (!text) return [];
    return text.toLowerCase()
      .replace(/[^\w\s]/g, ' ')
      .split(/\s+/)
      .filter(word => word.length > 0);
  }

  analyzeTagHierarchy(tags) {
    const hierarchy = {};
    
    tags.forEach(tag => {
      const parts = tag.split('/');
      let current = hierarchy;
      
      parts.forEach(part => {
        if (!current[part]) {
          current[part] = { children: {}, count: 0 };
        }
        current[part].count++;
        current = current[part].children;
      });
    });
    
    return hierarchy;
  }

  /**
   * API 调用方法
   */
  async getAllNotes() {
    try {
      const response = await this.callZenoAPI('get_all_notes');
      return response.data || [];
    } catch (error) {
      console.error('[NoteStats] Failed to get notes:', error);
      return [];
    }
  }

  async getAllTags() {
    try {
      const response = await this.callZenoAPI('get_all_tags');
      return response.data || [];
    } catch (error) {
      console.error('[NoteStats] Failed to get tags:', error);
      return [];
    }
  }

  async getAllLinks() {
    try {
      const response = await this.callZenoAPI('get_all_links');
      return response.data || [];
    } catch (error) {
      console.error('[NoteStats] Failed to get links:', error);
      return [];
    }
  }

  async exportStatistics() {
    const format = this.config?.settings?.export_format || 'json';
    const data = this.formatExportData(format);
    
    try {
      await this.callZenoAPI('save_file', {
        path: `.zeno/stats/statistics_${Date.now()}.${format}`,
        content: data
      });
      
      await this.sendNotification('success', `Statistics exported as ${format.toUpperCase()}`);
    } catch (error) {
      await this.sendNotification('error', `Export failed: ${error.message}`);
    }
  }

  formatExportData(format) {
    switch (format) {
      case 'json':
        return JSON.stringify(this.statistics, null, 2);
      case 'csv':
        return this.formatAsCSV(this.statistics);
      case 'markdown':
        return this.formatAsMarkdown(this.statistics);
      default:
        return JSON.stringify(this.statistics, null, 2);
    }
  }

  // 其他辅助方法...
  async callZenoAPI(endpoint, data = {}) {
    // API 调用实现
    console.log(`[NoteStats] API call: ${endpoint}`, data);
    return { success: true, data: null };
  }

  async sendNotification(type, message) {
    console.log(`[NoteStats] ${type.toUpperCase()}: ${message}`);
  }

  async sendEvent(eventType, data) {
    console.log(`[NoteStats] Event: ${eventType}`, data);
  }
}

// 创建并导出插件实例
const plugin = new NoteStatsPlugin();

if (typeof module !== 'undefined' && module.exports) {
  module.exports = plugin;
}

if (typeof window !== 'undefined') {
  window.ZenoNoteStatsPlugin = plugin;
}