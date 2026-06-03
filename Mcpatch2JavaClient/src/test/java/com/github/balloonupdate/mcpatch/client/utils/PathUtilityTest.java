package com.github.balloonupdate.mcpatch.client.utils;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class PathUtilityTest {
    @TempDir
    Path tempDir;

    @Test
    void resolvesNormalPathInsideBaseDirectory() throws IOException {
        Path baseDir = tempDir.resolve("base");
        Path resolved = PathUtility.resolveUnderBase(baseDir, "mods/example.jar");

        assertEquals(baseDir.resolve("mods/example.jar").toAbsolutePath().normalize(), resolved);
    }

    @Test
    void rejectsParentTraversalOutsideBaseDirectory() {
        Path baseDir = tempDir.resolve("base");

        assertThrows(IOException.class, () -> PathUtility.resolveUnderBase(baseDir, "../escape.txt"));
    }

    @Test
    void rejectsAbsolutePath() {
        Path baseDir = tempDir.resolve("base");
        Path absolute = tempDir.resolve("escape.txt").toAbsolutePath();

        assertThrows(IOException.class, () -> PathUtility.resolveUnderBase(baseDir, absolute.toString()));
    }
}
