/** Copies text with the webview clipboard. */
export function copyText(text: string): Promise<void> {
  return navigator.clipboard.writeText(text);
}
