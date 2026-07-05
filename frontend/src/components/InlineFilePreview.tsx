import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { MarkdownEditor } from "./MarkdownEditor";
import { StateMessage } from "./StateMessage";
import { kyInstance } from "../api/mutator";
import { uploadFile } from "../api/upload";
import { pushNotification } from "../notifications/store";
import type {
  GetFileContent200,
  UpdateFileTextBody,
} from "../api/generated/backend.schemas";
import { fileContentKeyFor, useFileContent } from "../hooks/useFile";
import { ImageLightbox } from "./ImageLightbox";

interface InlineFilePreviewProps {
  fileId: string | null;
}

type SaveState = "idle" | "saving" | "saved" | "error";
const AUTOSAVE_DELAY_MS = 1500;
const AUTOSAVE_SAVED_VISIBLE_MS = 1600;
const PDF_LOAD_FALLBACK_MS = 1200;

function SkeletonBar({ className }: { className: string }) {
  return (
    <div
      class={`relative overflow-hidden rounded bg-overlay-faint before:absolute before:inset-0 before:-translate-x-full before:animate-[skeleton-shimmer_1.35s_ease-in-out_infinite] before:bg-gradient-to-r before:from-transparent before:via-white/15 before:to-transparent motion-reduce:before:animate-none ${className}`}
    />
  );
}

function FilePreviewSkeleton() {
  return (
    <div class="p-5" aria-hidden>
      <SkeletonBar className="h-[28rem] min-h-[18rem] w-full rounded-lg bg-overlay-soft" />
    </div>
  );
}

function escapeHtmlAttribute(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

export function InlineFilePreview({ fileId }: InlineFilePreviewProps) {
  const { t } = useTranslation();
  const { mutate } = useSWRConfig();
  const { data, error } = useFileContent(fileId ?? undefined);
  const activeData = fileId && data?.file.id === fileId ? data : undefined;
  const file = activeData?.file;
  const content = activeData?.content;
  const showLoading = Boolean(fileId && !activeData && !error);
  const isText = content?.kind === "text";
  const [draft, setDraft] = useState<string | null>(null);
  const [savedText, setSavedText] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const initializedFileRef = useRef<string | null>(null);
  const urlContent = content?.kind === "url" ? content : null;
  const isPdf =
    urlContent?.content_type === "application/pdf" ||
    Boolean(urlContent && file?.name.toLowerCase().endsWith(".pdf"));
  const isImage = Boolean(urlContent?.content_type.startsWith("image/"));
  const pdfSourceUrl = isPdf && urlContent ? urlContent.url : null;
  const imageSourceUrl = isImage && urlContent ? urlContent.url : null;
  const [pdfLoaded, setPdfLoaded] = useState(false);
  const [imageLoaded, setImageLoaded] = useState(false);
  const [imageError, setImageError] = useState(false);
  const [imageExpanded, setImageExpanded] = useState(false);
  const pdfDocument = pdfSourceUrl
    ? `<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <style>
      html,
      body {
        width: 100%;
        height: 100%;
        margin: 0;
        background: #fff;
        overflow: hidden;
      }
      embed {
        width: 100%;
        height: 100%;
        border: 0;
        display: block;
      }
    </style>
  </head>
  <body>
    <embed src="${escapeHtmlAttribute(pdfSourceUrl)}" type="application/pdf" />
  </body>
</html>`
    : null;

  useEffect(() => {
    if (!pdfSourceUrl) {
      setPdfLoaded(false);
      return;
    }

    setPdfLoaded(false);
    const fallback = window.setTimeout(
      () => setPdfLoaded(true),
      PDF_LOAD_FALLBACK_MS,
    );
    return () => window.clearTimeout(fallback);
  }, [pdfSourceUrl]);

  useEffect(() => {
    setImageLoaded(false);
    setImageError(false);
    setImageExpanded(false);
  }, [imageSourceUrl]);

  useEffect(() => {
    if (!fileId || content?.kind !== "text") {
      initializedFileRef.current = null;
      setDraft(null);
      setSavedText(null);
      setSaveState("idle");
      return;
    }
    if (initializedFileRef.current === fileId) return;
    initializedFileRef.current = fileId;
    setDraft(content.content);
    setSavedText(content.content);
    setSaveState("idle");
  }, [fileId, content]);

  useEffect(() => {
    if (!fileId || draft === null || savedText === null) return;
    if (draft === savedText) {
      setSaveState((state) => (state === "saving" ? "idle" : state));
      return;
    }

    const handle = window.setTimeout(async () => {
      setSaveState("saving");
      try {
        const body: UpdateFileTextBody = { text: draft };
        await kyInstance.put(`files/${fileId}/text`, { json: body });
        setSavedText(draft);
        setSaveState("saved");
        await mutate(
          fileContentKeyFor(fileId),
          (prev: GetFileContent200 | undefined) =>
            prev
              ? { ...prev, content: { kind: "text" as const, content: draft } }
              : prev,
          { revalidate: false },
        );
      } catch {
        setSaveState("error");
      }
    }, AUTOSAVE_DELAY_MS);

    return () => window.clearTimeout(handle);
  }, [draft, savedText, fileId, mutate]);

  useEffect(() => {
    if (saveState !== "saved") return;
    const handle = window.setTimeout(
      () => setSaveState("idle"),
      AUTOSAVE_SAVED_VISIBLE_MS,
    );
    return () => window.clearTimeout(handle);
  }, [saveState]);

  const handleEditorImagePaste = async (pasted: File) => {
    if (!file) return "";
    const ext = pasted.type.split("/")[1] || "png";
    const id = crypto.randomUUID().replace(/-/g, "");
    const named = new File([pasted], `image-${id}.${ext}`, {
      type: pasted.type,
    });
    try {
      const result = await uploadFile({
        file: named,
        parentId: file.parent_id,
        parentKind: file.parent_kind,
      });
      return `/api/files/${result.id}/raw`;
    } catch {
      pushNotification({
        kind: "error",
        message: t("file.errors.imageUpload"),
      });
      throw new Error("image_upload_failed");
    }
  };

  return (
    <>
      <section class="mt-6 flex min-h-[18rem] flex-1 flex-col overflow-hidden rounded-[12px] border border-border-subtle bg-surface-dark">
        {(showLoading || file) && (
          <div class="flex items-center justify-between gap-3 border-b border-border-subtle p-4">
            <div class="min-w-0">
              {showLoading ? (
                <SkeletonBar className="mt-1 h-4 w-48 max-w-[48vw] bg-overlay-soft" />
              ) : file ? (
                <div class="flex min-w-0 items-center gap-2">
                  <h3 class="truncate text-sm font-bold text-text-primary">
                    {file.name}
                  </h3>
                  {isText && saveState !== "idle" && (
                    <span
                      class={`shrink-0 text-xs ${
                        saveState === "error"
                          ? "text-danger"
                          : "text-text-secondary"
                      }`}
                      aria-live="polite"
                    >
                      {saveState === "saving" && t("file.autosave.saving")}
                      {saveState === "saved" && t("file.autosave.saved")}
                      {saveState === "error" && t("file.errors.save")}
                    </span>
                  )}
                </div>
              ) : null}
            </div>
          </div>
        )}

        <div class="flex min-h-[18rem] flex-1 flex-col bg-surface-container-low">
          {showLoading && <FilePreviewSkeleton />}

          {error && (
            <div class="p-5">
              <StateMessage tone="danger">{t("file.errors.load")}</StateMessage>
            </div>
          )}

          {content?.kind === "text" && draft !== null && file && (
            <div class="flex min-h-0 flex-1 flex-col bg-surface-container-low">
              <MarkdownEditor
                key={fileId ?? file.id}
                defaultValue={draft}
                onChange={setDraft}
                onImagePaste={handleEditorImagePaste}
              />
            </div>
          )}

          {isPdf && pdfSourceUrl && pdfDocument && (
            <div class="relative min-h-[34rem] flex-1 bg-white">
              {!pdfLoaded && (
                <div class="absolute inset-0 z-10 bg-surface-container-low">
                  <FilePreviewSkeleton />
                </div>
              )}
              <iframe
                key={pdfSourceUrl}
                srcDoc={pdfDocument}
                title={file?.name ?? "PDF"}
                class="block h-full w-full border-0 bg-white"
              />
            </div>
          )}

          {imageSourceUrl && !imageError && (
            <div class="relative flex min-h-[28rem] flex-1 items-center justify-center bg-surface-container-low p-5">
              {!imageLoaded && (
                <div class="absolute inset-0 z-10 bg-surface-container-low">
                  <FilePreviewSkeleton />
                </div>
              )}
              <button
                type="button"
                class="border-0 bg-transparent p-0 disabled:cursor-default"
                disabled={!imageLoaded}
                aria-label={t("file.preview.expandImage")}
                onClick={() => setImageExpanded(true)}
              >
                <img
                  key={imageSourceUrl}
                  src={imageSourceUrl}
                  alt={file?.name ?? ""}
                  class="max-h-[70vh] max-w-full cursor-zoom-in rounded-lg object-contain"
                  onLoad={() => setImageLoaded(true)}
                  onError={() => setImageError(true)}
                />
              </button>
            </div>
          )}

          {content?.kind === "url" && !isPdf && (!isImage || imageError) && (
            <div class="flex flex-1 flex-col items-start gap-4 p-5">
              <p class="text-sm text-text-secondary">
                {t("file.binary.notice")}
              </p>
              <a
                href={content.url}
                target="_blank"
                rel="noopener noreferrer"
                class="btn-primary no-underline"
              >
                <span class="material-symbols-outlined text-[20px]">
                  open_in_new
                </span>
                {t("file.binary.open")}
              </a>
            </div>
          )}
        </div>
      </section>

      {imageExpanded && imageSourceUrl && !imageError && (
        <ImageLightbox
          src={imageSourceUrl}
          alt={file?.name ?? ""}
          title={file?.name}
          onClose={() => setImageExpanded(false)}
        />
      )}
    </>
  );
}
