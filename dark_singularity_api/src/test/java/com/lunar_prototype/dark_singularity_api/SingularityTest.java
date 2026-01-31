package com.lunar_prototype.dark_singularity_api;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SingularityTest {

    @Test
    @DisplayName("Singularityインスタンスが正常に初期化され、クローズされること")
    void testSingularityInitializationAndClose() {
        // 512状態、2カテゴリー (5アクションと3アクション)
        try (Singularity singularity = new Singularity(512, 5, 3)) {
            assertNotNull(singularity);
            assertEquals(0.5f, singularity.getSystemTemperature(), 0.001f);
            System.out.println("Initial System Temperature: " + singularity.getSystemTemperature());
        } catch (Exception e) {
            fail("Singularityインスタンスの初期化またはクローズ中にエラーが発生しました: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("多層アクション選択と学習メソッドがエラーなく実行されること")
    void testActionSelectionAndLearning() {
        try (Singularity singularity = new Singularity(512, 5, 3)) {
            float[] inputs = {10.0f, 0.5f, 0.2f}; 
            
            // 全カテゴリーのアクションを取得
            int[] actions = singularity.selectActions(inputs);
            assertEquals(2, actions.length, "カテゴリー数と同じ数のアクションが返されること");
            assertTrue(actions[0] >= 0 && actions[0] < 5);
            assertTrue(actions[1] >= 0 && actions[1] < 3);

            System.out.println("Selected Actions: " + actions[0] + ", " + actions[1]);

            // 学習メソッドを呼び出す (選択された全アクションに報酬が適用される)
            float reward = 1.0f;
            singularity.learn(reward);
            System.out.println("System Temperature after learning: " + singularity.getSystemTemperature());

            // 各種ゲッターの呼び出し
            assertNotNull(singularity.getNeuronStates());
            assertTrue(singularity.getNeuronStates().length > 0);
        } catch (Exception e) {
            fail("アクション選択または学習中にエラーが発生しました: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("モデルの保存とロードがエラーなく実行されること")
    void testModelSaveLoad() {
        try (Singularity singularity = new Singularity(512, 5, 3)) {
            String modelPath = "/Users/RuskDev/.gemini/tmp/6eeac807ac384f89118f6187006433e704b82079ef401588c6acbb7b966e6e8e/test_model_v2.dsym";

            singularity.learn(0.5f);
            float tempBeforeSave = singularity.getSystemTemperature();

            int saveResult = singularity.saveModel(modelPath);
            assertEquals(0, saveResult, "モデル保存が成功すること");

            // 同じ構成でロード
            try (Singularity loadedSingularity = new Singularity(512, 5, 3)) {
                int loadResult = loadedSingularity.loadModel(modelPath);
                assertEquals(0, loadResult, "モデルロードが成功すること");
                assertEquals(tempBeforeSave, loadedSingularity.getSystemTemperature(), 0.001f);
            }
        } catch (Exception e) {
            fail("モデルの保存またはロード中にエラーが発生しました: " + e.getMessage());
        }
    }
}