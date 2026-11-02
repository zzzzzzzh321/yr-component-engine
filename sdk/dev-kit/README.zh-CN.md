# VERA20K 插件独立开发包

本开发包用于在没有 VERA20K 引擎源码的环境中，修改、测试并编译 `vera20k-ares.wasm`。编译只依赖插件源码、SDK 的类型与接口定义，以及 Rust 工具链。

本说明同时保存在引擎仓库的 `sdk/dev-kit/README.zh-CN.md`，并作为独立开发包的根 README 使用。下文所有路径和命令均以**独立开发包根目录**为起点。

## 开发包内容

```text
开发包根目录/
├── Cargo.toml                   # 仅 SDK 与插件的独立 workspace
├── Cargo.lock                   # 独立开发包的依赖版本
├── LICENSE-GPL
├── sdk/
│   ├── api/                     # 请求、结果、公共数据类型与计时器
│   ├── wit/                     # 插件与引擎之间的接口定义
│   ├── package/                 # 通用组件打包工具源码
│   └── dev-kit/                 # 本说明与 workspace 模板
└── plugins/
    └── vera20k-ares/            # 可修改的功能插件源码与 INI 示例
```

根 workspace 只有 `vera-mod-api`、`vera-mod-pack` 和 `vera20k-ares` 三个 crate，默认构建成员为插件。其他第三方依赖由 Cargo 按锁文件获取。这个工程不包含引擎 crate、Wasmtime 运行时或游戏资源，也不需要 Python 和引擎专用的 `tools.cargo_run`。

SDK 中的 API 和 WIT 类似 C++ 插件开发中的公共头文件与调用契约：它们说明双方交换哪些数据、可以调用哪些接口。具体的 Host 服务由运行插件时的引擎提供，编译插件时无需链接引擎源码或可执行文件。

## 准备环境

安装支持 Rust 2024 edition 的 Rust 工具链，确保 Cargo 和当前平台所需的链接器可用。然后安装插件编译目标：

```sh
rustup target add wasm32-unknown-unknown
```

首次构建通常需要联网下载 Cargo 依赖；已缓存的依赖可以复用。保留开发包附带的 `Cargo.lock`，以使用同一组依赖版本。

## 修改与测试

功能代码位于 `plugins/vera20k-ares/src/features/`，仍按防御、武器等领域组织。公共宿主调用集中在 `src/support/host.rs`，组件入口和功能组合位于 `src/component.rs`。

从开发包根目录运行：

```sh
cargo test --locked -p vera20k-ares -p vera-mod-pack --lib
```

这些测试检查插件决策与打包行为。它们可以在没有 `vera20k.exe` 和原版游戏资源的环境中运行；实际游戏效果、引擎接入、存读档和联机行为仍需放入兼容引擎中验收。

## 编译插件

```sh
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
```

原始输出为：

```text
target/wasm32-unknown-unknown/release/vera20k_ares.wasm
```

这是包含 WIT 元数据的 core Wasm，接下来需要封装为引擎加载的组件。此处使用 `wasm32-unknown-unknown`，保持当前 SDK 所要求的导入接口。

## 生成发布文件

先在开发包根目录创建 `dist` 文件夹，再执行以下命令。输出子目录 `release-new` 必须尚不存在；下一次发布使用另一个新子目录。

```sh
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/release-new --program vera20k-ares.wasm
```

Cargo 会编译本机使用的打包工具并执行，产物为：

```text
dist/release-new/
├── vera20k-ares.wasm
└── vera-mod.json
```

声明中的文件名与内容摘要由打包工具生成。将这两个文件一起放入作品资源系统，再用作品实际加载的 INI 选择功能。作品的其他美术、音频、地图和规则资源沿用原有分发流程。

如果设置了 `CARGO_TARGET_DIR`，原始 Wasm 的位置随之改变；打包命令中的输入路径也应使用该目标目录。源码修改和编译过程不会自动改变作品当前对局中的组件。

## SDK 与引擎版本

当前接口为 SDK API v1，WIT 名称为 `vera:mod-sdk@0.1.0`。参考插件保留内部标识 `vera.reference-gameplay`、组件版本 `0.4.0`、配置版本 `4` 和状态版本 `1`。发布文件名仍为 `vera20k-ares.wasm`。

修改现有接口能够表达的算法，可以只重编插件。需要新的引擎能力时，应先扩展 SDK 契约和引擎接入点，再使用匹配的 SDK 构建插件；独立编译并不表示所有未来 SDK 与引擎版本自动兼容。

引擎加载时会检查组件声明、摘要、接口及允许的导入。读档仍要求对应的组件内容，因此更新插件时保留旧版本文件，供旧存档使用。开发包沿用 GPL 授权，见 `LICENSE-GPL`。

## 维护者如何组装开发包

在引擎仓库之外创建新的输出目录，保持上述相对布局，复制 `sdk/api`、`sdk/wit`、`sdk/package`、`sdk/dev-kit`、`plugins/vera20k-ares` 和许可证。将源码仓库中的 `sdk/dev-kit/Cargo.toml.template` 复制为开发包根 `Cargo.toml`，将本说明复制为根 README。

将同一SDK发布版本验证过的源工作区`Cargo.lock`复制到开发包根目录作为初始输入。在发布依赖已缓存的维护环境中，执行：

```sh
cargo metadata --offline --format-version 1
```

第一次整理不加`--locked`，允许Cargo裁掉与独立workspace无关的成员，同时保留已锁定的依赖版本。检查解析后的依赖图只包含开发包成员及其依赖，再使用前述带`--locked`的测试、构建和打包命令验证。分发整理后的锁文件与专用workspace；不直接把引擎原workspace作为开发包根配置。

源码仍在原有目录维护，组装时复制当前版本。一般作者直接使用包内锁文件，无需重新生成；只有维护者明确升级依赖时才重新解析版本。
