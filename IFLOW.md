# IFLOW.md - digraphx-rs 项目说明

## 项目概述

`digraphx-rs` 是一个用 Rust 编写的网络优化算法库，专注于有向图中的算法，特别是与最短路径和负环检测相关的算法。该项目基于 `petgraph` 库构建，提供了 Bellman-Ford 算法的变体，用于处理带负权重边的图，并包含专门用于查找负环和解决参数化优化问题的算法。

主要特性包括：
- 带负权重边的最短路径算法（Bellman-Ford）
- 负环检测和查找算法
- 参数化优化算法（如参数化最小比率环算法）
- 使用 `num` 和 `num-traits` 库支持多种数值类型（如浮点数、有理数）

## 构建和运行

### 环境要求
- Rust 工具链（包括 Cargo）

### 安装
```bash
# 通过 Cargo 安装
cargo install digraphx-rs
```

### 本地开发
```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行所有测试（包括文档测试）
cargo test --all

# 构建发布版本
cargo build --release
```

## 代码结构

- `src/lib.rs`: 库的主入口点，包含核心的 Bellman-Ford 算法实现。
- `src/neg_cycle.rs`: 包含 `NegCycleFinder` 结构体，用于查找负环，实现 Howard 算法。
- `src/parametric.rs`: 包含参数化优化求解器，用于解决最小比率环等问题。
- `Cargo.toml`: 项目的依赖和元数据配置文件。

## 核心功能

### Bellman-Ford 算法
- `bellman_ford`: 计算从源节点到所有其他节点的最短路径，支持负权重边，但会检测负权重环。
- `find_negative_cycle`: 从给定源节点开始查找负权重环。
- `bellman_ford_initialize_relax`: 执行 Bellman-Ford 算法的初始化和松弛步骤。

### 负环查找
- `NegCycleFinder`: 使用 Howard 算法查找图中的负环。

### 参数化优化
- `MaxParametricSolver`: 解决参数化优化问题，如最小比率环问题。

## 开发约定

- 代码遵循 Rust 编码风格和最佳实践。
- 使用 `petgraph` 库进行图表示和基本操作。
- 支持泛型编程，以处理不同的节点值类型和边权重类型。
- 包含全面的单元测试和文档测试。

## 许可证

该项目根据 MIT 许可证或 Apache 许可证 2.0 版（LICENSE-MIT 或 LICENSE-APACHE）双重许可。