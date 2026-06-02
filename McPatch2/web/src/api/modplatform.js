import instance from "@/utils/request.js";

export const modPlatformStatusRequest = () => instance.post('/modplatform/status', {})

export const modPlatformSearchRequest = (query, platform, game_version, mod_loader) =>
  instance.post('/modplatform/search', {query, platform, game_version, mod_loader})

export const modPlatformUpdateConfigRequest = (config) =>
  instance.post('/modplatform/update-config', config)
