import { border, colors, fontFamily, fontFamilyDisplay, fontFamilyMono, fontSize, fontWeight, lineHeight, motion, radius, shadow, shadowModal, size, space, zIndex } from "./tokens";

const STYLE_ELEMENT_ID = "muniko-theme";

type TokenGroup = Readonly<Record<string, string>>;

function toKebab(name: string): string {
  return name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}

function toVariables(prefix: string, group: TokenGroup): string {
  return Object.entries(group)
    .map(([name, value]) => `--${prefix}-${toKebab(name)}: ${value};`)
    .join("\n");
}

function buildCss(): string {
  const shared = [
    toVariables("space", space),
    toVariables("radius", radius),
    toVariables("font-size", fontSize),
    toVariables("border", border),
    toVariables("line-height", lineHeight),
    toVariables("font-weight", fontWeight),
    toVariables("motion", motion),
    toVariables("shadow", shadow),
    toVariables("shadow", shadowModal),
    toVariables("size", size),
    toVariables("z", zIndex),
    `--font-family-mono: ${fontFamilyMono};`,
    `--font-family: ${fontFamily};`,
    `--font-family-display: ${fontFamilyDisplay};`,
  ].join("\n");
  return `:root {\n${shared}\n${toVariables("color", colors.light)}\ncolor-scheme: light dark;\n}\n` +
    `@media (prefers-color-scheme: dark) {\n:root {\n${toVariables("color", colors.dark)}\n}\n}\n` +
    `body { margin: 0; background: var(--color-bg); color: var(--color-text); font-family: var(--font-family); font-size: var(--font-size-body); line-height: var(--line-height-body); }\n` +
    `h1, h2, h3 { font-family: var(--font-family-display); font-weight: var(--font-weight-bold); }\n` +
    `:focus-visible { outline: var(--border-focus) solid var(--color-primary); outline-offset: 2px; }\n` +
    `@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; } }\n`;
}

/** Injects the theme CSS variables once; safe to call again. */
export function installTheme(): void {
  if (document.getElementById(STYLE_ELEMENT_ID)) return;
  const style = document.createElement("style");
  style.id = STYLE_ELEMENT_ID;
  style.textContent = buildCss();
  document.head.appendChild(style);
}
