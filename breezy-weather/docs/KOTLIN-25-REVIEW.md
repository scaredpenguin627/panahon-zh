# Kotlin 2.5.0-RC 升级评审

评审对象：分支 `kotlin-25-eap-rewrite`，相对 `origin/main` 的 7 个提交。
评审人：lead。规则依据：[Kotlin 2.5.0-RC what's new](https://kotlinlang.org/docs/whatsnew-eap.html)、
[The Companions to Come](https://blog.jetbrains.com/kotlin/2026/09/the-companions-to-come/)，
以及 fable 5.2（effort max）给出的逐文件升级规则清单。

## 结论

**通过，附带 3 条已知未验证项。** 编译、R8、签名、真机运行四关都过了；单元测试因网络无法执行。

## 工具链

| 项 | 值 | 核对方式 |
| --- | --- | --- |
| Kotlin | 2.5.0-RC | Maven Central `release` 版本 |
| Gradle | 9.9.2 | services.gradle.org `current` |
| JVM target | 21 | `AndroidConfig.kt`，AGP 9.5.1 + D8 的实际上限 |
| KSP | 2.3.14 | 最新发布版，Hilt/Room 代码生成通过 |
| 编译器开关 | `-Xcompanion-blocks`、`-opt-in=kotlin.ExperimentalStdlibApi` | `ProjectExtensions.kt` |

没有使用 `-Xcompanion-blocks-and-extensions`：按 JetBrains 的说明，它会把产物标记为 pre-release 二进制，库在特性转正前不可被依赖。本工程只用到 block，不需要扩展。

没有把 JVM target 提到 22–28：Kotlin 2.5 能输出 Java 28 字节码，但 Android 侧由 AGP 自带的 D8/R8 决定，ART 不执行 JVM 字节码，提上去只会破坏构建。

## companion 改写

- 全仓 1352 个 `.kt` 文件全部过了一遍，逐文件结论在 [KOTLIN-25-AUDIT.tsv](KOTLIN-25-AUDIT.tsv)。
- 166 个 `companion object` 改为 `companion {}`。
- 19 个保留 `companion object`，原因分四类，均为语言或二进制约束：
  1. 成员是扩展（`val Int.celsius` 等）——companion 块不允许扩展成员；
  2. 被 `X.Companion.member` 引用——改成静态成员后该路径失效；
  3. companion 内有嵌套类或 `init {}`——语言不允许；
  4. 声明了父类型——只能由 object 承载。
- 删除了 companion 内唯一的 `@JvmStatic`（静态成员已由编译器生成）。

## 其余 2.5 特性评估

| 特性 | 处理 | 原因 |
| --- | --- | --- |
| name-based destructuring | 默认生效，不改代码 | `val (a, b)` 语义不变，没有必须改写的点 |
| `onTrue` / `onFalse` | 未使用 | 全仓 `if (c) { ...; true } else false` 命中 0 处 |
| `ifOrNull` | 未使用 | 55 处 `if (c) x else null` 均比新写法短，按规则 20 不批量替换 |
| `CodePoint` | 未使用 | 没有需要按码点处理的字符串逻辑 |
| full value classes | 未使用 | 会改变相等性、装箱与序列化，需逐类重验 |
| `@IntroducedAt` | 未使用 | 全源码同编的应用没有二进制消费者 |
| unused return value checker | 默认开启，未产生错误 | 仅有警告 |

## 未验证项

1. **单元测试未执行**：`testBasicDebugUnitTest` 需要 `org.apiguardian:apiguardian-api:1.1.2`，本机到 Maven Central 的 TLS 被网络阻断，依赖拉不下来。
2. **Compose 稳定性报告未对比**：没有对比转换前后的 `*-classes.txt`。
3. **Room schema `identityHash` 未对比**：本机没有导出旧 schema 做差分。

这三项都不影响编译和运行，但如果要追到“零风险”，需要在有网络的环境补跑。

## 构建与产物

- `:app:assembleBasicDebug` 通过。
- `:app:assembleBasicRelease` 通过（R8 + 资源压缩）。
- 五个 APK 用发布密钥签名，证书 SHA-256 `ab1a327e665d8e17899b075240762e5541978dce09402b4ef3158135b2572dba`。
- Pixel 10a（Android 17）安装运行正常。
