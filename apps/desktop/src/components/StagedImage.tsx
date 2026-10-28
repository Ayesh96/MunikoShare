import { CLIPBOARD_REMOVE_IMAGE_ICON } from "../constants/icons";
import { useObjectUrl } from "../hooks/useObjectUrl";
import { CLIPBOARD_SHARE_IMAGE_ALT, CLIPBOARD_SHARE_REMOVE_IMAGE_LABEL } from "../messages/ui";
import styles from "./StagedImage.module.css";
import { Button } from "./ui";

export interface StagedImageProps {
  image: Blob;
  onRemove: () => void;
}

/** One square draft thumbnail with its own remove button pinned to the corner. */
export function StagedImage({ image, onRemove }: StagedImageProps) {
  const url = useObjectUrl(image);

  return (
    <div className={styles.tile}>
      {url && <img className={styles.image} src={url} alt={CLIPBOARD_SHARE_IMAGE_ALT} />}
      <div className={styles.remove}>
        <Button variant="tertiary" size="sm" iconOnly icon={CLIPBOARD_REMOVE_IMAGE_ICON} label={CLIPBOARD_SHARE_REMOVE_IMAGE_LABEL} onClick={onRemove} />
      </div>
    </div>
  );
}
