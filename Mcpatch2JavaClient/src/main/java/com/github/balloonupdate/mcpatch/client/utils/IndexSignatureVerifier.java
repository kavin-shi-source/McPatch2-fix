package com.github.balloonupdate.mcpatch.client.utils;

import com.github.balloonupdate.mcpatch.client.data.VersionIndex;

import java.nio.charset.StandardCharsets;
import java.security.KeyFactory;
import java.security.PublicKey;
import java.security.Signature;
import java.security.spec.X509EncodedKeySpec;
import java.util.Base64;

public final class IndexSignatureVerifier {
    private IndexSignatureVerifier() {
    }

    public static String canonicalString(VersionIndex version) {
        return "label=" + version.label + "\n" +
                "filename=" + version.filename + "\n" +
                "offset=" + version.offset + "\n" +
                "length=" + version.len + "\n" +
                "hash=" + version.hash;
    }

    public static boolean hasSignature(VersionIndex version) {
        return version.signature != null && !version.signature.isBlank();
    }

    public static boolean isConfiguredPublicKeyMissing(String configuredPublicKeyBase64) {
        return configuredPublicKeyBase64 == null || configuredPublicKeyBase64.isBlank();
    }

    public static boolean verify(VersionIndex version, String configuredPublicKeyBase64) {
        if (!hasSignature(version)) {
            return false;
        }

        String publicKeyBase64 = resolvePublicKey(configuredPublicKeyBase64);
        if (publicKeyBase64.isBlank()) {
            throw new IllegalStateException("未配置更新索引验签公钥");
        }

        try {
            byte[] publicKeyBytes = Base64.getDecoder().decode(publicKeyBase64);
            PublicKey publicKey = KeyFactory.getInstance("Ed25519")
                    .generatePublic(new X509EncodedKeySpec(publicKeyBytes));
            Signature verifier = Signature.getInstance("Ed25519");
            verifier.initVerify(publicKey);
            verifier.update(canonicalString(version).getBytes(StandardCharsets.UTF_8));

            byte[] signatureBytes = Base64.getDecoder().decode(version.signature);
            return verifier.verify(signatureBytes);
        } catch (Exception e) {
            throw new IllegalStateException("更新索引签名校验失败", e);
        }
    }

    static String resolvePublicKey(String configuredPublicKeyBase64) {
        if (configuredPublicKeyBase64 == null) {
            return "";
        }
        return configuredPublicKeyBase64.trim();
    }
}
