package com.github.balloonupdate.mcpatch.client.utils;

import com.github.balloonupdate.mcpatch.client.data.VersionIndex;
import org.json.JSONObject;
import org.junit.jupiter.api.Test;

import java.nio.charset.StandardCharsets;
import java.security.KeyFactory;
import java.security.PrivateKey;
import java.security.Signature;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Base64;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class IndexSignatureVerifierTest {
    public static final String TEST_PRIVATE_KEY_BASE64 =
            "MC4CAQAwBQYDK2VwBCIEIA8kef4Ht946ZEJtY6I6noyxVvPBxnfLjWCMORBSwDY6";
    public static final String TEST_PUBLIC_KEY_BASE64 =
            "MCowBQYDK2VwAyEAUQ7CMo6Lrj/7/xFx30E/W77LUBK0cJT2wGdIAz4UR84=";

    @Test
    void verifyPassesWhenSignatureMatches() throws Exception {
        VersionIndex version = createVersion("1.0.0", "abc123");
        version.signature = sign(version);

        assertTrue(IndexSignatureVerifier.verify(version, TEST_PUBLIC_KEY_BASE64));
    }

    @Test
    void verifyFailsWhenVersionIndexIsTampered() throws Exception {
        VersionIndex version = createVersion("1.0.0", "abc123");
        version.signature = sign(version);
        version.hash = "deadbeef";

        assertFalse(IndexSignatureVerifier.verify(version, TEST_PUBLIC_KEY_BASE64));
    }

    @Test
    void missingSignatureCanBeDetectedForLegacyCompatibility() {
        VersionIndex version = createVersion("1.0.0", "abc123");

        assertFalse(IndexSignatureVerifier.hasSignature(version));
    }

    @Test
    void emptyConfiguredKeyIsRejectedForSignedIndex() throws Exception {
        VersionIndex version = createVersion("1.0.0", "abc123");
        version.signature = sign(version);

        assertThrows(IllegalStateException.class, () -> IndexSignatureVerifier.verify(version, ""));
    }

    @Test
    void emptyConfiguredKeyIsRecognizedAsMissing() {
        assertTrue(IndexSignatureVerifier.isConfiguredPublicKeyMissing(""));
    }

    public static String sign(VersionIndex version) throws Exception {
        byte[] privateKeyBytes = Base64.getDecoder().decode(TEST_PRIVATE_KEY_BASE64);
        PrivateKey privateKey = KeyFactory.getInstance("Ed25519")
                .generatePrivate(new PKCS8EncodedKeySpec(privateKeyBytes));
        Signature signer = Signature.getInstance("Ed25519");
        signer.initSign(privateKey);
        signer.update(IndexSignatureVerifier.canonicalString(version).getBytes(StandardCharsets.UTF_8));
        return Base64.getEncoder().encodeToString(signer.sign());
    }

    public static VersionIndex createVersion(String label, String hash) {
        JSONObject json = new JSONObject();
        json.put("label", label);
        json.put("filename", label + ".tar");
        json.put("offset", 123);
        json.put("length", 456);
        json.put("hash", hash);
        return new VersionIndex(json);
    }
}
