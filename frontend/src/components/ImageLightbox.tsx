import { useEffect } from "preact/hooks";
import { useTranslation } from "react-i18next";

interface ImageLightboxProps {
  src: string;
  alt?: string;
  title?: string;
  onClose: () => void;
}

export function ImageLightbox({
  src,
  alt = "",
  title,
  onClose,
}: ImageLightboxProps) {
  const { t } = useTranslation();
  const visibleTitle = title?.trim() || alt.trim();
  const label = visibleTitle || t("file.preview.image");

  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleKey);
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", handleKey);
      document.body.style.overflow = previousOverflow;
    };
  }, [onClose]);

  return (
    <div
      class="image-lightbox"
      role="dialog"
      aria-modal="true"
      aria-label={label}
      onClick={onClose}
    >
      <div class="image-lightbox-stage">
        <img
          src={src}
          alt={alt}
          class="image-lightbox-image"
          onClick={(event) => event.stopPropagation()}
        />
      </div>
    </div>
  );
}
