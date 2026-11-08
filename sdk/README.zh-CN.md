# VERA Mod SDK：外置功能原型

## 当前开发状态

0.29 新增可选的 `weapon_firing_resource` 静态查询，参考组件实现位于独立插件工程的 `features/weapons/firing.rs`。宿主提供实际绑定武器的名称、`Anim` 数量、`IsSonic` 和 `UseFireParticles`；插件返回普通伤害开关与方向数量。资源接纳时一次查询，读档后从精确组件和冻结配置重建，开火时读取同一个 `WeaponType` 的不可变缓存，不逐发调用 Wasm。

安装参考组件后直接使用 Ares `ApplyDamage=yes/no`，无需新增功能开关；未设置时普通武器保留直接伤害，声波或喷火粒子武器关闭直接伤害。缺失、空值、非法值沿用已有 nullable 布尔读取历史。`Anim` 按最大的完整二次幂方向数选择，不足八项取第一项；驻军覆盖和运输车回退仍走原所有者。`ApplyDamage` 仅选择 FireAt 的普通伤害入口，不改变独立的 AI 伤害估值或声波／粒子自身效果。

该增量为组件0.29／配置19／状态1，API v1 和 WIT 名称不变，仍需支持新查询的配套宿主。新枚举追加在末尾，保留已发布枚举的存档编号；旧组件不接收新查询，旧非空冻结配置不自动开启新功能。完整 Ares 兼容继续按插件工程台账推进。

配置来源使用宿主的 `$weapon-config` 目录与 `weapon-effective:` 标量读取服务，读取实际 Weapon 实例保留的物理层历史。同名 Projectile 后分配时不会覆盖该来源；读取仍委托共同 INI 转换器，源码、读取范围和执行预算由宿主管理。插件公共入口为 `ConfigView::retained_weapon_read`，不读取平面投影来替代武器身份。

0.28 已完成本阶段既有 Ares 子集的外置迁移，完整 Ares 兼容仍未完成。接口见[超武声明](api/src/superweapon.rs)。

0.28 将引擎已有的超武扩展接入 `features/superweapons/`。`configure_super_read` 在原规则读取时点处理首次访问和 `SW.Warhead` 引用；`superweapon_resource` 在配置结束后一次性提供挂载、使用条件、初始/虚拟充能、费用、通用弹头、单位投送及演出声明。作者直接使用既有 Ares INI 标签，无需额外启用标签。Host 共用原数值、列表和固定音频目录读取接口；计时、发射次数、钱包、两阶段选择和投送队列继续由原引擎所有者维护，Wasm 不逐帧改写这些状态。

此候选的配置版本为18、实体状态版本为1，API v1 与 WIT 名称保持不变，但两个新增接口需要配套宿主。旧组件未声明时沿用宿主模式；显式默认声明选择原版行为。存档恢复使用存档中的精确组件与冻结配置，原始组件及配套引擎应与旧存档一起保留。范围限于引擎原有的超武子集；完整 charge-drain、新超武类型和更多 AI 选点方式仍未由本次迁移实现。

0.27 新增纯启动声明 `sound_sample_admission`，在固定 SOUNDMD 目录首次读取前选择未索引样本的接纳方式。参考组件直接声明 true，合法散装 WAV 可沿原有样本选择与解码流程加载；无需额外 INI 开关。未声明沿用宿主模式，显式 false 在存在音频索引时要求样本已登记。声音目录、资源查找和解码器仍由宿主统一维护，资源重新接入继续使用原冻结目录。相同 INI 摘要不代表接纳模式相同，恢复仍依赖精确组件身份与固定资源配方。配置17／状态1／API v1／WIT不变，新声明需要配套宿主；未进行音频设备输出或跨平台实跑。

0.26 将普通弹头的 House 关系过滤与既有空爆边界接入 `match_rules.warhead_house_filter`／`airburst_tail`。参考组件直接声明启用前者，即使 `AresEMP=no` 也保留关系过滤；缺少有效弹头关系数据的启用声明会被宿主拒绝，不再静默回退。空爆选择 `unsupported_replacement`，仅保留既有未实现边界，不新增替代算法，父弹伤害仍正常执行。未声明沿用宿主模式，显式 false／`retail_fixed_nine` 可选择原版行为。实际组件的伤害、正式发射、保存恢复、原版与旧模式对照及真实地图加载检查通过。配置17／状态1／API v1／WIT不变，仍需配套宿主；没有新增窗口、音频设备、联网或跨平台实跑。

0.25 新增纯启动声明 `prism_support_defaults`，在第一次 Rules 读取前选择既有的棱镜支援倍率默认单位修复。参考组件直接返回 true，无需 INI 开关；未声明沿用宿主模式，显式 false 选择原版。存在 `[General]` 但缺少倍率键时，后续读取保留正确的百分比；数值解析、支援计数及发射仍归原所有者。实际组件的分层读取、保存恢复、延迟发射、旧模式和真实资源加载检查通过。配置17／状态1／API v1／WIT不变，但需要配套宿主；原生规则存档布局已变化，旧存档保留对应旧引擎和组件。本次不新增 PrismForwarding，未进行窗口或跨平台实跑。

0.24 新增独立的纯启动声明 `spawn_identity`，在正式资源接纳后批准 CnCNet `spawn.ini` 的国家编号及战役 Side 范围。参考组件直接声明扩展 Country 与 byte Side，不读取玩法配置或增加 INI 开关；未声明沿用宿主原模式，显式 false 可覆盖旧内置模式。输入先由唯一解析器读取一次，再于原 Ready 流程按固定组件策略批准；国家是否注册、地图、人数和颜色仍由现有所有者校验。战役 Side／CampaignID 仍是元数据，显式 Scenario 仍使用记录0。实际组件的人类／电脑国家、阵营和基地车、旧实现、资源重建及七种真实资源加载检查通过。社区测试覆盖帧解码和同一批准入口，未安装全局快照，也未实际启动窗口／联网；独立网络房间描述的国家范围仍由既有协议校验。配置版本仍为17、状态版本仍为1，API v1/WIT不变，仍需配套宿主。 接口见[启动声明](api/src/startup.rs)。

0.23 新增一次性 `type_sound_resource` 目录，接通 `Promote.VeteranSound`／`Promote.EliteSound`。插件沿实际 Techno 读取点保留继承全局、静音和已注册音效三态，宿主校验固定 SOUNDMD 来源及原生类型身份后一次绑定；军衔、位置音效和 EVA 仍由原机制维护。已注册的字面 `none` 是音效，静音仅关闭音效、不关闭 EVA。未声明保留旧模式，显式空目录在旧构建中也使用全局音效。不增加 INI 开关或逐次晋升 Wasm 调用。实际晋升到派发、存档后继续升级、来源变化拒绝、旧实现、共享 PCX GPU 及七种真实资源模式检查通过；未测试音频设备实际播放，仍需配套新版宿主。 接口见[类型音效目录](api/src/audio.rs)。

0.22 新增一次性 `cameo_resource` 目录，迁移普通、晋升及超级武器的 PCX 图标。插件按实际 Rules／固定 ART 读取点保留文件名，宿主继续共用原文件名读取器、PCX/SHP 解码和图集；晋升 PCX 缺失时先回退普通 PCX，再走既有 SHP 路径，完全透明的 PCX 仍保持选中。安装即声明，不增加 INI 开关或逐图标 Wasm 调用。未声明保留旧模式，显式空目录选择原版。实际组件 GPU、同组件存档恢复、旧实现、七种真实资源模式及上一版组件加载检查通过。正式图标绑定在场景完成并经过共同 TypeReset 后发生，中间配置暂存不代表原生 Super 扩展跨 ClearScene 存活；Ares 完整文档兼容仍未完成。 接口见[图标目录](api/src/cameo.rs)。

0.21 增加可选 `match_rules.anim_damage`，选择读取已有动画所属方／附着 Techno 所属方，以及优先使用 ART 的 Warhead。没有指定弹头时仍按自身 INVISO 身份选择 C4Warhead，否则使用 FlameDamage2。原引用分配、累积、计时、整数扣量和伤害接收继续共用；所属方不是攻击对象，也不从视觉颜色推导。安装组件即声明，不加 INI 开关或每帧 Wasm 调用。实际组件、无组件／旧组件／显式关闭、旧构建和针对性真实资源加载检查通过；完整组合验收留到整轮收口。不扩展 Weapon= 或 Damage.Delay=。

0.20 增加可选的 `match_rules.building_presentation`，分别选择建筑 SHP／MAKE 独立调色板组和 DestroyAnim 无自定义组时的所属方颜色回退。参考组件安装即声明，不增加 INI 开关。ART 历史读取、调色板解析、动画状态与渲染仍由原引擎维护，正式组件绑定后在对象构造前重接同一资源目录。未声明保留原模式，显式 false 选择原版行为。实际销毁与保存恢复、索引 GPU 绘制、旧实现和真实资源加载对照通过；不扩展 bib、活动／超武动画、voxel 调色板或动画伤害归属。

0.19 增加独立的纯启动声明 `long_list_readers`，在冷启动读取前选定 House 的 `Allies` 与 Techno 的 `Prerequisite`／`PrerequisiteOverride` 容量。安装 Ares 组件即使用2048，不增加 INI 开关；显式 `Retail128` 覆盖旧内置模式，未声明则沿用原模式。General 六类前置仍为128，物理 INI 行仍受原511字节限制。实际组件、定向联盟、保存恢复、资源管理器重建、旧实现及真实资源加载检查通过。新增资源序列化字段沿现有保存格式识别，不承诺旧完整存档自动迁移。

保留交付为 `plugin-dev/dist/sdk-feature-lab-0.7.0`。当前源码是 **0.32.0 开发候选**，配置版本 **22**、状态版本 **1**；下文的 0.7.0 说明对应保留交付，不能据此把开发候选当成已经交付的新版本。API v1 与 WIT 包名不变，但新能力仍要求配套宿主。

0.32 在同一 `projectile_resource` 中增加可选散布上下界。插件独立领域通过 `projectile-effective:ares-nullable-leptons` 读取 `BallisticScatter.Min/Max`，只有原生解析成功才成为有效覆盖，0 与未配置严格区分。普通 FireAt 的 plain／flak 分别沿原调用点取全局缺省，随机流、距离／角度计算、飞行、爆炸和恢复继续归原有所有者。`Inviso + FlakScatter` 的额外放置仍用原全局值，不套入新字段；历史缺域保持旧行为，无新 Hook 或逐帧 WASM 调用。完整 Ares 兼容仍见插件兼容台账。

0.30 在既有不可变策略上增加 `match_rules.projectile_fuel` 和 `weapon_firing_resource.projectile_range`（有符号 lepton）。独立的 `weapons/projectile_range` 域通过 `weapon-effective:ares-leptons` 读取实际到达的物理 Weapon 节。普通 FireAt 使用武器预算；裸构造和原版碎片弹保留 INT_MAX。引擎唯一的 ProjectileStore 保存、扣除并校验剩余三维路程，耗尽进入原引爆尾，不逐帧调用 WASM。`Ranged=no`、无插件及旧组件保持原路径。0.31普通Ares Airburst／Splits子弹工厂也已接入这一预算；Firestorm与其它专门创建器仍是独立残余。旧完整存档请使用原配套引擎与组件，布局校验不绕过。

阵营增量将 Side／Country 的初始化、一次父国继承和静态声明交给 `features/factions`，原引擎继续执行乘员、伪装、AI 防御与阵营条件、空降／降落伞和初始军衔。独立的 `configure_country_read` 在真实 Country 读取前选择原版短列表或 2048 容量的只查找读取器；`configure_faction_pass` 只处理实际发生的 Country-only 读取。已完成的步骤逐一发布同一 Program 检查点，独立启动投影只完成自己的读取记录，保留真实进程的配置历史。`faction_resource` 按实际类别内编号和 stored ID 查询冻结策略，引用保留未设置、清空和已解析三态。接口见 [阵营声明](api/src/faction.rs)。

新对局默认启用 `AresFaction`，可在 `[VeraFeatures]` 中显式关闭。开关在实际 Country 前读时读取；先前发生的类型分配和父国尝试继续保留。`eva_catalog` 在规则读取前确定固定 EVA 目录布局，与该开关分别冻结。本批已通过真实 Wasm 与存档、关联消费者、正式应用加载、旧内置模式、七种零售资源模式及上一版组件加载检查。0.17 继续接入菜单／旗帜、早晚加载画面与加载音乐；0.18 接入 EVA SetSide、侧栏、消息颜色、胜负文字与结算资源；Ares 完整文档兼容仍未完成。组件摘要及不透明配置继续参与 SDK 规则校验，包括其中的表现字段；旧完整存档保留对应引擎和组件。

0.17 的菜单与加载快照一次读取同一实时 Program，保留国家编号、重复 stored ID 和显式原版选择。ScenarioClear 保留尚未销毁的 Side／Country 继承记录，真正 TypeReset 才清除。独立可选 `loading_theme_resource` 从原 root Side→Country 或已解析战役 Basic 读取有界名称，由原 Theme 模块播放；要求冷 Rules 前接纳及 `configure_boundary`，不强制实现阵营或 Country 前读。受限查询通过共享 Host 的原读取器，避免原始文本存在性探测。实际组件、关联消费者和旧实现对照已通过，七种真实资源模式及上一版组件加载也已通过；未验证完整窗口／设备输出。接口见[加载音乐声明](api/src/loading.rs)。

类型转换增量在现有类型资源中增加可选的转换目标、带符号的转换专用炮塔转速，并以对局策略选择 Script68。目标引用保留真实读取点的共同 Techno 注册顺序；原 Team VM 和同对象转换所有者继续维护编号、队伍、生命值、弹药、采矿与运动状态。新配置默认启用 `AresTypeConversion`，显式关闭时 Script68 保持原版不执行行为。本增量已通过关联行为、旧实现对照、正式资源恢复和七种真实资源加载检查。普通引擎接纳原有的停用矿车枚举分支后，完整存档布局随之变化；旧存档保留对应旧引擎和组件。

探照灯增量在现有类型资源中增加可选声明，包含能力覆盖、类型规则选择、高度／距离、朝向附件和颜色通道。原版建筑灯和扩展附着灯共用原有状态、生命周期和绘制链；本增量已通过关联行为、旧实现对照、应用资源加载、现有 GPU 像素对照和七种真实资源加载检查。关闭扩展时使用实际物理读取保留的原版建筑开关；文本 INI 入库前已去掉首尾空白，底层布尔读取器的区别不能直接当作物理文件的空白差异。原有绘制残差继续保留。

EMP 增量增加可选 `emp_resource`，按全局、四类对象和弹头发布冻结规则。资源名明确区分继承、清空和已注册名称。组件解释配置，原引擎继续拥有倒计时、任务、触发器顺序和动画引用；本增量已通过关联行为、旧实现对照、正式读档和七种真实资源加载检查，不扩展原有不支持的受击路径。触发器冻结策略进入完整存档布局，旧存档保留对应旧引擎和组件。

生产增量增加 `production_resource`，发布冻结的前置组、类型资格和现有生产所有者的策略。可选 `configure_boundary` 接收同一个原生 Rules 所有者记录的 ScenarioClear／TypeReset 与真实物理读取历史；仅相同内容和清单的组件可以继承不透明配置。读档直接采用保存的 Program，不重跑配置。本增量的生产关联回归、旧实现对照、正式读档和七种真实资源加载方式已通过，Ares 完整文档兼容仍未完成。国家来源和永久计划字段进入普通引擎后，会改变自动计算的完整存档布局；旧存档应保留对应旧引擎和组件。

护甲增量新增可选的 `armor_resource`，提供目录、按类别区分的对象与地形绑定，以及弹头交互表。新配置的 `AresArmor` 默认启用，历史冻结配置缺失该开关时仍关闭。宿主的伤害、选武器和目标准入继续共用原有交互查询；数值通过 `verse-token` 调用原解析器，不在插件复制百分比或 x87 算法。本增量的护甲及关联消费者、真实资源加载检查已通过；Ares 完整文档兼容仍未完成，也未替换稳定交付。接口见 [护甲资源](api/src/armor.rs)。

开发候选增加 `match_rules`、成对的 `effect_resource` / `effects`，以及类型上的生存贡献与隐形许可。生存与附加效果在原四项基础上默认启用，相关开关是 `AresKeepAlive` 和 `AresAttachEffect`。历史非空冻结配置不自动获得新功能；护盾和过载仍需显式开启。

附加效果由插件解释配置、决定有序的添加、刷新、到期和移除；引擎原有机制保留唯一实例列表、动画引用、倍率、永久箱子奖励和隐形时钟。附加效果回调不覆盖护盾的不透明对象状态。接口见 [对局策略](api/src/match_rules.rs) 和 [效果事务](api/src/effects.rs)。

手工地图与随机生成地图均在最终 Rules 处理后、对象构造前绑定同一冻结组件；读档采用保存的原组件和配置，不重跑 Configure，并通过共用资源闭包绑定动画。这里没有提供地图生成器扩展回调。相关生产路径已通过关联检查，尚不表示引擎已有的 Ares 能力已全部外置。

宿主新增的序列化字段也改变完整世界存档布局。旧存档应保留对应旧引擎、旧组件及声明；组件状态版本相同不能自动迁移宿主存档。

本目录保留 VERA20K 外置插件的 API、WIT 契约和运行时；插件实现与开发工具统一迁到相邻的 [plugin-dev 独立工程](../../plugin-dev/README.md)。第一份参考插件位于其中的 `plugins/vera20k-ares`，发布文件名为 `vera20k-ares.wasm`。一个作品加载一个可选的外置功能组件；当前参考组件0.7.0包含护盾、自定义过载及Ares武器弹药成本、装填数量、低弹量武器选择、手动操作限制，通过INI配置，无须按组合重新编译。新对局安装组件后，四个已外置Ares模块默认启用，护盾和过载仍须显式开启。

护盾的配置解释、状态、伤害处理和恢复逻辑已经移到外置 Rust 插件中。作者可以修改插件源码、重新编译发布，无须为这两种护盾分别修改或编译引擎。INI 仍用于配置，功能实现位于插件代码中。

这是实验版接口，目前已外置原有护盾链，并增加一组Ares弹药生命周期功能。其他已有的Ares/Phobos扩展仍按各自构建开关保留，尚未完成整体外置化，也不代表完整兼容Ares或Phobos。

英文文档保持独立，见 [README.md](README.md)。

[Ares弹药与武器选择说明](../../plugin-dev/plugins/vera20k-ares/AMMO.zh-CN.md)介绍三个独立开关：`AresAmmo`解释武器成本，`AresReload`解释普通/空仓装填量，`AresNoAmmoWeapon`解释低弹量选用的武器。可选的`weapon_resource`和`type_resource`在加载时提供静态策略，复用引擎已有开火、弹仓、装填时钟与武器选择；原有四个回调仍是基础要求。

`AresManualControl` 接入 `NoManualFire`、`NoManualUnload`、`NoManualEnter`、`NoSelfGuardArea`，通过类型上的可选 `InputPolicy` 供已有鼠标与按键操作流程消费；不在模拟层过滤 AI、脚本或回放命令。模块默认启用不改变四个标签自身默认`no`，隧道与混合/多选治疗步兵仍有未外置部分。见[手动操作说明](../../plugin-dev/plugins/vera20k-ares/MANUAL.zh-CN.md)。配置版本5和状态版本1沿用0.5.0的结构。

0.6.0 的低弹药选择保留完整武器索引，建筑动画延迟、射击、存档和状态校验继续使用宿主原有所有者。配套宿主保留旧主/副武器的存档字节与哈希，支持有效0..17槽；索引越界时安全地返回无武器。它不改变 SDK/WIT 消息结构，但高槽延迟射击需要配套新版引擎。

Ares 完整公开功能目录、INI 键索引与后续开发拆分见 [Ares 功能中文开发参考](../docs/zh-CN/ares-feature-guide.md)。其中的 SDK 接入描述为开发建议，当前已外置范围仍以上文为准。

Phobos 的功能与修复目录、INI 键索引、稳定/开发文档对照及 SDK 实现建议见 [Phobos 功能中文开发参考](../docs/zh-CN/phobos-feature-guide.md)。目录还区分了原生 DLL 互操作接口与本 SDK，以及护盾示例的已完成范围和待补能力。

发布文件名和 Cargo 包名为 `vera20k-ares`，内部组件标识仍为 `vera.reference-gameplay` 0.7.0，config_version=5、state_version=1，API v1及WIT不变。本版只改变新对局初始功能选择，不迁移旧冻结配置；Phobos兼容护盾与自定义过载仍作为现有示例保留，文件名不表示完整兼容Ares。

后续插件代码在 `plugin-dev` 中维护，按[独立构建指南](../../plugin-dev/sdk/dev-kit/README.zh-CN.md)使用标准 Cargo 完成测试、编译和打包。引擎仓库中的 `sdk/api` 和 `sdk/wit` 是接口权威定义，独立工程使用固定版本的 SDK 开发快照；协议扩展先更新定义及引擎支持，再同步快照。插件构建无需引擎源码、可执行文件、Python 或游戏资源；引擎构建和验收继续在引擎仓库运行 `tools.cargo_run`。

## 同一个引擎，三种运行方式

| 方式 | 作品提供的内容 | 行为 |
| --- | --- | --- |
| 原版 | 原版资源，不声明外置组件 | 不加载 SDK 护盾 |
| 兼容护盾 | 原版资源、参考组件，INI 开启 Shield | 外置组件实现当前支持的护盾逻辑 |
| 过载护盾 | 同一个参考组件，INI 开启 Shield 和 Overload | 在兼容护盾基础上加入吸收伤害后的过载机制 |

验收示例使用生命值为 300 的原版 `MTNK`、100 点护盾，连续施加三次 60 点伤害。三种方式使用同一个引擎可执行文件：

| 方式 | 初始生命值 | 第一次命中后 | 第二次命中后 | 第三次命中后 |
| --- | ---: | ---: | ---: | ---: |
| 原版 | 300 | 240 | 180 | 120 |
| 兼容护盾 | 300 | 300 | 280 | 220 |
| 过载护盾 | 300 | 300 | 240 | 180 |

兼容护盾先抵挡 100 点伤害，剩余伤害进入引擎原有处理流程。过载护盾第一次吸收 60 点伤害后达到阈值，在过载期间暂时停止吸收，所以后两次命中直接扣除生命值。

## 目录与职责

| 位置 | 用途 |
| --- | --- |
| [api/src/lib.rs](api/src/lib.rs) | 请求、返回计划、版本声明、状态大小限制与通用显示数据 |
| [api/src/resource.rs](api/src/resource.rs) | 武器成本、类型装填量与低弹量武器选择的公共静态策略 |
| [api/src/timer.rs](api/src/timer.rs) | 引擎与插件共用的倒计时实现 |
| [wit/gameplay.wit](wit/gameplay.wit) | `vera:mod-sdk@0.1.0` 组件接口 |
| [runtime](runtime/README.md) | Wasmtime 运行时、组件缓存、执行限制与宿主服务 |
| [独立工程的打包工具](../../plugin-dev/sdk/package/src/main.rs) | 离线打包工具 `vera-mod-pack`，不再属于引擎 workspace |
| [../../plugin-dev/plugins/vera20k-ares/src/lib.rs](../../plugin-dev/plugins/vera20k-ares/src/lib.rs) | WIT 导出、消息解码和回调名称核对 |
| [../../plugin-dev/plugins/vera20k-ares/src/component.rs](../../plugin-dev/plugins/vera20k-ares/src/component.rs) | 组件声明、配置封装、固定顺序的功能分派 |
| [../../plugin-dev/plugins/vera20k-ares/src/features/defense/shield](../../plugin-dev/plugins/vera20k-ares/src/features/defense/shield/mod.rs) | 护盾配置绑定、状态、伤害、恢复和过载逻辑 |
| [../../plugin-dev/plugins/vera20k-ares/src/features/weapons/ammo_cost.rs](../../plugin-dev/plugins/vera20k-ares/src/features/weapons/ammo_cost.rs) | 外置 Ares 武器弹药策略 |
| [../../plugin-dev/plugins/vera20k-ares/src/features/weapons/reload.rs](../../plugin-dev/plugins/vera20k-ares/src/features/weapons/reload.rs) | 外置Ares普通/空仓装填数量配置 |
| [../../plugin-dev/plugins/vera20k-ares/src/features/weapons/empty_weapon.rs](../../plugin-dev/plugins/vera20k-ares/src/features/weapons/empty_weapon.rs) | 外置Ares低弹量武器选择配置 |
| [../../plugin-dev/plugins/vera20k-ares/src/support](../../plugin-dev/plugins/vera20k-ares/src/support/mod.rs) | 通用宿主访问与浮点位编码 |
| [../src/mod_sdk.rs](../src/mod_sdk.rs) | 保持现有调用路径的引擎入口，不承载功能算法 |
| [../src/mod_sdk/program.rs](../src/mod_sdk/program.rs) | 组件身份、冻结配置、执行缓存和存档校验 |
| [../src/mod_sdk/assets.rs](../src/mod_sdk/assets.rs) | 作品声明接纳与读档绑定 |
| [../src/mod_sdk/host.rs](../src/mod_sdk/host.rs) | 原有 INI 读取器、类型目录与伤害计算服务 |
| [../src/mod_sdk/simulation.rs](../src/mod_sdk/simulation.rs) | 构造、伤害、更新阶段的数据转换和计划提交 |
| [../src/mod_sdk/acceptance.rs](../src/mod_sdk/acceptance.rs) | 使用正式引擎路径的命令行验收入口 |

引擎负责对象生命周期、真实生命值、原有伤害流程，以及校验并提交插件返回的计划。插件负责自己的配置、护盾值、计时器和算法，通过通用进度条与染色数据表达显示效果。

更详细的边界、数据所有权和添加功能步骤见 [架构与扩展指南](ARCHITECTURE.zh-CN.md)。

当前按“一个作品、一个功能组件”设计。一个组件内部可以包含多个功能；本版没有多插件依赖解析、加载顺序和冲突合并系统。

## 接口如何工作

插件采用 WebAssembly Component Model。引擎通过 `evaluate` 传入类型化消息，插件返回变更计划。宿主可提供只读配置、原有 INI 类型读取器、类型目录与共用伤害计算函数。

实验版 SDK v1 要求组件声明以下四个回调；没有需要处理的行为时返回未修改的计划。

| 回调 | 触发时机 | 插件承担的工作 |
| --- | --- | --- |
| `configure` | 每次实际到达的 Rules 配置读取阶段，以及最终合并阶段 | 解释插件配置；最后利用完整对象和弹头目录建立绑定 |
| `attach` | 游戏对象构造时 | 为适用对象建立插件状态 |
| `damage` | 目标进入伤害接收流程时 | 更新自身状态，返回交给引擎继续处理的目标伤害值 |
| `tick` | 对象进入对应更新阶段时 | 推进恢复、过载等状态，更新通用显示数据 |

武器与类型资源使用以下可选静态查询：

| 回调 | 查询时机 | 返回策略 |
| --- | --- | --- |
| `weapon_resource` | 加载时按实际登记武器查询，读档后从冻结配置重建 | 每次开火成本、是否请求装填、适用类别、建筑是否采用普通武器ROF |
| `weapon_firing_resource` | 加载时按实际绑定武器查询，读档后从冻结配置重建 | 普通伤害开关、开火动画方向数量；旧 `weapon_resource` 消息结构保持不变 |
| `type_resource` | 加载时按对象类型与实际类别查询，读档后从冻结配置重建 | 普通/空仓装填量、低弹量武器选择及手动输入策略 |

查询消息和策略统一定义于`api/src/resource.rs`，经已有`evaluate`导出传递，没有增加Host服务。正式模拟路径读取不可变投影，不占用额外对象插件状态，也不新增一套弹药或计时器。

规则按实际加载阶段传入，随后执行一次最终合并回调。对局期间配置冻结，不继续修改。护盾配置键由插件解释，引擎不需要认识 `ShieldType` 或 `VeraOverload`。

插件状态由插件序列化成字节并返回，由宿主管理保存。不要依赖 Wasm 全局可变变量跨回调保留状态：每次调用都会创建新的实例。返回空的对象状态表示取消该对象的后续订阅；无状态但需要持续回调的功能可以保留一个字节的标记。

`damage` 可以修改当前接收目标的伤害输入及自身插件状态。当前引擎拒绝所有预留的 `Operation` 操作，包括直接改生命值的操作；不能借此绕过死亡流程，也不能直接写入另一个对象的生命值。

## 通过 INI 选择功能

0.7.0新对局加载组件后，`AresAmmo`、`AresReload`、`AresNoAmmoWeapon`、`AresManualControl`默认启用，直接使用原Ares标签即可。Phobos护盾和自定义过载仍默认关闭；例如要在默认Ares功能上加护盾，把下列开关放在作品已经加载的Rules INI或地图Rules配置层中，不需要新增JSON功能清单：

```ini
[VeraFeatures]
Shield=yes
Overload=no
```

需要隔离对照时，可分别显式关闭Ares模块：

```ini
[VeraFeatures]
AresAmmo=no
AresReload=no
AresNoAmmoWeapon=no
AresManualControl=no
```

这些是VERA自有的选择开关，原Ares标签本身的默认值不变。缺失、空值和无效布尔值保留前一规则层的有效选择；显式`no`不会因后层缺失而重新启用。Overload必须同时开启Shield，否则最终加载会明确报错。只在最终阶段检查依赖，允许先定义、后开启。四个Ares开关彼此独立，关闭一项不代表其余安装修复也关闭；若宿主还启用了内置`ares`构建特性，其行为须单独区分。

`Shield`控制对象护盾订阅，`Overload`控制自定义过载行为。`AresAmmo`控制武器节`Ammo`成本及已接入的建筑ROF安装修复，`AresReload`控制对象类型节`ReloadAmount/EmptyReloadAmount`，`AresNoAmmoWeapon`控制`NoAmmoWeapon/NoAmmoAmount`，`AresManualControl`控制四个手动操作限制标签的解释。关闭某项就不发布该项外置策略；关闭Shield时不创建护盾状态和护盾条。原版对象类型节中的`Ammo`容量、`InitialAmmo`和装填间隔仍由引擎已有读取器处理。

组合示例见[resource-cycle.ini](../../plugin-dev/plugins/vera20k-ares/examples/resource-cycle.ini)：每发成本2，普通装填2、空仓装填3，弹量不高于阈值时选副武器。若副武器需要在空仓继续开火，须保持AresAmmo启用（新对局默认值）并将该武器成本设为0；选择到副武器后仍接受正常开火条件检查。飞机不应用武器成本与装填量扩展，但可以使用低弹量选择策略。完整默认值、原始指令差异及适用边界见[Ares功能说明](../../plugin-dev/plugins/vera20k-ares/AMMO.zh-CN.md)。

每次实际 Rules 层都读取配置，地图层也能覆盖。最终选择随配置冻结，进入存档和联机校验。对局中不重新读取 INI；读档使用保存的选择，修改磁盘上的开关仅影响新对局。修改算法或增加功能仍需重新编译插件。

完整开关示例见[features.ini](../../plugin-dev/plugins/vera20k-ares/examples/features.ini)。多个示例合并时，保留一个`[VeraFeatures]`节，合并所需的显式选择，避免重复节头；四个Ares模块不必重复填写`yes`。

## 护盾配置示例

把下面内容作为作品的 Rules 或地图配置层加载；完整示例见 [shield.ini](../../plugin-dev/plugins/vera20k-ares/examples/shield.ini)。SDK 声明文件本身不负责自动加载额外 INI。

```ini
[VeraFeatures]
Shield=yes
Overload=no
AresAmmo=no
AresReload=no
AresNoAmmoWeapon=no
AresManualControl=no

[ShieldTypes]
0=SDKShield

[SDKShield]
Strength=100
Armor=none
SelfHealing=0.25
SelfHealing.Rate=0.1
Respawn=0.5
Respawn.Rate=0.1
Pips=16,16,16

[MTNK]
ShieldType=SDKShield

[VeraOverload]
HeatLimit=60
Duration=75
Cooling=1
```

`Overload=no` 时忽略 `[VeraOverload]` 参数。INI 中设为 `Overload=yes` 后，同一个组件增加以下行为：

- 每次命中按实际消耗的护盾点数增加热量。
- 热量达到 `HeatLimit` 后，停止吸收伤害与恢复，持续 `Duration` 个模拟帧。
- 每次符合更新条件的对象 AI 调用按 `Cooling` 降低热量；它不是按现实秒计量。
- 过载期间，恢复、重生及各自战斗延迟共四个倒计时保持暂停。再次命中可以重设倒计时，但不会提前解除暂停。
- 过载结束后，在符合条件的对象 AI 更新中恢复计时，再应用原有供电等规则；Temporal、Chrono 和 limbo 的暂停或跳过语义继续生效。

这部分是 SDK 作者扩展示例，不属于对上游 Ares/Phobos 功能的兼容承诺。修改玩法时从 `logic.rs` 入手；添加配置时同步调整 `features/defense/shield/config.rs` 或 `features/defense/shield/mod.rs` 中对应读取代码。

## 独立开发与引擎构建

插件实现、打包工具和各自测试在相邻的 `plugin-dev` 工程维护。以下 PowerShell 示例从引擎仓库根目录开始，先进入独立工程；插件命令使用标准 Cargo：

```powershell
Set-Location ../plugin-dev
cargo test --locked -p vera20k-ares -p vera-mod-pack --lib
cargo build --locked --release --target wasm32-unknown-unknown -p vera20k-ares
New-Item -ItemType Directory -Force -Path dist | Out-Null
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/reference-new --program vera20k-ares.wasm
```

输出子目录 `dist/reference-new` 必须是新目录。Cargo 生成原始 core 文件 `vera20k_ares.wasm`，打包器生成 `vera20k-ares.wasm` 与匹配的 `vera-mod.json`。设置了 `CARGO_TARGET_DIR` 时，打包输入路径须相应调整。安装目标、独立依赖和日常开发步骤见[插件开发指南](../../plugin-dev/sdk/dev-kit/README.zh-CN.md)。

功能仍由作品加载的 INI 选择，无须为过载或弹药组合分别构建。使用 `wasm32-unknown-unknown` 保持当前接口；宿主只链接 SDK 服务，不提供 WASI 文件、时钟、随机数或网络服务。

需要构建引擎时，返回引擎仓库并使用其构建工具：

```powershell
Set-Location ../engine
python -m tools.cargo_run -- build --locked --profile fast --no-default-features --bin vera20k
$engine = python -m tools.cargo_run --resolve vera20k --profile fast
if ($LASTEXITCODE -ne 0) { throw "无法定位引擎" }
$component = Join-Path (Resolve-Path ../plugin-dev) "dist/reference-new/vera20k-ares.wasm"
$outputParent = "已存在的验收报告目录"
```

各模式复用这份引擎和同一参考组件。此引擎构建关闭默认开发界面功能，也未启用额外的 `phobos` 构建特性。已有兼容引擎可直接用于运行和验收，不必因插件算法修改而重新构建。

## 作品如何发布

把打包工具生成的两个文件作为作品资源发布，并保持声明中的资源路径可被作品资源系统解析：

```text
作品资源根目录/
├── vera-mod.json
├── vera20k-ares.wasm
└── ……作品自己的规则、地图和美术资源
```

生成的 `vera-mod.json` 形式如下；实际摘要由工具计算，不要保留占位内容：

```json
{
  "api_version": 1,
  "program": "vera20k-ares.wasm",
  "sha256": "<vera20k-ares.wasm 的小写 SHA-256 摘要>"
}
```

这两个文件可以沿用社区客户端现有的作品内存资源包路径。引擎仍会校验声明、内容摘要、插件自身接口声明及允许的导入。没有 SDK 声明的作品走原版路径；声明错误或组件不兼容会明确报错。

本 SDK 提供功能扩展能力。VXL、SHP、INI 等资源的打包、加载和防提取属于另外的机制，本 SDK 不提供素材保护保证。

## 如何验收

如果使用包含 `verify.py` 的本地验收包，在验收包目录执行：

```powershell
python verify.py --game-dir "D:/steam/steamapps/common/Command & Conquer Red Alert II" --map XMP03T4.MAP
```

新版脚本使用同一个引擎和同一个组件依次运行无组件、全关闭、仅弹药、兼容护盾、过载、护盾加弹药、三功能组合和原版资源加载检查，并把报告及测试存档写入新目录。原版安装目录只读。验收入口在创建窗口前运行，属于机制测试，不是新的大厅或游戏启动界面。

从源码构建后，也可以直接调用引擎。以下继续使用前文的 `$engine` 和 `$outputParent`，报告目录必须存在、报告文件必须是新文件；工作目录与输出目录应放在原版安装目录之外：

```powershell
$retail = "D:/steam/steamapps/common/Command & Conquer Red Alert II"
& $engine --mod-sdk-probe --retail-dir $retail --map XMP03T4.MAP --mode none --output (Join-Path $outputParent "none.json")
& $engine --mod-sdk-probe --retail-dir $retail --map XMP03T4.MAP --mode compat --module $component --output (Join-Path $outputParent "compat.json")
& $engine --mod-sdk-probe --retail-dir $retail --map XMP03T4.MAP --mode overload --module $component --output (Join-Path $outputParent "overload.json")
& $engine --game-data-checkfast $retail XMP03T4.MAP (Join-Path $outputParent "retail.json")
```

`none` 不接受组件，其他模式提供同一参考组件。可用模式为 `disabled`、`compat`、`overload`、`ammo`、`ammo-only` 和 `all`；验收器只编写不同的测试 INI，并通过原版资源加载、正式对象构造、伤害接收、帧推进和联机校验值计算路径完成验证。存档部分覆盖带资源记录的存档接纳、插件状态保留、恢复后继续推进，以及读档后新单位的插件绑定。

修改插件后，先在 `plugin-dev` 中运行上文的插件与打包工具单元测试，再回到引擎仓库使用实际 Wasm 和原版资源验收。单元测试通过不能替代真实组件加载测试。

功能选择集成测试使用实际 Wasm：设置 `VERA_SDK_REFERENCE_WASM`、`VERA_SDK_COMPAT_WASM` 和 `VERA_SDK_AMMO_WASM` 为同一个独立工程产出的组件路径（如上文 `$component`），按仓库要求配置原版 INI、`RA2_DIR`、`VERA20K_REQUIRE_RETAIL_INI=1` 和 `VERA20K_REQUIRE_RETAIL_ASSETS=1` 后执行。后一个开关确保缺少游戏档案时明确失败，避免实际对局测试被跳过：

```powershell
python -m tools.cargo_run -- test --locked -p vera20k-core --lib sdk_ -- --include-ignored --skip sdk_refactor_preserves_
```

`COMPAT`、`AMMO`是历史测试变量名；测试INI应明确区分默认Ares组合、显式关闭对照和显式开启的护盾/过载。验证0.7.0默认安装时，应省略四个Ares的`yes`，检查原Ares标签及已接入的无标签修复；隔离模式须显式关闭不测的模块。目录或包装迁移时，可分别设置`VERA_SDK_BASELINE_WASM`和`VERA_SDK_CANDIDATE_WASM`为迁移前后的组件，再运行`sdk_refactor_preserves_`，比较同一接口与配置版本的manifest、冻结配置和状态字节。该检查仅用于声明、接口和配置/状态布局保持相同的纯移动或重构；0.6.0行为扩展和0.7.0新对局默认变化都不能用跨版本冻结配置相等来验收，也不表示支持跨配置版本迁移。新增机制的验证应覆盖实际WASM、正式选武器/装填路径与存读档，不能只检查返回的策略字段。

## 状态、存档与联机一致性

- 组件内容、摘要、接口声明、冻结后的配置和每个对象的插件状态随世界保存；编译后的机器码缓存不写入存档。
- 读档要求当前作品提供与存档摘要完全相同的组件，再使用保存的原组件字节与冻结配置；即使存档含有Wasm字节，也不能用新版资源替代。旧非空配置不补入新默认，不重读当前INI。更新组件需要开始新对局；保留原组件及声明供旧存档使用。本版不提供状态格式迁移或运行中热替换。
- 组件身份、配置与对象状态进入诊断状态哈希和实际联机 `FRAMEINFO` 校验。原有校验收集中的两次 Scenario 随机数抽取保持不变。
- 进度条、染色等显示投影由权威状态派生，不作为独立权威状态参与哈希。预览与提交使用同一插件处理入口；预览不发布状态或投影，也不消耗模拟随机数。
- 公共数值计算继续调用引擎已有实现。浮点配置通过明确的 IEEE-754 位模式传递，避免另写一套伤害计算算法。
- 插件超限、执行失败或返回非法计划时，拒绝计划并阻止继续接纳该帧；不会悄悄卸载插件后按原版继续运行。带故障状态的存档同样拒绝恢复。

## 执行限制与当前边界

每次调用使用新的 Store 和组件实例，已编译组件可缓存。当前默认限制见 [运行时源码](runtime/src/lib.rs) 和 [API 定义](api/src/lib.rs)：

| 项目 | 当前默认值 |
| --- | ---: |
| 单次调用执行额度 | 5,000,000 fuel |
| 实例线性内存上限 | 32 MiB |
| 组件文件上限 | 16 MiB |
| 单次调用宿主服务次数上限 | 65,536 |
| 单对象插件状态上限 | 16 KiB |
| 插件配置上限 | 4 MiB |
| 消息上限 | 8 MiB |

fuel 衡量客体指令执行量，不代表固定毫秒，也不限制任意宿主 Rust 回调工作量或组件编译时间。没有链接 WASI 服务也不代表完整的安全审计。

当前还需要明确以下范围：

- 支持预制地图。声明外置组件的作品进入随机地图生成路径时会明确拒绝；本版没有生成器扩展接口。
- 护盾保留当前已实现范围。待机、命中、破盾和重生动画及武器效果、弹头附加/移除/破盾操作、类型转换时的护盾转移、`SelfHealing.EnabledBy` 消费逻辑，以及可配置的其他 `Pips.Background` 资源尚未补齐。详见 [参考插件说明](../../plugin-dev/plugins/vera20k-ares/README.md)。
- 面向 Windows、Linux、macOS 的原生桌面架构；目前本机实际构建与验收平台为 Windows，其他平台仍需构建和运行验证。
- 已验证实际联机校验算法和存档资源接纳路径；尚未完成真实网络传输对局、完整图形界面读档及常驻资源重新绑定验收。
- 尚未完成两万单位性能验收。逐次新建实例和重复解码整份配置是第一版隔离原型的实现选择，后续应先测量开销，再设计批量回调及不可变配置驻留。

## 继续开发与授权

建议先在统一参考组件的对应功能模块中修改玩法，保持宿主只提供通用能力。新增功能如果超出现有六种回调和数据边界，应先设计通用接口及其状态、存档、联机一致性规则，再接入引擎，避免把某一个作品的专用规则写回宿主。

参考护盾代码迁自 VERA20K 已有 Rust 实现，保留对应来源引用及贡献历史，继续遵循仓库的 [GPL 授权](../LICENSE-GPL)。本示例不包含 Ares/Phobos C++ 源码或心灵终结素材；运行原版资源验收需要自行提供游戏文件。

0.31 增加可选 `projectile_resource`（HookKind 29，旧编号不变），由 `features/projectiles/airburst` 冻结普通 Airburst／Splits 策略。字段沿用 Ares 的 `AirburstSpread`、`RetargetAccuracy`、`Splits`、`RetargetSelf`、`AroundTarget`，通过 `$projectile-config`／`projectile-effective:` 读取实际 Projectile 的物理历史；新旧 Weapon/Projectile 同名来源相互隔离。宿主选择 `AresReplacement` 后继续使用原有对象数组、Scenario RNG、弹体创建、飞行和引爆清理；无逐帧 WASM 和第二套弹体状态。历史非空缺域保持旧选择。原生不终止的圆枚举输入，以及会空 Weapon 解引用的 Inviso+FlakScatter 子弹组合，宿主会明确报错；不会用裁剪或传入工厂武器掩盖这些边界。其它专门爆炸入口和完整 Ares 兼容仍需继续验收。
