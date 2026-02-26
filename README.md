<div align="center">
  
# Dark Singularity

> **The Monolithic Wave-State Intelligence Engine**

[![Version](https://img.shields.io/badge/version-1.5.0-purple.svg)](#)

</div>

---

## 💎 The MWSO Architecture
### Monolithic Wave-State Operator (v1.5 "Relativity & Flow")

本エンジンは、複素潜在空間における波動干渉を基盤とした**「流体知能アーキテクチャ」**です。静的なデータ処理にとどまらず、空間内のエネルギー伝播やポテンシャル場の変動、時系列的な「流れ」をシミュレートすることで、柔軟かつ強固な適応知能を実現します。

---

### 1. 🌊 Temporal Flow Dynamics（時系列的流体特性）
情報は断続的な「点」ではなく、時間的な連続性を持つ**「流れ（Flow）」**として処理されます。

- **State Temporal Smearing:** 入力状態に適切な「余韻」を持たせることで、直近のコンテキストを現在の入力と統合。一過性のノイズに惑わされない、安定した「軌道（Trajectory）」を形成します。
- **Action Momentum:** 行動決定に物理的な「慣性」の概念を導入。カオス的な環境下でも、一貫性のある合理的なアプローチを維持します。

### 2. 🌌 Field-Centric Memory System（場による記憶定義）
記憶を固定的な「パラメータ」としてではなく、**「場の歪み（ポテンシャル・フィールド）」**として動的に定義します。

- **Stability Attractors (Singularity):** 高い報酬を得た状態を強力な引力点（アトラクタ）として定着。無入力時でも安定したパターンを保持する「自己保存状態」への移行を可能にします。
- **Entangled Synchrony (Wormhole):** 離れた事象間に位相的な同期パスを形成。複雑な論理ステップをバイパスし、前提から結論への「直感的な最適化」を低遅延で実現します。
- **Dynamic Dissipation (Hawking Radiation):** 場のポテンシャルを緩やかに調整することで、情報の過度な固着を防ぎ、常に最新の環境へ適応するための「最適忘却」を制御します。

### 3. 🧠 Motivation Inference（動機的適応）
外部からの報酬を待つ強化学習に加え、対象の振る舞いからその背後にある**「目的（意図）」を逆算**する機能を統合しています。

- **Hamiltonian Inference:** 対象の行動軌跡から、その動きを規定しているエネルギー地形（ハミルトニアン場）を推定。環境の「本質的な法則」を早期に特定します。
- **Intentional Phase Locking:** 明示的な報酬信号が不足している状況でも、周囲の意図に同調（位相同期）し、目的に沿った高度な模倣や連携を可能にします。

---

## 🧠 Core Philosophy

1. **時定数のスペクトラム化**
   1つのモデルの中に「ゆっくり変化する波（重力記憶）」と「速く変化する波（反射）」を共存。
2. **弾性的失敗記憶**
   失敗した行動には「反発力」を付与。時間の経過とともに弾性的に回復する「 dissipative failure memory」を実装。
3. **因果関係の物理化**
   「Aの次にBが起こる」という論理を、波動の伝播速度とワームホール接続によって物理量として表現。

---

## 🚀 Quick Start (Java API)

本プロジェクトは、Minecraft プラグイン等の Java 環境から JNI 経由で直接利用可能です。

### 1. Integration

#### Maven
`pom.xml` にリポジトリと依存関係を追加してください。

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

### 2. Implementation
`Singularity` クラスは `AutoCloseable` を実装しているため、`try-with-resources` での使用を推奨します。

```java
import com.lunar_prototype.dark_singularity_api.Singularity;

public class SingularityAI {
    public void onTick() {
        try (Singularity ai = new Singularity(1024, 8, 4)) {
            // 状態(インデックス)を衝撃波として注入し、行動波形を観測
            int[] actions = ai.selectActions(new float[]{currentStateIdx});
            
            // 報酬は空間の屈折率(Theta)を直接書き換える
            float reward = execute(actions);
            ai.learn(reward);
            
            // 波動状態を含むスナップショットを保存 (DSYM V6)
            ai.saveModel("brain.dsym");
        }
    }
}
```

### 3. ⚡ Hamiltonian Scripting (Initial Knowledge)
MWSOは学習（本能）だけでなく、ハミルトニアン・スクリプティングによる初期知識（理性）の注入が可能です。ルールは「if-then」の条件分岐ではなく、波動を特定の行動へ誘う**「外場（Potential Field）」**として機能します。

```java
// 1. ルールの登録 (初期化時)
// 条件ID 0(LowHP) のとき、アクション 3(Evade) への引力を 0.8 の強さで発生させる
ai.registerHamiltonianRules(
    new int[]{0},      // Condition IDs
    new int[]{3},      // Target Action Indices
    new float[]{0.8f}  // Resonance Strengths
);

// 2. 状況の注入 (毎チック)
if (entity.getHealth() < 5.0) {
    ai.setActiveConditions(0); // LowHP 条件をアクティブ化
} else {
    ai.setActiveConditions();  // 条件クリア
}

// 以降の selectActions は、学習された直感と注入された理性のベクトル合成で決定される
```

---

## 📊 Evolutionary Phases

システム温度（System Temperature）に基づき、空間の「媒質」としての性質が変化します。

| Phase | State | Characteristics |
| :--- | :--- | :--- |
| 🟦 **Solid** | 結晶化 | 戦術波による精密な指令。重力による記憶の固定が最大化。 |
| 🟩 **Liquid** | 流動体 | 標準的な適応状態。環境の波紋に柔軟に同調。 |
| 🟥 **Gas** | 混沌 | 高エネルギー状態。既存の重力場を破壊し新たな解を模索。 |

---

## 🛠 For Developers

### Project Structure
- `src/core/mwso.rs`: 波動作用素・重力場・ワームホールの核
- `src/core/singularity.rs`: 履歴管理・IRL・時系列学習の統括
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
