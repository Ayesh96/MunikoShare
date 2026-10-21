export type Tone = "info" | "success" | "danger";

export interface Notice {
  message: string;
  tone: Tone;
}
