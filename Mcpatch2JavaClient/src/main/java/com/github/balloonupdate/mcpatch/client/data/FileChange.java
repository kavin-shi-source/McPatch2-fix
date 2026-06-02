package com.github.balloonupdate.mcpatch.client.data;

/**
 * 代表单个文件操作
 */
public interface FileChange {
    /**
     * 创建一个目录
     */
    class CreateFolder implements FileChange {
        /**
         * 要创建目录的路径
         */
        public String path;

        @Override
        public String toString() {
            return "create folder: " + path;
        }
    }

    /**
     * 新增新的文件或者更新现有文件
     */
    class UpdateFile implements FileChange {
        /**
         * 要更新的文件路径
         */
        public String path;

        /**
         * 文件校验值
         */
        public String hash;

        /**
         * 文件长度
         */
        public long len;

        /**
         * 文件的修改时间
         */
        public long modified;

        /**
         * 文件二进制数据在更新包中的偏移值
         */
        public long offset;

        /**
         * 文件来源标识（可选），例如 "cdn" 或 "server"。为空时表示从更新包中提取
         */
        public String source;

        /**
         * 文件独立下载地址（可选）。不为空时客户端直接从此 URL 下载，不走 Range 分片
         */
        public String downloadUrl;

        @Override
        public String toString() {
            return "update file: " + path + " (" + len + ")";
        }
    }

    /**
     * 删除一个目录
     */
    class DeleteFolder implements FileChange {
        /**
         * 要删除的目录的路径
         */
        public String path;

        @Override
        public String toString() {
            return "delete folder: " + path;
        }
    }

    /**
     * 删除一个文件
     */
    class DeleteFile implements FileChange {
        /**
         * 要删除的文件的路径
         */
        public String path;

        @Override
        public String toString() {
            return "delete file: " + path;
        }
    }

    /**
     * 移动一个文件
     */
    class MoveFile implements FileChange {
        /**
         * 文件从哪里来
         */
        public String from;

        /**
         * 文件到哪里去
         */
        public String to;

        @Override
        public String toString() {
            return "move: " + from + " => " + to;
        }
    }
}
