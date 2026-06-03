package com.github.balloonupdate.mcpatch.client.utils;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Arrays;
import java.util.Locale;
import java.util.regex.Pattern;

/**
 * 文件 hash 计算类，所有计算文件哈希值时都会调用此函数，可以在此函数中替换任意哈希算法
 */
public class HashUtility {
    private static final Pattern LEGACY_HASH_PATTERN = Pattern.compile("^[0-9a-fA-F]{16}_[0-9a-fA-F]{4}$");

    /**
     * 计算一个文件的校验值，默认使用 SHA-256
     */
    public static String calculateHash(Path file) throws IOException {
        try (InputStream stream = new BufferedInputStream(Files.newInputStream(file))) {
            return calculateSha256(stream);
        }
    }

    /**
     * 计算一段字节流的校验值，默认使用 SHA-256
     */
    public static String calculateHash(byte[] data) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            return toHex(digest.digest(data));
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("当前环境不支持 SHA-256", e);
        }
    }

    public static boolean matchesHash(Path file, String expectedHash) throws IOException {
        return calculateHashForExpected(file, expectedHash).equals(normalizeHash(expectedHash));
    }

    public static boolean matchesHash(byte[] data, String expectedHash) {
        return calculateHashForExpected(data, expectedHash).equals(normalizeHash(expectedHash));
    }

    public static String calculateHashForExpected(Path file, String expectedHash) throws IOException {
        if (isLegacyHash(expectedHash)) {
            return calculateLegacyHash(file);
        }
        return calculateHash(file);
    }

    public static String calculateHashForExpected(byte[] data, String expectedHash) {
        if (isLegacyHash(expectedHash)) {
            return calculateLegacyHash(data);
        }
        return calculateHash(data);
    }

    static boolean isLegacyHash(String hash) {
        return hash != null && LEGACY_HASH_PATTERN.matcher(hash).matches();
    }

    static String calculateLegacyHash(Path file) throws IOException {
        Crc64_XZ crc64 = new Crc64_XZ();
        Crc16_IBM_SDLC crc16 = new Crc16_IBM_SDLC();
        crc64.reset();
        crc16.reset();
        crc64.update(file);
        crc16.update(file);
        return formatLegacyHash(crc64.getValue(), crc16.getValue());
    }

    static String calculateLegacyHash(byte[] data) {
        Crc64_XZ crc64 = new Crc64_XZ();
        Crc16_IBM_SDLC crc16 = new Crc16_IBM_SDLC();
        crc64.reset();
        crc16.reset();
        crc64.update(data, 0, data.length);
        crc16.update(data, 0, data.length);
        return formatLegacyHash(crc64.getValue(), crc16.getValue());
    }

    private static String calculateSha256(InputStream stream) throws IOException {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] buffer = new byte[128 * 1024];
            int read;

            while ((read = stream.read(buffer)) != -1) {
                digest.update(buffer, 0, read);
            }

            return toHex(digest.digest());
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("当前环境不支持 SHA-256", e);
        }
    }

    private static String toHex(byte[] hash) {
        StringBuilder sb = new StringBuilder(hash.length * 2);
        for (byte b : hash) {
            sb.append(String.format("%02x", b));
        }
        return sb.toString();
    }

    private static String formatLegacyHash(long crc64Value, long crc16Value) {
        String crc64 = String.format("%016x", crc64Value);
        String crc16 = String.format("%04x", crc16Value);

        return crc64 + "_" + crc16;
    }

    private static String normalizeHash(String hash) {
        return hash.toLowerCase(Locale.ROOT);
    }
}

class Crc64_XZ {
    private final long polynomial = 0x42f0e1eba9ea3693L;
    private final long initialValue = 0xffffffffffffffffL;
    private final long finalXorValue = 0xffffffffffffffffL;
    private final boolean reflectInput = true;
    private final boolean reflectOutput = true;

    private long crc = initialValue;

    byte[] buf = new byte[128 * 1024];

    public void reset() {
        crc = initialValue;

        Arrays.fill(buf, (byte) 0);
    }

    public void update(Path file) throws IOException {
        try (BufferedInputStream stream = new BufferedInputStream(Files.newInputStream(file))) {
            int read;

            while ((read = stream.read(buf)) != -1)
                update(buf, 0, read);
        }
    }

    public void update(byte[] data, int offset, int len) {
        for (int x = offset; x < len; x++) {
            byte b = data[x];
            long value = b & 0xFF;
            if (reflectInput) {
                value = reflect(value, 8);
            }
            crc ^= (value << 56);
            for (int i = 0; i < 8; i++) {
                if ((crc & 0x8000000000000000L) != 0) {
                    crc = (crc << 1) ^ polynomial;
                } else {
                    crc <<= 1;
                }
            }
        }
    }

    public long getValue() {
        long result = crc;
        if (reflectOutput) {
            result = reflect(result, 64);
        }
        return result ^ finalXorValue;
    }

    private long reflect(long value, int bits) {
        long reflected = 0;
        for (int i = 0; i < bits; i++) {
            if ((value & (1L << i)) != 0) {
                reflected |= (1L << (bits - 1 - i));
            }
        }
        return reflected;
    }
}

class Crc16_IBM_SDLC {
    private final int polynomial = 0x1021;
    private final int initialValue = 0xffff;
    private final int finalXorValue = 0xffff;
    private final boolean reflectInput = true;
    private final boolean reflectOutput = true;

    private int crc = initialValue;

    byte[] buf = new byte[128 * 1024];

    public void reset() {
        crc = initialValue;

        Arrays.fill(buf, (byte) 0);
    }

    public void update(Path file) throws IOException {
        try (BufferedInputStream stream = new BufferedInputStream(Files.newInputStream(file))) {
            int read;

            while ((read = stream.read(buf)) != -1)
                update(buf, 0, read);
        }
    }

    public void update(byte[] data, int offset, int len) {
        for (int x = offset; x < len; x++) {
            byte b = data[x];
            int value = b & 0xFF;
            if (reflectInput) {
                value = reflect(value, 8);
            }
            crc ^= (value << 8);
            for (int i = 0; i < 8; i++) {
                if ((crc & 0x8000) != 0) {
                    crc = (crc << 1) ^ polynomial;
                } else {
                    crc <<= 1;
                }
            }
        }
    }

    public int getValue() {
        int result = crc;
        if (reflectOutput) {
            result = reflect(result, 16);
        }
        return (result ^ finalXorValue) & 0xFFFF;
    }

    private int reflect(int value, int bits) {
        int reflected = 0;
        for (int i = 0; i < bits; i++) {
            if ((value & (1 << i)) != 0) {
                reflected |= (1 << (bits - 1 - i));
            }
        }
        return reflected;
    }
}
