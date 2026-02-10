<div align="center">
  
# Dark Singularity

> **The Monolithic Wave-State Intelligence Engine**

[![Version](https://img.shields.io/badge/version-1.5.0-purple.svg)](#)

</div>

---

## 💎 The MWSO Architecture
### Monolithic Wave-State Operator (V1.5 "Relativity & Flow")

本エンジンは、複素ラテント空間における波動干渉を核とした**「動的知能構造」**です。単なるマッピングではなく、空間内のエネルギー伝播、重力場、および時系列的な流れをシミュレートすることで知能を実現しています。

### 1. 🌊 Temporal Flow Dynamics (時間的重畳)
情報は単一の「点」ではなく、減衰する残響を伴う**「流れ（Flow）」**として注入されます。
- **State Temporal Smearing:** 直前の入力状態を余韻として保持し、現在の入力と干渉させることで「文脈（Trajectory）」を物理的に符号化。
- **Action Momentum:** 過去の成功体験に基づく「慣性」を行動スコアに付与し、カオス的なノイズ下でも安定した一貫性を維持。

### 2. 🌌 Cosmic General Relativity (重力場と特異点)
学習した記憶を「パラメータ」ではなく**「時空の歪み（重力場）」**として定義します。
- **Black Holes (Singularity):** 高い報酬が得られた地点に `gravity_field` を形成。事象の地平線内では情報の「蒸発（忘却）」が極限まで低下し、無入力でもリズムが保存される「公転状態（Orbit）」へと移行。
- **Wormholes (Entanglement):** 非局所的な地点間に位相の同期パスを形成。前提から結論への「論理の跳躍」を、情報の瞬間移動（量子もつれ）として実現。
- **Hawking Radiation:** 重力場の緩やかな自然蒸発による動的な忘却制御。

### 3. 🧠 Inverse Reinforcement Learning (動機逆算)
「失敗から学ぶ」強化学習に加え、「行動から動機を学ぶ」逆強化学習を統合。
- **Motivation Inference:** エキスパートの行動を観測し、その背後にある「ハミルトニアン場」を逆算して自己の知能構造へ定着。
- **Imitation Phase Locking:** 報酬信号なしでも、他者の行動からその「意図」を抽出・模倣することが可能。

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