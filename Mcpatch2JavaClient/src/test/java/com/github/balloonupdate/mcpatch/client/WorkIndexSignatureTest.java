package com.github.balloonupdate.mcpatch.client;

import com.github.balloonupdate.mcpatch.client.config.AppConfig;
import com.github.balloonupdate.mcpatch.client.data.IndexFile;
import com.github.balloonupdate.mcpatch.client.data.VersionIndex;
import com.github.balloonupdate.mcpatch.client.exceptions.McpatchBusinessException;
import com.github.balloonupdate.mcpatch.client.utils.IndexSignatureVerifierTest;
import org.json.JSONObject;
import org.junit.jupiter.api.Test;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

class WorkIndexSignatureTest {
    @Test
    void acceptsSignedIndexWhenSignatureMatches() throws Exception {
        AppConfig config = createConfig();
        VersionIndex version = createSignedVersion("1.0.0", "abc123");

        assertDoesNotThrow(() -> invokeIndexVerifier(version, config));
    }

    @Test
    void rejectsSignedIndexWhenSignatureDoesNotMatch() throws Exception {
        AppConfig config = createConfig();
        VersionIndex version = createSignedVersion("1.0.0", "abc123");
        version.hash = "deadbeef";

        McpatchBusinessException error = assertThrows(McpatchBusinessException.class, () -> invokeIndexVerifier(version, config));
        assertTrue(error.getMessage().contains("索引签名"));
    }

    @Test
    void acceptsLegacyUnsignedIndexForCompatibility() {
        AppConfig config = createConfig();
        VersionIndex version = IndexSignatureVerifierTest.createVersion("1.0.0", "abc123");

        assertDoesNotThrow(() -> invokeIndexVerifier(version, config));
    }

    @Test
    void verifiesAllVersionsInIndexFile() throws Exception {
        AppConfig config = createConfig();
        VersionIndex version = createSignedVersion("1.0.0", "abc123");
        IndexFile indexFile = new IndexFile();
        indexFile.versions.add(version);

        assertDoesNotThrow(() -> invokeIndexFileVerifier(indexFile, config));
    }

    @Test
    void rejectsSignedIndexWhenPublicKeyIsMissing() throws Exception {
        AppConfig config = createConfigWithoutPublicKey();
        VersionIndex version = createSignedVersion("1.0.0", "abc123");

        McpatchBusinessException error = assertThrows(McpatchBusinessException.class, () -> invokeIndexVerifier(version, config));
        assertTrue(error.getMessage().contains("索引签名"));
    }

    private static AppConfig createConfig() {
        Map<String, Object> map = new HashMap<>();
        map.put("urls", java.util.List.of("http://127.0.0.1"));
        map.put("index-signature-public-key", IndexSignatureVerifierTest.TEST_PUBLIC_KEY_BASE64);
        return new AppConfig(map);
    }

    private static AppConfig createConfigWithoutPublicKey() {
        Map<String, Object> map = new HashMap<>();
        map.put("urls", java.util.List.of("http://127.0.0.1"));
        return new AppConfig(map);
    }

    private static VersionIndex createSignedVersion(String label, String hash) throws Exception {
        VersionIndex version = IndexSignatureVerifierTest.createVersion(label, hash);
        version.signature = IndexSignatureVerifierTest.sign(version);
        return version;
    }

    private static void invokeIndexVerifier(VersionIndex version, AppConfig config) throws Exception {
        Method method;

        try {
            method = Work.class.getDeclaredMethod("verifyVersionIndexSignature", VersionIndex.class, AppConfig.class);
            method.setAccessible(true);
        } catch (NoSuchMethodException e) {
            fail("Work 缺少版本索引签名校验入口 verifyVersionIndexSignature");
            return;
        }

        try {
            method.invoke(null, version, config);
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof Exception exception) {
                throw exception;
            }
            throw e;
        }
    }

    private static void invokeIndexFileVerifier(IndexFile indexFile, AppConfig config) throws Exception {
        Method method;

        try {
            method = Work.class.getDeclaredMethod("verifyIndexSignatures", IndexFile.class, AppConfig.class);
            method.setAccessible(true);
        } catch (NoSuchMethodException e) {
            fail("Work 缺少索引列表签名校验入口 verifyIndexSignatures");
            return;
        }

        try {
            method.invoke(null, indexFile, config);
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof Exception exception) {
                throw exception;
            }
            throw e;
        }
    }
}
