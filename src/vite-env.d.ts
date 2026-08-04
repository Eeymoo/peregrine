/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** GlitchTip TEST 项目 DSN（开发构建）。 */
  readonly VITE_GLITCHTIP_DSN_TEST?: string;
  /** GlitchTip 正式项目 DSN（正式构建）。 */
  readonly VITE_GLITCHTIP_DSN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
