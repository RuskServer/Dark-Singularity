# DarkSingularity

> **Thermodynamic Q-Homeostasis (TQH) AI Engine for Minecraft Entities**

`DarkSingularity` は、Minecraft のエンティティに高度な戦術的知能を与えるために設計された、Rust 製のハイブリッド型ニューラルネットワーク・エンジンです。Java 側の低速な演算を排し、物理シミュレーションに基づいた熱力学的な意思決定プロセスを提供します。

---

## 🧠 Core Philosophy

本エンジンは、生物の神経系が持つ「興奮」と「冷却」のプロセスを数学的に模倣した **TQH (Thermodynamic Q-Homeostasis)** 理論に基づいています。

- **Thermal Liquidity (熱流動性)**
  TD誤差を「熱」として扱い、未知の状況下で脳構造を相転移させます。
- **Elastic Action Selection**
  疲労（Fatigue Map）と不満（Frustration）による動的な行動選択。
- **JNI-Bridge Optimization**
  Java と Rust 間のメモリ共有を最適化し、ティック毎のオーバーヘッドを極限まで削減。

---

## 🛠 Project Structure

- `src/core/`: Rust 側の知能中枢 (`Singularity`, `Node`, `Horizon`)
- `src/lib.rs`: JNI ブリッジインターフェース

---

## ⚠️ Maintenance Policy (Disclaimer)

**本プロジェクトは RuskLabo (Lunar_prototype) の専属的な研究成果です。**

- **Latest Only**
  我々は常に「最新こそが最強」であると信じています。そのため、**過去バージョンのメンテナンス、バグ修正、および下位互換性の維持は一切行いません。**
- **No Support**
  過去のアーティファクトに関する問い合わせには応答しません。常に最新のブランチを使用してください。
- **Destructive Updates**
  予告なしに JNI シグネチャやデータ構造が変更される場合があります。

---

## 🚀 Installation

1. `cargo build --release` を実行。
2. 生成された `libdark_singularity.so` (または `.dll` / `.dylib`) を、プラグイン(https://github.com/RuskServer/Deepwither)のデータフォルダ (`plugins/deepwither/`) に配置。 
3. `LiquidBrain.java` を通じてエンティティの意思決定を委譲。

---

## 📊 Analytics & Visualisation

`LiquidBrain` は、戦闘中の脳状態を Snapshot として記録可能です。

| Phase | State | Characteristics |
| :--- | :--- | :--- |
| 🟦 **Solid** | Blue | 冷徹な最適化状態。予測精度が高い。 |
| 🟩 **Liquid** | Green | 標準的な学習状態。 |
| 🟥 **Gas** | Red | 相転移・混沌状態。予測が外れ、新たな戦術を模索中。 |

---

© 2026 **RuskLabo / Lunar_prototype**. All rights reserved.
