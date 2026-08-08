import { useState } from "react";
import { Download, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { MarkdownReleaseNotes } from "@/components/MarkdownReleaseNotes";
import { useI18n } from "@/lib/i18n";

interface UpdateDialogProps {
  version: string;
  body?: string;
  onUpdate: () => void;
  onLater: () => void;
}

export function UpdateDialog({ version, body, onUpdate, onLater }: UpdateDialogProps) {
  const { t } = useI18n();
  const [isUpdating, setIsUpdating] = useState(false);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-card border rounded-lg shadow-lg p-4 max-w-sm w-full mx-4 space-y-2">
        <div className="flex items-start justify-between gap-2">
          <p className="text-sm font-medium">
            {t("settings.updateAvailable")}：v{version}
          </p>
          <button
            type="button"
            aria-label={t("common.close")}
            onClick={onLater}
            className="text-muted-foreground hover:text-foreground shrink-0"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
        {body && (
          <div className="text-xs text-muted-foreground max-h-48 overflow-y-auto">
            <MarkdownReleaseNotes body={body} />
          </div>
        )}
        <div className="flex gap-2">
          <Button
            size="sm"
            disabled={isUpdating}
            onClick={() => {
              setIsUpdating(true);
              onUpdate();
            }}
          >
            <Download className="h-3.5 w-3.5" />
            {t("settings.updateNow")}
          </Button>
          <Button variant="outline" size="sm" onClick={onLater}>
            {t("settings.updateLater")}
          </Button>
        </div>
      </div>
    </div>
  );
}
