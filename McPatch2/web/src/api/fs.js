import instance from "@/utils/request.js";

export const fsDiskInfoRequest = () => instance.post('/fs/disk-info', {})

export const fsListRequest = (path = '') => instance.post('/fs/list', {path})

export const fsMakeDirectoryRequest = (path = '') => instance.post('/fs/make-directory', {path})

export const fsDeleteRequest = (path = '') => instance.post('/fs/delete', {path})

export const fsSignFileRequest = (path = '') => instance.post('/fs/sign-file', {path})

const readFileAsBase64 = (file) => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = () => {
      if (typeof reader.result !== 'string') {
        reject(new Error('文件编码失败'));
        return;
      }

      const [, base64 = ''] = reader.result.split(',');
      resolve(base64);
    };

    reader.onerror = () => {
      reject(reader.error || new Error('文件读取失败'));
    };

    reader.readAsDataURL(file);
  });
}

export const fsUploadRequest = async (path = '', file, onProgress = () => {}) => {
  onProgress({percent: 10});
  const content = await readFileAsBase64(file);
  onProgress({percent: 80});

  const response = await instance.post('/fs/upload', {path, content});
  onProgress({percent: 100});

  return response;
}

export const fsDownloadRequest = (path = '') => instance.post('/fs/download', {path})
