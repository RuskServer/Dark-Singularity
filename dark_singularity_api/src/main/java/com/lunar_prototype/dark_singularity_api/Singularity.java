package com.lunar_prototype.dark_singularity_api;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Objects;

/**
 * JNI wrapper for the Rust `dark-singularity` native library.
 * This class corresponds to the JNI functions defined in `src/lib.rs`.
 */
public class Singularity implements AutoCloseable {

    private final long handle;

    static {
        loadNativeLibrary();
    }

    private static void loadNativeLibrary() {
        try {
            String os = System.getProperty("os.name").toLowerCase();
            String arch = System.getProperty("os.arch").toLowerCase();
            
            String osDir;
            String extension;
            String prefix = "lib";

            if (os.contains("win")) {
                osDir = "windows";
                extension = ".dll";
                prefix = "";
            } else if (os.contains("mac")) {
                osDir = "macos";
                extension = ".dylib";
            } else {
                osDir = "linux";
                extension = ".so";
            }

            // アーキテクチャの正規化 (例: x86_64, aarch64)
            String archDir = arch.contains("64") ? (arch.contains("aarch") || arch.contains("arm") ? "aarch64" : "x86_64") : arch;

            String libName = "dark_singularity";
            String nativeFileName = prefix + libName + extension;
            
            // パス構造: /native/{os}/{arch}/{libname}
            String nativeResourcePath = "/native/" + osDir + "/" + archDir + "/" + nativeFileName;
            
            // フォールバック: アーキテクチャ指定なしのパスも試行 (互換性のため)
            InputStream in = Singularity.class.getResourceAsStream(nativeResourcePath);
            if (in == null) {
                nativeResourcePath = "/native/" + osDir + "/" + nativeFileName;
                in = Singularity.class.getResourceAsStream(nativeResourcePath);
            }

            if (in == null) {
                // JAR外（開発環境など）からのロードを試行
                try {
                    System.loadLibrary(libName);
                    return;
                } catch (UnsatisfiedLinkError e) {
                    throw new UnsatisfiedLinkError("Could not find native library " + nativeFileName + " in JAR at " + nativeResourcePath + " or in java.library.path");
                }
            }

            Path tempLib = Files.createTempFile("dark_singularity_", extension);
            tempLib.toFile().deleteOnExit();
            Files.copy(in, tempLib, StandardCopyOption.REPLACE_EXISTING);
            System.load(tempLib.toAbsolutePath().toString());
            
        } catch (IOException e) {
            throw new UnsatisfiedLinkError("Failed to extract native library: " + e.getMessage());
        }
    }

    // --- Native Methods ---
    private native long initNativeSingularity(int stateSize, int[] categorySizes);
    private native void destroyNativeSingularity(long handle);
    private native int selectActionNative(long handle, float[] inputs);
    private native int[] selectActionsNative(long handle, float[] inputs);
    private native void learnNative(long handle, float reward);
    private native float getSystemTemperature(long handle);
    private native float getGliaActivityNative(long handle);
    private native float getActionScoreNative(long handle, int action_idx);
    private native float getFrustration(long handle);
    private native float getAdrenaline(long handle);
    private native float[] getNeuronStates(long handle);
    private native int saveNativeModel(long handle, String path);
    private native int loadNativeModel(long handle, String path);

    /**
     * Initializes a new Singularity instance with dynamic action categories.
     * 
     * @param stateSize Total number of possible environmental states.
     * @param categorySizes Sizes of each action category (e.g., 5 for movement, 3 for combat).
     */
    public Singularity(int stateSize, int... categorySizes) {
        if (categorySizes == null || categorySizes.length == 0) {
            throw new IllegalArgumentException("At least one action category must be defined.");
        }
        this.handle = initNativeSingularity(stateSize, categorySizes);
        if (this.handle == 0) {
            throw new IllegalStateException("Failed to initialize native Singularity instance.");
        }
    }

    // --- Public Java API ---

    /**
     * Selects the best action for the first defined category.
     */
    public int selectAction(float[] inputs) {
        return selectActionNative(handle, inputs);
    }

    /**
     * Selects the best action for EACH defined category.
     * Returns an array where each index corresponds to a category.
     */
    public int[] selectActions(float[] inputs) {
        return selectActionsNative(handle, inputs);
    }
    
    /**
     * Learns from the reward, applying it to the last selected set of actions.
     */
    public void learn(float reward) {
        learnNative(handle, reward);
    }

    public float getSystemTemperature() {
        return getSystemTemperature(handle);
    }
    
    public float getGliaActivity() {
        return getGliaActivityNative(handle);
    }

    public float getActionScore(int actionIndex) {
        return getActionScoreNative(handle, actionIndex);
    }

    public float getFrustration() {
        return getFrustration(handle);
    }

    public float getAdrenaline() {
        return getAdrenaline(handle);
    }

    public float[] getNeuronStates() {
        return getNeuronStates(handle);
    }

    public int saveModel(String path) {
        return saveNativeModel(handle, path);
    }

    public int loadModel(String path) {
        return loadNativeModel(handle, path);
    }

    @Override
    public void close() {
        if (handle != 0) {
            destroyNativeSingularity(handle);
        }
    }
}