const FOLDER_TONES = [
  "text-accent-blue",
  "text-success",
  "text-warning",
  "text-danger",
  "text-secondary",
] as const;

export function folderTone(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) | 0;
  }
  return FOLDER_TONES[Math.abs(hash) % FOLDER_TONES.length];
}

export function fileMeta(name: string): {
  icon: string;
  tone: string;
  type: string;
} {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  if (ext === "pdf")
    return { icon: "picture_as_pdf", tone: "text-danger", type: "PDF" };
  if (ext === "note")
    return { icon: "sticky_note_2", tone: "text-accent-blue", type: "Note" };
  if (["png", "jpg", "jpeg", "gif", "webp", "svg"].includes(ext))
    return { icon: "image", tone: "text-warning", type: "Image" };
  if (["mp3", "wav", "m4a", "ogg"].includes(ext))
    return { icon: "audio_file", tone: "text-success", type: "Audio" };
  if (["mp4", "mov", "webm"].includes(ext))
    return { icon: "video_file", tone: "text-success", type: "Video" };
  return {
    icon: "description",
    tone: "text-text-secondary",
    type: ext ? ext.toUpperCase() : "File",
  };
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function formatDate(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  const diff = Date.now() - date.getTime();
  const oneDay = 24 * 60 * 60 * 1000;
  if (diff < oneDay) {
    const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
    const hours = Math.round(diff / (60 * 60 * 1000));
    if (hours < 1) {
      const minutes = Math.max(1, Math.round(diff / (60 * 1000)));
      return rtf.format(-minutes, "minute");
    }
    return rtf.format(-hours, "hour");
  }
  return date.toLocaleDateString(locale, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}
