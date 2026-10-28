const keys = new WeakMap<Blob, number>();
let next = 0;

/** A stable React key for a blob, so removing one item does not remount the others. */
export function blobKey(blob: Blob): number {
  let key = keys.get(blob);
  if (key === undefined) {
    key = next++;
    keys.set(blob, key);
  }
  return key;
}
