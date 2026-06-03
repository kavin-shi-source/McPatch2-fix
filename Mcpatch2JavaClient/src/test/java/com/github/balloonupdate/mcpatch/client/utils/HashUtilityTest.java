package com.github.balloonupdate.mcpatch.client.utils;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.security.MessageDigest;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class HashUtilityTest {
    @TempDir
    Path tempDir;

    @Test
    void calculateHashUsesSha256ForFilesAndBytes() throws Exception {
        byte[] data = "[{\"label\":\"1.0.0\",\"changes\":[]}]".getBytes(StandardCharsets.UTF_8);
        Path tempFile = tempDir.resolve("metadata.json");
        Files.write(tempFile, data);

        String expected = sha256Hex(data);

        assertEquals(expected, HashUtility.calculateHash(tempFile));
        assertEquals(expected, HashUtility.calculateHash(data));
    }

    @Test
    void matchesHashSupportsSha256AndLegacyCrcFormats() throws Exception {
        byte[] data = "[{\"label\":\"1.0.0\",\"changes\":[]}]".getBytes(StandardCharsets.UTF_8);
        Path tempFile = tempDir.resolve("legacy-metadata.json");
        Files.write(tempFile, data);

        String sha256 = sha256Hex(data);
        String legacy = legacyHash(data);

        assertTrue(HashUtility.matchesHash(tempFile, sha256));
        assertTrue(HashUtility.matchesHash(data, sha256));
        assertTrue(HashUtility.matchesHash(tempFile, legacy));
        assertTrue(HashUtility.matchesHash(data, legacy));
        assertFalse(HashUtility.matchesHash(tempFile, "deadbeef"));
    }

    private String sha256Hex(byte[] data) throws Exception {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        byte[] hash = digest.digest(data);
        StringBuilder sb = new StringBuilder();

        for (byte b : hash) {
            sb.append(String.format("%02x", b));
        }

        return sb.toString();
    }

    private String legacyHash(byte[] data) {
        Crc64_XZ crc64 = new Crc64_XZ();
        Crc16_IBM_SDLC crc16 = new Crc16_IBM_SDLC();
        crc64.reset();
        crc16.reset();
        crc64.update(data, 0, data.length);
        crc16.update(data, 0, data.length);

        return String.format("%016x_%04x", crc64.getValue(), crc16.getValue());
    }
}
