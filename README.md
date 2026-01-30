# DarkSingularity

> **Thermodynamic Q-Homeostasis (TQH) AI Engine for Minecraft Entities**

`DarkSingularity` は、Minecraft のエンティティに高度な戦術的知能を与えるために設計された、Rust 製のハイブリッド型ニューラルネットワーク・エンジンです。
物理シミュレーションに基づいた熱力学的な意思決定プロセスを JNI 経由で Java 側へ提供します。

---

## 🧠 Core Philosophy

本エンジンは、生物の神経系が持つ「興奮」と「冷却」のプロセスを数学的に模倣した **TQH (Thermodynamic Q-Homeostasis)** 理論に基づいています。

- **Thermal Liquidity (熱流動性)**
  TD誤差を「熱」として扱い、未知の状況下で脳構造を動的に相転移させます。
- **Elastic Action Selection**
  疲労（Fatigue Map）と不満（Frustration）をパラメータに含めた、揺らぎのある行動選択。
- **Cross-Platform Integration**
  Windows, macOS, Linux の各プラットフォームに対応し、単一の JAR ファイルでシームレスに動作します。

---

## 🚀 Getting Started (Java API)

本プロジェクトは現在、他の Java プロジェクト（Minecraft プラグイン等）から直接利用可能なライブラリとして提供されています。

### 1. 導入
Maven または Gradle を使用してプロジェクトに追加できます。

#### Maven
`pom.xml` に以下のリポジトリと依存関係を追加してください。

```xml
<repositories>
    <repository>
        <id>ruskserver-releases</id>
        <url>https://repo.ruskserver.com/repository/maven-releases/</url>
    </repository>
</repositories>

<dependencies>
    <dependency>
        <groupId>com.lunar_prototype</groupId>
        <artifactId>dark_singularity_api</artifactId>
        <version>1.0.0</version> <!-- 適宜最新バージョンに変更してください -->
    </dependency>
</dependencies>
```

#### Gradle (Kotlin DSL)
`build.gradle.kts` に以下を追加してください。

```kotlin
repositories {
    maven("https://repo.ruskserver.com/repository/maven-releases/")
}

dependencies {
    implementation("com.lunar_prototype:dark_singularity_api:1.0.0")
}
```

> [!TIP]
> 直接 JAR を入手したい場合は、[GitHub Releases](https://github.com/RuskServer/Dark-Singularity/releases) からもダウンロード可能です。この JAR にはすべてのプラットフォーム用のネイティブライブラリが同梱されています。

### 2. 基本的な実装
`Singularity` クラスは `AutoCloseable` を実装しているため、`try-with-resources` での使用を推奨します。

```java
import com.lunar_prototype.dark_singularity_api.Singularity;

public class MyEntityAI {
    public void tick() {
        try (Singularity ai = new Singularity()) {
            // 1. 状態の入力とアクション選択 (0-7)
            float[] state = getEnvironmentalState();
            int action = ai.selectAction(state);
            
            // 2. 実行結果の学習
            float reward = performAction(action);
            ai.learn(reward);
            
            // 3. 状態の保存
            ai.saveModel("data/brain.dsym");
        }
    }
}
```

---

## 📊 Phase Analysis

`Singularity` は、システム温度（System Temperature）によって脳の結合構造を変化させます。

| Phase | Characteristics | Characteristics (JP) |
| :--- | :--- | :--- |
| 🟦 **Solid** | Precision & Optimization | 冷徹な最適化。予測精度が高い安定状態。 |
| 🟩 **Liquid** | Flexibility & Learning | 標準的な適応状態。環境に合わせて柔軟に変化。 |
| 🟥 **Gas** | Chaos & Exploration | 相転移・混沌状態。新たな戦術を模索する暴走状態。 |

---

## 🛠 For Developers

### プロジェクト構造
- `src/core/`: Rust 製 AI エンジンのコアロジック
- `src/lib.rs`: JNI ブリッジ (Rust 側)
- `dark_singularity_api/`: Java 向けラッパーライブラリ

### ビルド
本プロジェクトは GitHub Actions による完全自動ビルドに対応しています。
- **Local**: `dark_singularity_api` ディレクトリで `mvn package` を実行すると、Rust のコンパイルから JAR の生成まで自動で行われます。
- **Release**: `v*` タグをプッシュすることで、全プラットフォーム対応の統合 JAR が自動生成され、GitHub Release にアップロードされます。

---

## ⚠️ Maintenance Policy (Disclaimer)

**本プロジェクトは RuskLabo (Lunar_prototype) の専属的な研究成果です。**

- **Latest Only**: 常に最新のブランチ・バージョンのみをサポート対象とします。下位互換性の維持や過去バージョンの修正は行いません。
- **No Support**: 過去のアーティファクトに関する問い合わせには応答しません。
- **Destructive Updates**: 予告なしに内部構造や JNI シグネチャが変更される場合があります。

---

© 2026 **RuskLabo / Lunar_prototype**. All rights reserved.