<div align="center">
  
# Dark Singularity

> **The Monolithic Wave-State Intelligence Engine**

[![Version](https://img.shields.io/badge/version-1.3.0-purple.svg)](#)

</div>

---

## 💎 The MWSO Architecture
### Monolithic Wave-State Operator

本エンジンは、512個のパラメータによって定義された**「知能という性質を持つ空間」**そのものです。上位も下位も、判断も行動も、すべては一つの波動体の中に重畳されています。

- **🌊 Universal Dynamics**
  情報は複素ラテント空間 ($\mathbb{C}^{256}$) における波として処理されます。
  - **Strategy:** 空間全体のゆったりとした大きな「うねり」
  - **Memory:** 空間に残り続ける定常波の「残響」
  - **Reflex:** 瞬時に伝播する鋭い「衝撃波」
- **🧬 Holographic Coupling**
  隣接する成分のみならず、遠隔周波数同士を `stride` 結合。文脈が反射を直接変調する高次元ホログラムを形成。
- **🚀 Table-less Intelligence (DSYM V6)**
  Qテーブルを完全に排除。入力された環境状態は「波」として注入され、パラメータ ($\theta$) を通じてアクションスコアへと直接収束（投影）されます。

---

## 🧠 Core Philosophy

1. **時定数のスペクトラム化**
   1つのモデルの中に「ゆっくり変化する波」と「速く変化する波」を共存させ、思考と行動を完全同期。
2. **自己参照的フィードバック**
   波の一部を自身へ還流させることで、RNNのような記憶保持を物理的ループで実現。
3. **極限のメモリ効率**
   状態数（State Size）に依存せず、常に**固定 512 パラメータ**で完結。

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
| 🟦 **Solid** | 結晶化 | 戦術波による精密な指令。無駄なノイズを排除した最適化。 |
| 🟩 **Liquid** | 流動体 | 標準的な適応状態。環境の波紋に柔軟に同調。 |
| 🟥 **Gas** | 混沌 | 高エネルギー状態。既存の干渉パターンを破壊し新たな解を模索。 |

---

## 🛠 For Developers

### Build
- **Local**: `mvn package` (Rust コンパイルと JAR 生成を自動実行)
- **Engine Only**: `cargo build --release`

### Project Structure
- `src/core/mwso.rs`: 波動作用素の核 (The Unified PDE)
- `src/core/singularity.rs`: 波動空間の統括
- `dark_singularity_api/`: Java 用 JNI ラッパー

---

## ⚠️ Disclaimer

**本プロジェクトは RuskLabo (Lunar_prototype) の専属的な研究成果です。**

- **Latest Only**: 常に最新のブランチ・バージョンのみをサポート対象とします。下位互換性の維持や過去バージョンの修正は行いません。
- **No Support**: 過去のアーティファクトに関する問い合わせには応答しません。
- **Destructive Updates**: 予告なしに内部構造や JNI シグネチャが変更される場合があります。

## 🚫 Usage Restriction Notice

本ソフトウェアの軍事目的での利用は想定されておらず、推奨もいたしません。開発者である RuskLabo (Lunar_prototype) は、技術が平和的かつ建設的な文脈で活用されることを望んでいます。

---

© 2026 **RuskLabo / Lunar_prototype**. *Toward the Singularity.*
