# 外置 Ares 武器弹药消耗

使用 `ares-ammo` 编译特性，在同一护盾组件中增加 Ares 武器 `Ammo=`。引擎不再直接解释武器节里的这个扩展键；无组件或没有新增回调时保持原版的一发需求。

```ini
[MTNK]
Ammo=5
InitialAmmo=5
Reload=120
Primary=SDKAmmoGun
ElitePrimary=SDKAmmoGun

[SDKAmmoGun]
Damage=20
ROF=30
Range=6
Speed=100
Projectile=InvisibleLow
Warhead=SA
Ammo=2
```

单位节的 `Ammo` 是弹仓容量，武器节的 `Ammo` 是一次开火成本。上例五发先变三发，再变一发；剩余一发时等待装填。完整示例见 [examples/ammo.ini](examples/ammo.ini)。

- 默认成本 1，后层缺失、空值或读取失败保留先前有效值，使用引擎已有 Ares 整数读取器。
- 零或负成本允许空弹仓开火，不扣弹，不因射击重启装填；负成本不会补充弹药。
- 当前弹药数为负时保留无限弹药语义。
- 支持车辆、步兵和建筑。建筑和步兵复用通用 Reload；保留建筑 Hospital/Armory 排除条件。
- 按 [官方说明](https://ares-developers.github.io/Ares-docs/new/weapons/ammo.html)，不应用于飞机，保留其独立投弹与返航流程。本版宿主会拒绝插件返回的飞机资源策略，避免只改变开火门槛却未改变独立扣弹流程。
- NoAmmoWeapon、ReloadAmount、EmptyReloadAmount 不在本例中。未启用内置 Ares 时，建筑保留原版弹仓连发节奏，独立的 Ares ROF 修复不包含在本功能中。

## 代码与接口

[src/ammo.rs](src/ammo.rs) 解释配置、选择成本、开火后的装填请求及适用类别。实际弹药状态、扣减和计时继续由引擎已有所有者负责。

可选回调 `weapon_resource` 接收武器名称，返回 `WeaponResourcePolicy`：`cost` 为成本，`reload_after_use` 为射击后装填请求，`users` 为适用类别位掩码。

最终配置阶段通过只读 `registry("$weapons")` 枚举引擎实际分配的武器节（保留精确大小写），使用 `ares-int` 读取器读取成本，并冻结在组件配置中。新增回调只查询冻结配置，不读运行时 INI，不写对象状态。

Rules 接纳时一次查询各武器并绑定只读派生策略，热路径不跨 Wasm。装填资格按对象实际普通、精英和驻军武器派生，局部策略不会改变无关对象。某类别的全部武器都被策略覆盖时，也允许该类别的无武器对象使用通用装填，以兼容脚本开火。局部策略搭配脚本指定未绑定武器时，应同时为对象绑定的武器提供策略。读档从保存的组件配置重建策略。原有四回调组件不会收到新回调，可以继续使用。

## 构建与测试

从引擎仓库根目录执行：

```powershell
python -m tools.cargo_run -- build --locked -p vera-reference-shield --target wasm32-unknown-unknown --release --features ares-ammo
```

按 [中文 SDK README](../../README.zh-CN.md) 将构建工具报告的 target 目录下 `wasm32-unknown-unknown/release/vera_reference_shield.wasm` 打包到新目录。格式仍是 `gameplay.wasm` 加 `vera-mod.json`。

需要同时启用过载时使用 `--features overload,ares-ammo`。兼容版标识 `vera.compat-shield-ammo`，过载版标识 `vera.overload-shield-ammo`。

新引擎验收模式 `ammo` 使用兼容护盾加弹药组件，报告文件必须为新文件，父目录已存在：

```powershell
vera20k.exe --mod-sdk-probe --retail-dir "原版游戏目录" --map XMP03T4.MAP --mode ammo --module "组件目录/gameplay.wasm" --output "报告目录/ammo.json"
```

该模式检查护盾伤害链、原版地图加载、实际命令与开火产生的 5→3→1 弹药变化、余量不足停止开火及存档后策略绑定。

```powershell
python -m tools.cargo_run -- test --locked -p vera-reference-shield --lib --features ares-ammo
```

设置 `VERA_SDK_AMMO_WASM` 为已打包组件路径，并按仓库要求配置原版 INI 与 `RA2_DIR` 后，运行实际 Wasm 集成测试：

```powershell
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_ammo_ -- --include-ignored
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_resource_policy_ -- --include-ignored
```

示例不含游戏资源，沿用已有 Rust 实现和原始 Ares 指令引用。其他桌面平台、真实联网及大规模性能仍需单独验证。
