package com.github.balloonupdate.mcpatch.client.utils;

import java.io.IOException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

/**
 * 文件路径相关实用类
 */
public class PathUtility {
    /**
     * 获取文件名部分
     */
    public static String getFilename(String url) {
        String filename = url;

        if (url.contains("/"))
            filename = filename.substring(url.lastIndexOf("/") + 1);

        return filename;
    }

    /**
     * 将服务端提供的相对路径安全地解析到 baseDir 之下。
     */
    public static Path resolveUnderBase(Path baseDir, String rawPath) throws IOException {
        if (rawPath == null || rawPath.isEmpty()) {
            throw new IOException("path is empty");
        }

        Path input = Paths.get(rawPath);
        if (input.isAbsolute()) {
            throw new IOException("absolute path is not allowed: " + rawPath);
        }

        for (Path part : input) {
            String name = part.toString();
            if ("..".equals(name)) {
                throw new IOException("path traversal detected: " + rawPath);
            }
        }

        Path normalizedBase = baseDir.toAbsolutePath().normalize();
        Path resolved = normalizedBase.resolve(input).normalize();

        if (!resolved.startsWith(normalizedBase)) {
            throw new IOException("path escapes base directory: " + rawPath);
        }

        return resolved;
    }

    /**
     * 遍历删除文件夹或者普通文件，如果文件不存在不会抛异常
     */
    public static void delete(Path path) throws IOException {
        if (!Files.exists(path))
            return;

        if (Files.isDirectory(path)) {
            try (DirectoryStream<Path> stream = Files.newDirectoryStream(path)) {
                for (Path entry : stream) {
                    // 递归
                    delete(entry);
                }
            }
        }

        // 删除文件或空目录
        Files.delete(path);
    }
}
