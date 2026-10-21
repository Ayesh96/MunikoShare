// Single source of every design value; components only use the CSS variables built from it.
export const colors = {
  light: {
    bg: "#f5f6fa",
    surface: "#ffffff",
    border: "#e2e4ec",
    text: "#1b1d29",
    textMuted: "#6b6f82",
    primary: "#4f46e5",
    primaryText: "#ffffff",
    primaryHover: "#4338ca",
    success: "#16a34a",
    danger: "#dc2626",
    track: "#e2e4ec",
  },
  dark: {
    bg: "#12131a",
    surface: "#1c1e29",
    border: "#2c2f3f",
    text: "#eceef7",
    textMuted: "#9a9eb3",
    primary: "#818cf8",
    primaryText: "#12131a",
    primaryHover: "#a5b4fc",
    success: "#4ade80",
    danger: "#f87171",
    track: "#2c2f3f",
  },
} as const;

export const space = { xs: "4px", sm: "8px", md: "12px", lg: "16px", xl: "24px" } as const;
export const radius = { sm: "8px", md: "14px", pill: "999px" } as const;
export const fontSize = { xs: "12px", sm: "14px", md: "16px", lg: "18px", xl: "22px" } as const;
export const border = { hairline: "1px" } as const;
export const shadow = { card: "0 1px 3px rgba(0, 0, 0, 0.12)" } as const;
export const size = { touch: "48px", progressHeight: "8px", pageWidth: "640px", textAreaHeight: "120px" } as const;
export const fontFamily = "system-ui, -apple-system, 'Segoe UI', sans-serif";
