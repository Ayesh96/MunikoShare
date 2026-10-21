export const SETTINGS_COMMANDS = {
  get: "get_settings",
  setDeviceName: "set_device_name",
  setDownloadDir: "set_download_dir",
  openDownloadDir: "open_download_dir",
  takeLaunchPaths: "take_launch_paths",
} as const;

export const DEVICE_NAME_MAX_LENGTH = 64;
