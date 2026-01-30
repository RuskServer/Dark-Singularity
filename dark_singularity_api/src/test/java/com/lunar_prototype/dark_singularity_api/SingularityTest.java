package com.lunar_prototype.dark_singularity_api;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SingularityTest {

    @Test
    @DisplayName("Singularityインスタンスが正常に初期化され、クローズされること")
    void testSingularityInitializationAndClose() {
        try (Singularity singularity = new Singularity()) {
            assertNotNull(singularity);
            // 初期状態の温度を確認 (Rust側のデフォルト値 0.5)
            assertEquals(0.5f, singularity.getSystemTemperature(), 0.001f);
            System.out.println("Initial System Temperature: " + singularity.getSystemTemperature());
        } catch (Exception e) {
            fail("Singularityインスタンスの初期化またはクローズ中にエラーが発生しました: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("アクション選択と学習メソッドがエラーなく実行されること")
    void testActionSelectionAndLearning() {
        try (Singularity singularity = new Singularity()) {
            float[] inputs = {10.0f, 0.5f, 0.2f}; // ダミー入力
            int action = singularity.selectAction(inputs);
            assertTrue(action >= 0 && action <= 7, "選択されたアクションが有効な範囲内であること"); // Singularity.select_actionの戻り値範囲

            System.out.println("Selected Action: " + action);

            // 学習メソッドを呼び出す
            float reward = 1.0f;
            singularity.learn(reward);
            System.out.println("System Temperature after learning: " + singularity.getSystemTemperature());

            // マルチアクションもテスト
            int[] multiActions = {0, 5}; // ダミーアクション
            float multiReward = 0.5f;
            singularity.learnMulti(multiActions, multiReward);
            System.out.println("System Temperature after multi-learning: " + singularity.getSystemTemperature());

            // 各種ゲッターの呼び出し
            assertNotNull(singularity.getNeuronStates());
            assertTrue(singularity.getNeuronStates().length > 0);
            System.out.println("Glia Activity: " + singularity.getGliaActivity());
            System.out.println("Frustration: " + singularity.getFrustration());
            System.out.println("Adrenaline: " + singularity.getAdrenaline());

        } catch (Exception e) {
            fail("アクション選択または学習中にエラーが発生しました: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("モデルの保存とロードがエラーなく実行されること")
    void testModelSaveLoad() {
        try (Singularity singularity = new Singularity()) {
            // ダミーのモデルパス (一時ファイルを利用)
            String modelPath = "/Users/RuskDev/.gemini/tmp/6eeac807ac384f89118f6187006433e704b82079ef401588c6acbb7b966e6e8e/test_model.dsym";

            // 保存前に状態を変化させる
            singularity.learn(0.5f);
            float tempBeforeSave = singularity.getSystemTemperature();

            int saveResult = singularity.saveModel(modelPath);
            assertEquals(0, saveResult, "モデル保存が成功すること");

            // 新しいインスタンスにロード
            try (Singularity loadedSingularity = new Singularity()) {
                int loadResult = loadedSingularity.loadModel(modelPath);
                assertEquals(0, loadResult, "モデルロードが成功すること");

                assertEquals(tempBeforeSave, loadedSingularity.getSystemTemperature(), 0.001f, "保存前とロード後の温度が一致すること");
            }
        } catch (Exception e) {
            fail("モデルの保存またはロード中にエラーが発生しました: " + e.getMessage());
        }
    }
}