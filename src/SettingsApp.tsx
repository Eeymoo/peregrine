import { Separator } from "@/components/ui/separator";

export default function SettingsApp() {
  return (
    <div className="h-screen flex flex-col bg-background text-foreground p-6">
      <h1 className="text-xl font-semibold">设置</h1>
      <p className="text-sm text-muted-foreground mt-1">
        更多设置项将在后续版本加入。
      </p>

      <Separator className="my-4" />

      <div className="space-y-3">
        <h2 className="text-lg font-medium">关于 Peregrine</h2>
        <p className="text-sm text-muted-foreground">
          Peregrine 是一款桌面辅助贴图（准心 / 覆盖层）工具，主要用途是帮助玩家缓解 3D 眩晕。
        </p>
        <ul className="text-sm space-y-1 text-muted-foreground">
          <li>版本：0.1.1</li>
          <li>许可：PolyForm Noncommercial · 个人免费 · 禁止商业贩卖</li>
          <li>仓库：https://github.com/eeymoo/peregrine</li>
        </ul>
      </div>

      <div className="mt-auto text-xs text-muted-foreground text-right">
        Peregrine v0.1.1
      </div>
    </div>
  );
}
