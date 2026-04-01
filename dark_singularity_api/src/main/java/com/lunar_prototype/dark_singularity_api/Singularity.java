package com.lunar_prototype.dark_singularity_api;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * JNI wrapper for the Rust `dark-singularity` native library.
 */
public class Singularity implements AutoCloseable {

    private final long handle;
    private final AtomicBoolean closed = new AtomicBoolean(false);

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

            String archDir = arch.contains("64") ? (arch.contains("aarch") || arch.contains("arm") ? "aarch64" : "x86_64") : arch;
            String libName = "dark_singularity";
            String nativeFileName = prefix + libName + extension;
            String nativeResourcePath = "/native/" + osDir + "/" + archDir + "/" + nativeFileName;
            
            InputStream in = Singularity.class.getResourceAsStream(nativeResourcePath);
            if (in == null) {
                nativeResourcePath = "/native/" + osDir + "/" + nativeFileName;
                in = Singularity.class.getResourceAsStream(nativeResourcePath);
            }

            if (in == null) {
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
            
        } catch (IOException | UnsatisfiedLinkError e) {
            throw new UnsatisfiedLinkError("Failed to load native library: " + e.getMessage());
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
    private native void observeExpertNative(long handle, int stateIdx, int[] expertActions, float strength);
    private native void suppressExpertNative(long handle, int[] badActions, float strength);

    public Singularity(int stateSize, int... categorySizes) {
        if (categorySizes == null || categorySizes.length == 0) {
            throw new IllegalArgumentException("At least one action category must be defined.");
        }
        this.handle = initNativeSingularity(stateSize, categorySizes);
        if (this.handle == 0) {
            throw new IllegalStateException("Failed to initialize native Singularity instance.");
        }
    }

    private void checkClosed() {
        if (closed.get()) throw new IllegalStateException("Singularity instance is already closed.");
    }

    public int selectAction(float[] inputs) {
        checkClosed();
        return selectActionNative(handle, inputs);
    }

    public int[] selectActions(float[] inputs) {
        checkClosed();
        return selectActionsNative(handle, inputs);
    }
    
    public void learn(float reward) {
        checkClosed();
        learnNative(handle, reward);
    }

    public float getSystemTemperature() {
        checkClosed();
        return getSystemTemperature(handle);
    }
    
    public float getActionScore(int actionIndex) {
        checkClosed();
        return getActionScoreNative(handle, actionIndex);
    }

    public float getFrustration() {
        checkClosed();
        return getFrustration(handle);
    }

    public float getAdrenaline() {
        checkClosed();
        return getAdrenaline(handle);
    }

    public void setNeuronState(int idx, float state) {
        checkClosed();
        setNeuronStateNative(handle, idx, state);
    }

    public void setExplorationBeta(float beta) {
        checkClosed();
        setExplorationBetaNative(handle, beta);
    }

    public float getExplorationBeta() {
        checkClosed();
        return getExplorationBetaNative(handle);
    }

    public int generateVisualSnapshot(String path) {
        checkClosed();
        return generateVisualSnapshotNative(handle, path);
    }

    public float[] getNeuronStates() {
        checkClosed();
        return getNeuronStates(handle);
    }

    public int saveModel(String path) {
        checkClosed();
        return saveNativeModel(handle, path);
    }

    public int loadModel(String path) {
        checkClosed();
        return loadNativeModel(handle, path);
    }

    public void registerHamiltonianRules(int[] conditionIds, int[] actionIndices, float[] resonanceStrengths) {
        checkClosed();
        if (conditionIds == null || actionIndices == null || resonanceStrengths == null ||
            conditionIds.length != actionIndices.length || actionIndices.length != resonanceStrengths.length) {
            throw new IllegalArgumentException("Arrays must be non-null and have the same length.");
        }
        bootstrapNative(handle, conditionIds, actionIndices, resonanceStrengths);
    }

    public void setActiveConditions(int... conditionIds) {
        checkClosed();
        setActiveConditionsNative(handle, conditionIds);
    }

    /**
     * Observes expert actions to perform imitation learning (IRL).
     */
    public void observeExpert(int stateIdx, int[] expertActions, float strength) {
        checkClosed();
        observeExpertNative(handle, stateIdx, expertActions, strength);
    }

    /**
     * Suppresses specific bad actions via negative feedback.
     */
    public void suppressExpert(int[] badActions, float strength) {
        checkClosed();
        suppressExpertNative(handle, badActions, strength);
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            destroyNativeSingularity(handle);
        }
    }
}
