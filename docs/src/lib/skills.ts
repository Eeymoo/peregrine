/**
 * 官方 Skills 物料加载器（构建期执行，纯 fs 读取，无运行时开销）。
 *
 * 数据源：仓库根 skills/<name>/SKILL.md 的 YAML frontmatter（name + description）。
 * 该目录是面向外部发布 的
 * skills.sh 开放技能生态布局——只展示这些官方物料，仓库内部 .agents/skills
 * 工作流技能不在站点外显。
 */
import fs from 'node:fs';
import path from 'node:path';

/** 单条官方 skill 物料。 */
export interface SkillEntry {
	/** skill 目录名（即 skill id，如 material-creation）。 */
	name: string;
	/** frontmatter description（触发时机描述，中文）。 */
	description: string;
}

/** 仓库标识（skills CLI 的安装源）。 */
export const SKILLS_REPO = 'eeymoo/peregrine';

/**
 * 读取 skills/ 下全部官方 skill 并按目录名排序。
 * 在 Astro 组件 frontmatter（构建期）调用；目录缺失时返回空列表以容错。
 * 路径解析基于 process.cwd()（dev / build 时均为 docs/ 目录）——不能用
 * import.meta.url：静态构建会把组件打进 dist chunk，相对其定位会指错位置。
 */
export function loadSkills(): SkillEntry[] {
	const skillsDir = path.resolve(process.cwd(), '../skills');
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

/** GitHub 上 SKILL.md 的源文件链接。 */
export function skillSourceUrl(name: string): string {
	return `https://github.com/Eeymoo/peregrine/blob/main/skills/${name}/SKILL.md`;
}

/** 单个 skill 的 skills CLI 安装命令（指向仓库内目录，README 方式一）。 */
export function skillInstallCmd(name: string): string {
	return `npx skills add https://github.com/eeymoo/peregrine/tree/main/skills/${name}`;
}
