import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Plus, Pencil, Copy, Trash2 } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import {
  listProfiles,
  createProfile,
  renameProfile,
  deleteProfile,
  setActiveProfile,
  copyProfile,
} from "@/lib/api";

interface ProfileManagerProps {
  /** 当前激活的配置文件名称 */
  activeProfile: string;
  /** 配置文件列表（可选），不传则内部加载 */
  profiles?: string[];
  /** 配置文件切换后的回调函数，接收新的配置文件名称 */
  onActiveProfileChange?: (name: string) => void;
  /** 配置文件列表变化后的回调函数，接收新的配置文件列表 */
  onProfilesChange?: (profiles: string[]) => void;
}

/**
 * Profile（配置）管理器。
 *
 * 支持下拉选择、新建、重命名、删除、复制当前 active profile。
 * 所有变更通过 Tauri commands 持久化到后端。
 */
export function ProfileManager({
  activeProfile,
  profiles: externalProfiles,
  onActiveProfileChange,
  onProfilesChange,
}: ProfileManagerProps) {
  const { t } = useI18n();
  const [profiles, setProfiles] = useState<string[]>(externalProfiles ?? []);
  const [isEditing, setIsEditing] = useState(false);
  const [inputName, setInputName] = useState("");
  const [editMode, setEditMode] = useState<"create" | "rename">("create");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refreshProfiles = useCallback(async () => {
    try {
      const list = await listProfiles();
      list.sort();
      setProfiles(list);
      onProfilesChange?.(list);
    } catch (e) {
      console.error("Failed to list profiles:", e);
    }
  }, [onProfilesChange]);

  // 外部传入的 profiles 优先。
  const profileList = externalProfiles ?? profiles;

  const handleCreate = async () => {
    const name = inputName.trim();
    if (!name) return;
    setBusy(true);
    setError(null);
    try {
      await createProfile(name);
      await refreshProfiles();
      await setActiveProfile(name);
      onActiveProfileChange?.(name);
      setInputName("");
      setIsEditing(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleRename = async () => {
    const newName = inputName.trim();
    if (!newName || newName === activeProfile) return;
    setBusy(true);
    setError(null);
    try {
      await renameProfile(activeProfile, newName);
      await refreshProfiles();
      onActiveProfileChange?.(newName);
      setInputName("");
      setIsEditing(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async () => {
    if (profileList.length <= 1) return;
    setBusy(true);
    setError(null);
    try {
      await deleteProfile(activeProfile);
      await refreshProfiles();
      const remaining = await listProfiles();
      remaining.sort();
      const next = remaining[0] || "";
      if (next) {
        await setActiveProfile(next);
        onActiveProfileChange?.(next);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDuplicate = async () => {
    if (!activeProfile) return;
    setBusy(true);
    setError(null);
    try {
      const newName = await copyProfile(activeProfile);
      await refreshProfiles();
      await setActiveProfile(newName);
      onActiveProfileChange?.(newName);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const switchProfile = async (name: string) => {
    if (name === activeProfile) return;
    setBusy(true);
    setError(null);
    try {
      await setActiveProfile(name);
      onActiveProfileChange?.(name);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleConfirm = async () => {
    if (editMode === "create") {
      await handleCreate();
    } else {
      await handleRename();
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Select value={activeProfile} onValueChange={switchProfile} disabled={busy}>
        <SelectTrigger className="h-7 text-xs w-40">
          <SelectValue placeholder={t("profile.selectPlaceholder")} />
        </SelectTrigger>
        <SelectContent>
          {profileList.map((p) => (
            <SelectItem key={p} value={p} className="text-xs">
              {p}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {isEditing ? (
        <div className="flex items-center gap-1">
          <input
            type="text"
            value={inputName}
            onChange={(e) => setInputName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                handleConfirm();
              } else if (e.key === "Escape") {
                setIsEditing(false);
                setInputName("");
                setError(null);
              }
            }}
            placeholder={t("profile.namePlaceholder")}
            className="h-7 px-2 text-xs rounded border bg-background w-32"
            disabled={busy}
            autoFocus
          />
          <Button
            size="sm"
            className="h-7 text-xs px-2"
            onClick={handleConfirm}
            disabled={busy || !inputName.trim()}
          >
            {editMode === "create" ? t("common.add") : t("common.save")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            className="h-7 text-xs px-2"
            onClick={() => {
              setIsEditing(false);
              setInputName("");
              setError(null);
            }}
            disabled={busy}
          >
            {t("common.cancel")}
          </Button>
        </div>
      ) : (
        <div className="flex items-center gap-1">
          <Button
            variant="outline"
            size="icon"
            className="h-7 w-7"
            onClick={() => {
              setIsEditing(true);
              setInputName("");
              setEditMode("create");
            }}
            disabled={busy}
            title={t("profile.new")}
          >
            <Plus className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            className="h-7 w-7"
            onClick={() => {
              setIsEditing(true);
              setInputName(activeProfile);
              setEditMode("rename");
            }}
            disabled={busy || !activeProfile}
            title={t("profile.rename")}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            className="h-7 w-7"
            onClick={handleDuplicate}
            disabled={busy || !activeProfile}
            title={t("common.copy")}
          >
            <Copy className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            className="h-7 w-7"
            onClick={handleDelete}
            disabled={busy || profileList.length <= 1 || !activeProfile}
            title={t("common.delete")}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      )}

      {error && (
        <span className="text-xs text-destructive ml-1">{error}</span>
      )}
    </div>
  );
}
