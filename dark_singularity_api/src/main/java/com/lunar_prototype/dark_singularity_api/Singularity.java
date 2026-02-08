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

    // --- Constants for Default Neurons ---
    public static final int IDX_AGGRESSION = 0;
    public static final int IDX_FEAR = 1;
    public static final int IDX_TACTICAL = 2;
    public static final int IDX_REFLEX = 3;

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
    private native void setNeuronStateNative(long handle, int idx, float state);
    private native float[] getNeuronStates(long handle);
    private native void setExplorationBetaNative(long handle, float beta);
    private native float getExplorationBetaNative(long handle);
    private native int generateVisualSnapshotNative(long handle, String path);
    private native int saveNativeModel(long handle, String path);
    private native int loadNativeModel(long handle, String path);
    private native void setActiveConditionsNative(long handle, int[] conditionIds);
    private native void bootstrapNative(long handle, int[] conditionIndices, int[] actionIndices, float[] strengths);

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

    /**
     * Directly sets the state of a specific neuron.
     * 
     * @param idx   Index of the neuron (0-3 for default nodes).
     * @param state Value between 0.0 and 1.0.
     */
    public void setNeuronState(int idx, float state) {
        setNeuronStateNative(handle, idx, state);
    }

    public void setExplorationBeta(float beta) {
        setExplorationBetaNative(handle, beta);
    }

    public float getExplorationBeta() {
        return getExplorationBetaNative(handle);
    }

    /**
     * Generates a 3D visualization of the current MWSO wave-state.
     * Perfect for social preview or debugging the 'shape' of intelligence.
     * 
     * @param path File path to save the PNG (e.g., "wave.png").
     * @return 0 on success, non-zero on error.
     */
    public int generateVisualSnapshot(String path) {
        return generateVisualSnapshotNative(handle, path);
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

    /**
     * Injects Hamiltonian rules into the model. 
     * Rules act as "potential fields" that guide the agent's wave-state towards specific actions.
     * 
     * @param conditionIds IDs of environmental conditions (e.g., 0 for LowHP, 1 for EnemyNear).
     * @param actionIndices Target action indices to encourage.
     * @param resonanceStrengths Strength of the knowledge field (1.0 = strong, 0.1 = subtle hint).
     */
    public void registerHamiltonianRules(int[] conditionIds, int[] actionIndices, float[] resonanceStrengths) {
        if (conditionIds == null || actionIndices == null || resonanceStrengths == null ||
            conditionIds.length != actionIndices.length || actionIndices.length != resonanceStrengths.length) {
            throw new IllegalArgumentException("Arrays must be non-null and have the same length.");
        }
        bootstrapNative(handle, conditionIds, actionIndices, resonanceStrengths);
    }

    /**
     * Sets the currently active environmental conditions.
     * Rules associated with these IDs will exert "Hamiltonian force" on the current decision.
     */
    public void setActiveConditions(int... conditionIds) {
        setActiveConditionsNative(handle, conditionIds);
    }

    @Override
    public void close() {

    }
}