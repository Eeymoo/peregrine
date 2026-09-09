/**
 * Skills 开发物料加载器（构建期执行，纯 fs 读取，无运行时开销）。
 *
 * 数据源：仓库根 .agents/skills/<name>/SKILL.md 的 YAML frontmatter
 * （name + description），与 aukcraft.org/skills 的物料同一体裁——
 * agent skills 即「从日常工作中提炼的工程经验，随取随用」。
 */
import fs from 'node:fs';
import path from 'node:path';

/** 单条 skill 物料：目录名 + frontmatter 描述。 */
export interface SkillEntry {
	/** skill 目录名（即 skill id，如 release / bugfix）。 */
	name: string;
	/** frontmatter description（单行中文描述）。 */
	description: string;
}

/**
 * 读取全部 skills 并按目录名排序。
 * 在 Astro 组件 frontmatter（构建期）调用；目录缺失时返回空列表以容错。
 * 路径解析基于 process.cwd()（dev / build 时均为 docs/ 目录）——不能用
 * import.meta.url：静态构建会把组件打进 dist chunk，相对其定位会指错位置。
 */
export function loadSkills(): SkillEntry[] {
	const skillsDir = path.resolve(process.cwd(), '../.agents/skills');
	if (!fs.existsSync(skillsDir)) return [];
	return fs
		.readdirSync(skillsDir)
		.filter((d) => fs.statSync(path.join(skillsDir, d)).isDirectory())
		.sort()
		.map((name) => {
			const file = path.join(skillsDir, name, 'SKILL.md');
			if (!fs.existsSync(file)) return { name, description: '' };
			const raw = fs.readFileSync(file, 'utf8');
			const fm = raw.match(/^---\r?\n([\s\S]*?)\r?\n---/)?.[1] ?? '';
			const description =
				fm.match(/^description:\s*(.+)$/m)?.[1]?.trim().replace(/^["']|["']$/g, '') ?? '';
			return { name, description };
		});
}

/** GitHub 上 SKILL.md 的源文件链接（供页面「查看源文件」入口）。 */
export function skillSourceUrl(name: string): string {
	return `https://github.com/Eeymoo/peregrine/blob/main/.agents/skills/${name}/SKILL.md`;
}
