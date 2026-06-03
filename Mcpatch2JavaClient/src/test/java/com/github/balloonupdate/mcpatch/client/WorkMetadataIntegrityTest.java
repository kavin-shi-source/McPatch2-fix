package com.github.balloonupdate.mcpatch.client;

import com.github.balloonupdate.mcpatch.client.data.VersionIndex;
import com.github.balloonupdate.mcpatch.client.exceptions.McpatchBusinessException;
import com.github.balloonupdate.mcpatch.client.utils.HashUtility;
import org.json.JSONObject;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

class WorkMetadataIntegrityTest {
    @TempDir
    Path tempDir;

    @Test
    void acceptsMetadataWhenHashMatchesVersionIndex() throws Exception {
        String metadataText = "[{\"label\":\"1.0.0\",\"changes\":[]}]";
        VersionIndex version = createVersionIndex("1.0.0", metadataText);

        assertDoesNotThrow(() -> invokeMetadataVerifier(version, metadataText));
    }

    @Test
    void rejectsMetadataWhenHashDoesNotMatchVersionIndex() throws Exception {
        String metadataText = "[{\"label\":\"1.0.0\",\"changes\":[]}]";
        VersionIndex version = createVersionIndex("1.0.0", metadataText);
        version.hash = "deadbeef_dead";

        McpatchBusinessException error = assertThrows(McpatchBusinessException.class, () -> invokeMetadataVerifier(version, metadataText));

        assertTrue(error.getMessage().contains("1.0.0"));
        assertTrue(error.getMessage().contains("校验"));
    }

    @Test
    void acceptsLegacyMetadataWhenServerDidNotProvideHash() throws Exception {
        String metadataText = "[{\"label\":\"1.0.0\",\"changes\":[]}]";
        VersionIndex version = createVersionIndex("1.0.0", metadataText);
        version.hash = "no hash";

        assertDoesNotThrow(() -> invokeMetadataVerifier(version, metadataText));
    }

    @Test
    void acceptsLegacyMetadataWhenVersionIndexUsesOldCrcHash() throws Exception {
        String metadataText = "[{\"label\":\"1.0.0\",\"changes\":[]}]";
        VersionIndex version = createVersionIndex("1.0.0", metadataText);
        version.hash = legacyHash(metadataText.getBytes(StandardCharsets.UTF_8));

        assertDoesNotThrow(() -> invokeMetadataVerifier(version, metadataText));
    }

    private VersionIndex createVersionIndex(String label, String metadataText) throws Exception {
        Path tempFile = tempDir.resolve(label + ".json");
        Files.writeString(tempFile, metadataText, StandardCharsets.UTF_8);

        JSONObject json = new JSONObject();
        json.put("label", label);
        json.put("filename", label + ".tar");
        json.put("offset", 0);
        json.put("length", metadataText.getBytes(StandardCharsets.UTF_8).length);
        json.put("hash", HashUtility.calculateHash(tempFile));
        return new VersionIndex(json);
    }

    private void invokeMetadataVerifier(VersionIndex version, String metadataText) throws Exception {
        Method method;

        try {
            method = Work.class.getDeclaredMethod("verifyMetadataHash", VersionIndex.class, String.class);
            method.setAccessible(true);
        } catch (NoSuchMethodException e) {
            fail("Work 缺少版本元数据完整性校验入口 verifyMetadataHash");
            return;
        }

        try {
            method.invoke(null, version, metadataText);
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof Exception exception) {
                throw exception;
            }
            throw e;
        }
    }

    private String legacyHash(byte[] data) {
        LegacyCrc64 crc64 = new LegacyCrc64();
        LegacyCrc16 crc16 = new LegacyCrc16();
        crc64.reset();
        crc16.reset();
        crc64.update(data, 0, data.length);
        crc16.update(data, 0, data.length);

        return String.format("%016x_%04x", crc64.getValue(), crc16.getValue());
    }

    private static final class LegacyCrc64 {
        private static final long POLYNOMIAL = 0x42f0e1eba9ea3693L;
        private static final long INITIAL_VALUE = 0xffffffffffffffffL;
        private static final long FINAL_XOR_VALUE = 0xffffffffffffffffL;
        private long crc = INITIAL_VALUE;
        private final byte[] buf = new byte[128 * 1024];

        void reset() {
            crc = INITIAL_VALUE;
            Arrays.fill(buf, (byte) 0);
        }

        void update(byte[] data, int offset, int len) {
            for (int index = offset; index < len; index++) {
                long value = reflect(data[index] & 0xFFL, 8);
                crc ^= (value << 56);
                for (int i = 0; i < 8; i++) {
                    if ((crc & 0x8000000000000000L) != 0) {
                        crc = (crc << 1) ^ POLYNOMIAL;
                    } else {
                        crc <<= 1;
                    }
                }
            }
        }

        long getValue() {
            return reflect(crc, 64) ^ FINAL_XOR_VALUE;
        }
    }

    private static final class LegacyCrc16 {
        private static final int POLYNOMIAL = 0x1021;
        private static final int INITIAL_VALUE = 0xffff;
        private static final int FINAL_XOR_VALUE = 0xffff;
        private int crc = INITIAL_VALUE;
        private final byte[] buf = new byte[128 * 1024];

        void reset() {
            crc = INITIAL_VALUE;
            Arrays.fill(buf, (byte) 0);
        }

        void update(byte[] data, int offset, int len) {
            for (int index = offset; index < len; index++) {
                int value = (int) reflect(data[index] & 0xFFL, 8);
                crc ^= (value << 8);
                for (int i = 0; i < 8; i++) {
                    if ((crc & 0x8000) != 0) {
                        crc = (crc << 1) ^ POLYNOMIAL;
                    } else {
                        crc <<= 1;
                    }
                }
                crc &= 0xffff;
            }
        }

        long getValue() {
            return reflect(crc, 16) ^ FINAL_XOR_VALUE;
        }
    }

    private static long reflect(long value, int bits) {
        long reflected = 0;
        for (int i = 0; i < bits; i++) {
            if ((value & (1L << i)) != 0) {
                reflected |= (1L << (bits - 1 - i));
            }
        }
        return reflected;
    }
}
