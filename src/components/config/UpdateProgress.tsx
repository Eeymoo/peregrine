interface UpdateProgressProps {
  progress: number;
}

export function UpdateProgress({ progress }: UpdateProgressProps) {
  return (
    <div className="fixed bottom-4 right-4 z-50 bg-card border rounded-lg shadow-lg p-4 max-w-xs space-y-2">
      <p className="text-xs text-blue-500">更新中…</p>
      <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
        <div
          className="h-full bg-blue-500 rounded-full transition-all"
          style={{ width: `${progress || 30}%` }}
        />
      </div>
      {progress > 0 && (
        <p className="text-xs text-muted-foreground text-right">{progress}%</p>
      )}
    </div>
  );
}
