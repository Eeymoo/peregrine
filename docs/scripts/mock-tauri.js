/**
 * Tauri IPC mock（浏览器注入脚本，仅用于文档站截图管线）
 *
 * 通过 Playwright `addInitScript` 在页面任何脚本执行前注入，
 * 使根仓库 React 设置面板（`npm run dev`，纯浏览器环境）脱离 Tauri 运行：
 * - `get_config` / `build_shapes_ipc` / `list_materials` 返回 fixtures 真实数据
 *   （由 Rust `build_layers_shapes` 一次性导出，见 docs/scripts/fixtures/）
 * - 写操作（save_config / update_layer 等）作用于内存态 config，截图期间行为与真机一致
 * - 其余命令返回安全默认值，不产生网络请求
 *
 * 用法：node docs/scripts/capture-screenshots.mjs（内部引用本文件）
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const fixturesDir = path.join(path.dirname(fileURLToPath(import.meta.url)), 'fixtures');

/** 读取 fixtures 并生成注入用的页面脚本源码字符串。 */
export function buildMockInitScript() {
  const config = fs.readFileSync(path.join(fixturesDir, 'app-config.json'), 'utf8');
  const shapes = fs.readFileSync(path.join(fixturesDir, 'shapes-1920x1080.json'), 'utf8');
  const materials = fs.readFileSync(path.join(fixturesDir, 'materials.json'), 'utf8');

  // 注入脚本在页面上下文执行，无法访问 Node 模块，数据以内联字面量带入。
  return `
(() => {
  const config = ${config};
  const SHAPES = ${shapes};
  const MATERIALS = ${materials};

  // 多图层编辑模式（与 useConfigAppState 的 LAYERS_MODE_KEY 一致）。
  localStorage.setItem('peregrine:layers-mode', '1');

  let callbackSeq = 1;
  const callbacks = new Map();

  /** IPC 分发：模拟 src-tauri 的全部命令。 */
  async function invoke(cmd, args = {}) {
    switch (cmd) {
      // ---- 配置读写 ----
      case 'get_config':
        return structuredClone(config);
      case 'save_config':
        Object.assign(config, args.config);
        return null;
      case 'update_preferences':
        Object.assign(config.settings, args.preferences);
        return null;

      // ---- 图层 API（作用于内存态 config 的 active profile） ----
      case 'list_layers':
        return structuredClone(activeProfile().layers);
      case 'list_materials':
        return structuredClone(MATERIALS);
      case 'add_layer': {
        const mat = MATERIALS.find((m) => m.id === args.materialId);
        const layer = {
          id: 'layer-' + Math.random().toString(36).slice(2, 8),
          name: args.name,
          material: { type: 'builtin', id: args.materialId },
          params: mat ? mat.defaults : {},
          style: { color: [1, 1, 1, 1], opacity: 0.9, blend_mode: 'normal' },
          transform: defaultTransform(),
          visible: true,
          locked: false,
        };
        activeProfile().layers.push(layer);
        return structuredClone(layer);
      }
      case 'remove_layer': {
        const layers = activeProfile().layers;
        const i = layers.findIndex((l) => l.id === args.layerId);
        if (i >= 0) layers.splice(i, 1);
        return null;
      }
      case 'move_layer': {
        const layers = activeProfile().layers;
        const i = layers.findIndex((l) => l.id === args.layerId);
        if (i >= 0) {
          const [l] = layers.splice(i, 1);
          layers.splice(Math.min(args.newIndex, layers.length), 0, l);
        }
        return null;
      }
      case 'duplicate_layer': {
        const layers = activeProfile().layers;
        const src = layers.find((l) => l.id === args.layerId);
        if (!src) throw { code: 'NOT_FOUND', message: 'layer not found' };
        const copy = structuredClone(src);
        copy.id = 'layer-' + Math.random().toString(36).slice(2, 8);
        copy.name = src.name + ' 副本';
        layers.push(copy);
        return structuredClone(copy);
      }
      case 'update_layer': {
        const layer = activeProfile().layers.find((l) => l.id === args.layerId);
        if (layer) applyLayerPatch(layer, args.patch);
        return null;
      }

      // ---- 预览图元：返回 Rust 导出的真实 shapes ----
      case 'build_shapes_ipc':
        return structuredClone(SHAPES);

      // ---- Profile 管理 ----
      case 'list_profiles':
        return Object.keys(config.profiles);
      case 'get_active_profile_name':
        return config.active_profile;
      case 'get_profile':
        return structuredClone(config.profiles[args.name]);
      case 'set_active_profile':
        if (config.profiles[args.name]) config.active_profile = args.name;
        return null;
      case 'create_profile': {
        const base = structuredClone(config.profiles[config.active_profile]);
        config.profiles[args.name] = base;
        return structuredClone(base);
      }
      case 'rename_profile': {
        const p = config.profiles[args.oldName];
        if (p) {
          delete config.profiles[args.oldName];
          config.profiles[args.newName] = p;
          if (config.active_profile === args.oldName) config.active_profile = args.newName;
        }
        return null;
      }
      case 'delete_profile':
        delete config.profiles[args.name];
        return null;
      case 'copy_profile': {
        const base = config.profiles[args.baseName];
        const name = args.baseName + ' 副本';
        config.profiles[name] = structuredClone(base);
        return name;
      }
      case 'is_profile_legacy_compatible':
        return false;
      case 'update_profile_field':
        return null;

      // ---- 窗口 / 覆盖层 ----
      case 'list_window_titles':
        return ['Demo Game', 'Another Window'];
      case 'get_overlay_active':
        return false;
      case 'start_overlay':
      case 'stop_overlay':
      case 'focus_target_window':
      case 'relaunch_app':
      case 'restart_app':
        return null;
      case 'pick_image_path':
        return null;
      case 'set_crosshair_color':
        return null;

      // ---- 版本 / 更新 ----
      case 'get_app_version':
        return '0.2.4';
      case 'check_update':
        return { available: false };
      case 'download_install_update':
        return null;

      // ---- 遥测（截图环境 DSN 缺失，全部为无操作） ----
      case 'store_pending_report':
      case 'authorize_upload_all':
      case 'test_report':
        return null;

      // ---- 事件插件（listen / unlisten） ----
      case 'plugin:event|listen':
        return 0;
      case 'plugin:event|unlisten':
        return null;

      // ---- 窗口插件（set_title 等全部无操作） ----
      default:
        if (cmd.startsWith('plugin:')) return null;
        console.warn('[mock-tauri] unhandled command:', cmd, args);
        return null;
    }
  }

  function activeProfile() {
    return config.profiles[config.active_profile];
  }

  function defaultTransform() {
    return { scale: 1.0, rotation_deg: 0.0, offset_x: 0.0, offset_y: 0.0, mirror_x: false, mirror_y: false };
  }

  function applyLayerPatch(layer, patch) {
    if (!patch) return;
    if (patch.name !== undefined) layer.name = patch.name;
    if (patch.visible !== undefined) layer.visible = patch.visible;
    if (patch.locked !== undefined) layer.locked = patch.locked;
    if (patch.params !== undefined) layer.params = patch.params;
    if (patch.style !== undefined) Object.assign(layer.style, patch.style);
    if (patch.transform !== undefined) Object.assign(layer.transform, patch.transform);
  }

  // Tauri JS API 2.x 的内部契约（见 @tauri-apps/api/core.js）。
  window.__TAURI_INTERNALS__ = {
    invoke,
    transformCallback(callback, once) {
      const id = callbackSeq++;
      callbacks.set(id, (data) => {
        if (once) callbacks.delete(id);
        callback(data);
      });
      return id;
    },
    unregisterCallback(id) {
      callbacks.delete(id);
    },
    runCallback(id, data) {
      const cb = callbacks.get(id);
      if (cb) cb(data);
    },
    callbacks,
    convertFileSrc(filePath) {
      return 'asset://localhost/' + filePath;
    },
    metadata: {
      // label 取 'config'：ConfigApp 窗口承载图层编辑器与实时预览（多图层演示所在），
      // SettingsApp 是通用/快捷键等标签页，不适合做官网截图主体。
      currentWindow: { label: 'config' },
      currentWebview: { windowLabel: 'config', label: 'config' },
    },
  };

  // 事件插件的内部命名空间（listen 走 invoke，已在上分发）。
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {};
})();
`;
}
