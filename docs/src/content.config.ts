import { defineCollection } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

/**
 * Starlight 内容集合配置
 *
 * 使用 Starlight 的 docsLoader 从 src/content/docs/ 加载文档，
 * 并套用 docsSchema 解析 frontmatter。
 */
export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
};
