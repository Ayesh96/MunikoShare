// Single source of every design value; components only use the CSS variables built from it.
export const colors = {
  light: {
    bg: "#f6f3ec",
    surface: "#ffffff",
    border: "#ddd8cb",
    text: "#1e2420",
    textMuted: "#566057",
    primary: "#0f6b5c",
    primaryText: "#ffffff",
    primaryHover: "#0b574a",
    accent: "#9a4513",
    success: "#0f6b5c",
    danger: "#b3261e",
    dangerHover: "#8f1e18",
    dangerText: "#ffffff",
    badgeBg: "#ebe7da",
    overlay: "rgba(22, 26, 23, 0.5)",
    track: "#ddd8cb",
  },
  dark: {
    bg: "#161a17",
    surface: "#202521",
    border: "#323932",
    text: "#edeae0",
    textMuted: "#a3aaa0",
    primary: "#5cc4ae",
    primaryText: "#101412",
    primaryHover: "#7bd3c0",
    accent: "#e8a06a",
    success: "#5cc4ae",
    danger: "#f2877f",
    dangerHover: "#f6a39d",
    dangerText: "#101412",
    badgeBg: "#2a312b",
    overlay: "rgba(0, 0, 0, 0.6)",
    track: "#323932",
  },
} as const;

export const qr = { dark: "#000000", light: "#ffffff" } as const;

export const space = { xs: "4px", sm: "8px", md: "16px", lg: "24px", xl: "32px", "2xl": "48px" } as const;
export const radius = { sm: "6px", md: "10px", pill: "999px" } as const;
export const fontSize = { caption: "13px", body: "16px", title: "19px", heading: "23px", display: "28px" } as const;
export const lineHeight = { caption: "20px", body: "24px", title: "28px", heading: "32px", display: "36px" } as const;
export const fontWeight = { regular: "400", semibold: "600", bold: "700" } as const;
export const border = { hairline: "1px", focus: "2px" } as const;
export const shadow = { card: "0 1px 3px rgba(0, 0, 0, 0.12)" } as const;
export const shadowModal = { modal: "0 8px 32px rgba(0, 0, 0, 0.3)" } as const;
export const motion = { press: "80ms", hover: "120ms", modal: "160ms" } as const;
export const size = { modalWidth: "420px", progressHeight: "8px", codeWidth: "11ch", switchWidth: "36px", switchHeight: "20px", switchThumb: "16px", thumbnail: "64px", iconSize: "20px", skeletonRow: "56px", minTarget: "32px", touchTarget: "44px" } as const;
export const zIndex = { modal: "100" } as const;
export const fontFamilyMono = "'JetBrains Mono', Menlo, Consolas, monospace";
export const fontFamily = "'Source Sans 3', 'Segoe UI', 'Helvetica Neue', sans-serif";
export const fontFamilyDisplay = "'Bricolage Grotesque', 'Source Sans 3', sans-serif";
