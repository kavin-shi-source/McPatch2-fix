import instance from "@/utils/request.js";

export const modPlatformStatusRequest = () => instance.post('/modplatform/status', {})

export const modPlatformSearchRequest = (query, platform, game_version, mod_loader) =>
  instance.post('/modplatform/search', {query, platform, game_version, mod_loader})

export const modPlatformVersionsRequest = (platform, mod_id, game_version, mod_loader) =>
  instance.post('/modplatform/versions', {platform, mod_id, game_version, mod_loader})

export const modPlatformInstallRequest = (platform, mod_name, download_url, filename) =>
  instance.post('/modplatform/install', {platform, mod_name, download_url, filename})

export const modPlatformUpdateConfigRequest = (config) =>
  instance.post('/modplatform/update-config', config)
