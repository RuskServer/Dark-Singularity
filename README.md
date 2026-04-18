<div align="center">
  
# Dark Singularity

> **The Monolithic Wave-State Intelligence Engine**

[![Version](https://img.shields.io/badge/version-1.6.0-purple.svg)](#)

</div>

---

## 💎 The MWSO Architecture
### Monolithic Wave-State Operator (v2.0 "Resonance & Completion")

本エンジンは、複素潜在空間における波動干渉と共鳴現象を基盤とした**「流体共鳴知能アーキテクチャ」**です。最新バージョンでは、単なる情報の記録を超え、不完全な入力から過去の成功体験を物理的に復元する「自己連想レゾナンス」を統合しました。

---

### 1. 🌊 Wave-State Dynamics（波動動態）
情報は複素空間上の波として定義され、重ね合わせと干渉によって処理されます。

- **Similarity Resonance:** 現在の思考波形が記憶された成功パターンと一定以上の位相相関（Coherence）を示した瞬間、想起強度が非線形に爆発。微かな類似性を強力な確信へと変換します。
- **Self-Associative Completion:** 入力情報の一部が欠落していても、残された特徴量から波が自律的に全体像を復元。情報の「穴」を埋めながら、過去の最適解へ波動を導きます。

### 2. 🌌 Field-Centric Memory System（場による記憶定義）
記憶は静的なパラメータではなく、波動を導く**「エネルギー景観（Energy Landscape）」**の歪みとして定義されます。

- **Input-Memory Cross Resonance:** 外界からの入力と内部記憶が一致する箇所でポテンシャルの谷が深化。入力を「知覚」するだけでなく、それが「既知の重要な手がかり」である場合に処理の解像度を自動的に引き上げます。
- **Dynamic Dissipation & Phase Guard:** 矛盾する記憶や古いパターン（逆位相）は、コヒーレンス・チェックにより自動的に排斥。環境変化時にはシステム温度を上昇させ、固執を解除することで柔軟な再学習を可能にします。

### 3. 🧠 Multi-Feature Superposition（多特徴重畳）
単一の事象だけでなく、複数の重み付き特徴量を同時に波動空間へ注入可能です。

- **Vector State Processing:** 複雑な状況を「複数の純粋状態の重ね合わせ」として処理。それぞれの特徴量が持つ記憶を並列的に想起し、それらの干渉結果として最終的な意思決定を下します。

---

## 🧠 Core Philosophy

1. **レゾナンスによる確信の爆発**
   「なんとなく似ている」という曖昧な類似性を、波動の共鳴現象によって「これだ」という強い確信（収束）に変換。
2. **情報の穴埋め（自己補完）**
   断片的な手がかりから全体像を推論する能力を、物理的な「引き込み現象（Entrainment）」として実装。
3. **適応的忘却と慣性のバランス**
   成功への執着（自己連想）と、変化への柔軟性（フラストレーションによる加熱）をシステム温度によってダイナミックに制御。

---

## 🚀 Quick Start (Java API)

本プロジェクトは、Minecraft プラグイン等の Java 環境から JNI 経由で直接利用可能です。

### 1. Integration

#### Maven
~~`pom.xml` にリポジトリと依存関係を追加してください。~~ 廃止されました。代替手段を用意する予定ではありますが未定です。

```xml
<repositories>
    <repository>
        <id>ruskserver-releases</id>
        <url>https://repo.ruskserver.com/repository/maven-public/</url>
    </repository>
</repositories>

<dependencies>
    <dependency>
        <groupId>com.lunar_prototype</groupId>
        <artifactId>dark_singularity_api</artifactId>
        <version>1.3.0</version> <!-- 最新バージョン -->
    </dependency>
</dependencies>
```

#### Gradle (Kotlin DSL)
`build.gradle.kts` に以下を追加してください。

```kotlin
repositories {
    maven("https://repo.ruskserver.com/repository/maven-public/")
}

dependencies {
    implementation("com.lunar_prototype:dark_singularity_api:1.3.0")
}
```

> [!TIP]
> [GitHub Releases](https://github.com/RuskDev/dark-singularity/releases) からダウンロード可能な JAR には、Windows, macOS, Linux すべてのネイティブライブラリが同梱されています。

### 2. Implementation (Vector State / Hole-Filling)
新しく追加された `selectActionsVector` を使用することで、複数の特徴量を組み合わせた高度な推論が可能です。

```java
import com.lunar_prototype.dark_singularity_api.Singularity;

public class SingularityAI {
    public void onTick() {
        try (Singularity ai = new Singularity(1024, 16)) {
            // 複数特徴量の注入 (状態5: 1.0, 状態10: 0.5 の重み)
            int[] actions = ai.selectActionsVector(
                new int[]{5, 10}, 
                new float[]{1.0f, 0.5f}
            );
            
            float reward = execute(actions);
            // ベクトル履歴に基づいた学習
            ai.learn(reward); 
            
            // スナップショットの保存
            ai.saveModel("brain.dsym");
        }
    }
}
```

### 3. ⚡ Hamiltonian Scripting (Initial Knowledge)
MWSOは学習（本能）だけでなく、ハミルトニアン・スクリプティングによる初期知識（理性）の注入が可能です。ルールは「if-then」の条件分岐ではなく、波動を特定の行動へ誘う**「外場（Potential Field）」**として機能します。

```java
// 1. ルールの登録 (初期化時)
ai.registerHamiltonianRules(
    new int[]{0},      // Condition IDs
    new int[]{3},      // Target Action Indices
    new float[]{0.8f}  // Resonance Strengths
);

// 2. 状況の注入 (毎チック)
if (entity.getHealth() < 5.0) {
    ai.setActiveConditions(0); // LowHP 条件をアクティブ化
}
```

---

## 📊 Evolutionary Phases

システム温度（System Temperature）に基づき、空間の「媒質」としての性質が変化します。温度が高い時は古い記憶（自己連想）への依存が自動的に抑制されます。

| Phase | State | Characteristics |
| :--- | :--- | :--- |
| 🟦 **Solid** | 結晶化 | 想起のレゾナンスが最大化。過去の成功パターンへの強烈な引き込み。 |
| 🟩 **Liquid** | 流動体 | 標準的な適応状態。入力と記憶のバランスが取れた状態。 |
| 🟥 **Gas** | 混沌 | 高エネルギー状態。自己連想を解除し、全く新しい解を模索。 |

---

## 🛠 For Developers

### Project Structure
- `src/core/mwso.rs`: 波動作用素・記憶レゾナンス・穴埋めロジックの核
- `src/core/singularity.rs`: 履歴管理・ベクトル学習・温度制御の統括
- `dark_singularity_api/`: Java 用 JNI ラッパー

---

## ⚠️ Disclaimer

**本プロジェクトは RuskLabo (Lunar_prototype) の専属的な研究成果です。**

- **Latest Only**: 常に最新のブランチ・バージョンのみをサポート対象とします。
- **No Support**: 過去のアーティファクトに関する問い合わせには応答しません。
- **Destructive Updates**: 予告なしに内部構造や JNI シグネチャが変更される場合があります。

## 🚫 Usage Restriction Notice

本ソフトウェアの軍事目的での利用は想定されておらず、推奨もいたしません。開発者である RuskLabo (Lunar_prototype) は、技術が平和的かつ建設的な文脈で活用されることを望んでいます。

---

© 2026 **RuskLabo / Lunar_prototype**. *Toward the Singularity.*
