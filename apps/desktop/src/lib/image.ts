import { PNG_MIME } from "../constants/board";

/** The image as PNG bytes; other formats are redrawn on a canvas so every shared image is a PNG. */
export async function toPng(image: Blob): Promise<Uint8Array> {
  if (image.type === PNG_MIME) return new Uint8Array(await image.arrayBuffer());
  const bitmap = await createImageBitmap(image);
  const canvas = document.createElement("canvas");
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  canvas.getContext("2d")?.drawImage(bitmap, 0, 0);
  bitmap.close();
  const png = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, PNG_MIME));
  if (!png) throw new Error(image.type);
  return new Uint8Array(await png.arrayBuffer());
}

/** The first image in a paste, if any. */
export function pastedImage(data: DataTransfer | null): File | null {
  const item = Array.from(data?.items ?? []).find((entry) => entry.kind === "file" && entry.type.startsWith("image/"));
  return item?.getAsFile() ?? null;
}
