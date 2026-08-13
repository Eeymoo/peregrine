/**
 * 构建时 GitHub Releases 数据获取（决策 D1：构建时 API，零版本号硬编码）。
 *
 * docs 构建时请求 GitHub Releases API，解析 assets[].browser_download_url。
 * 失败（限流 / 网络异常）返回 null，由页面降级为「查看 Releases」外链。
 */

/** 单个架构的下载资产（release.yml 产物命名：peregrine-v<ver>-windows-<arch>-setup.exe / .zip）。 */
export interface ArchAssets {
  /** NSIS 安装包直链（可能缺失）。 */
  setup?: string;
  /** 便携 zip 直链（可能缺失）。 */
  zip?: string;
}

export interface ReleaseInfo {
  /** tag 名，如 v0.2.4。 */
  tag: string;
  /** 是否预发布通道。 */
  prerelease: boolean;
  /** 按架构归类的资产。 */
  arches: Record<'x64' | 'x86' | 'arm64', ArchAssets>;
}

export interface ReleasesData {
  /** 最新稳定版（prerelease=false），无则缺省。 */
  stable?: ReleaseInfo;
  /** 最新预发布版（prerelease=true），无则缺省。 */
  prerelease?: ReleaseInfo;
}

const API_URL = 'https://api.github.com/repos/Eeymoo/peregrine/releases?per_page=20';

interface ApiAsset {
  name: string;
  browser_download_url: string;
}

interface ApiRelease {
  tag_name: string;
  prerelease: boolean;
  draft: boolean;
  assets: ApiAsset[];
}

/** 从资产列表解析三架构的 setup / zip 直链。 */
function parseArches(assets: ApiAsset[]): ReleaseInfo['arches'] {
  const arches: ReleaseInfo['arches'] = { x64: {}, x86: {}, arm64: {} };
  for (const a of assets) {
    const setup = a.name.match(/windows-(x64|x86|arm64)-setup\.exe$/);
    const zip = a.name.match(/windows-(x64|x86|arm64)\.zip$/);
    const m = setup ?? zip;
    if (!m) continue;
    const arch = m[1] as keyof ReleaseInfo['arches'];
    if (setup) arches[arch].setup = a.browser_download_url;
    else arches[arch].zip = a.browser_download_url;
  }
  return arches;
}

/** 判断一个 release 是否至少有一个可下载资产。 */
function hasAssets(info: ReleaseInfo): boolean {
  return Object.values(info.arches).some((a) => a.setup || a.zip);
}

/**
 * 拉取最新稳定版 / 预发布版各一个；失败返回 null（降级路径）。
 * 未认证限流 60 次/小时，docs 仅发版时构建，通常足够。
 */
export async function fetchReleases(): Promise<ReleasesData | null> {
  try {
    const res = await fetch(API_URL, {
      headers: {
        Accept: 'application/vnd.github+json',
        // GitHub API 要求 User-Agent。
        'User-Agent': 'peregrine-docs-build',
      },
      // 构建时请求不应无限挂起。
      signal: AbortSignal.timeout(15000),
    });
    if (!res.ok) return null;
    const list = (await res.json()) as ApiRelease[];
    if (!Array.isArray(list)) return null;
    const data: ReleasesData = {};
    for (const r of list) {
      if (r.draft) continue;
      const info: ReleaseInfo = { tag: r.tag_name, prerelease: r.prerelease, arches: parseArches(r.assets) };
      if (!hasAssets(info)) continue;
      if (r.prerelease) {
        data.prerelease ??= info;
      } else {
        data.stable ??= info;
      }
      if (data.stable && data.prerelease) break;
    }
    return data.stable || data.prerelease ? data : null;
  } catch {
    return null;
  }
}
