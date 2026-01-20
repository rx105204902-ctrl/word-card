# 项目背景

## 目标
- 问题：传统背单词软件（手机 App）需要专门时间且易被打断；传统桌面软件窗口过大，干扰工作。
- 目标用户：长时间面对电脑工作或学习，希望利用“视觉碎片时间”潜移默化记忆单词的学生、开发者及外企职员。
- 目标：打造一款占用极低、不干扰操作的固定桌面右下角单词记忆卡片。
- 范围内：单词记忆程度显示（绿到红，五个阶段）、生词本管理、艾宾浩斯记忆算法、有道API加载词库。
- 范围外：音频流/发音播放（暂不做）。
- 成功指标：内存占用 < 50MB；右下角卡片稳定运行 24h 不闪退；用户单日交互次数 > 20 次。

## 技术栈
- 语言：Rust 1.75+，edition 2021
- 桌面框架：Tauri（纯桌面端）
- 前端：Vue 3
- 构建与包管理：cargo，标准 Tauri workspace 结构
- 工具链：rustfmt、clippy、rust-analyzer
- 异步运行时：tokio（Tauri 默认支持）
- 通信：本地 IPC（Tauri Commands / invoke）
- 序列化：serde、serde_json
- 错误处理：thiserror（底层逻辑库）、anyhow（Tauri 指令入口）
- 日志与追踪：tracing、tracing-subscriber
- 配置：tauri-plugin-store（UI 配置持久化）
- 存储：SQLite，驱动 sqlx（异步且轻量）
- 缓存或队列：无
- 测试：cargo test
- CI/CD：GitHub Actions（自动打包 Release EXE/DMG）
- 容器化/部署：无

## 项目约定

### 代码风格
- 格式化：rustfmt 默认配置。
- 静态检查：CI 中运行 clippy；禁止 warning。
- 命名：Rust 侧遵循标准规范；Vue 侧组件采用 PascalCase。
- 错误处理：避免 unwrap；Tauri Commands 返回 Result<T, String> 供前端捕获。
- 导入：显式导入。

### 架构模式
- 分层：
    - UI（Vue 3）：交互与动画。
    - Tauri Commands：Controller，负责前后端通信。
    - Core Service：记忆算法逻辑（SM-2）。
    - Repository：SQLite 单词数据的增删改查。
- 接口设计：Rust 定义 Word struct 并派生 Serialize，通过 invoke 返回前端。

### 测试策略
- 单元测试：重点覆盖 SM-2 记忆算法的日期计算逻辑。
- 集成测试：覆盖 SQLite 数据迁移与初始词库加载。

### Git Workflow
分支模型:采用简化的GitHubFlow。
main分支是主分支，始终保持可部署状态。
从main 创建功能分支(例如feature/file-upload)。
完成开发后，提交PullRequest(PR)合并回main。
提交规范:遵循 Conventional Commits 规范(例如feat:,fix:,refactor:)以便清晰地追踪变更。

## 领域上下文
- 术语表：
    - Retention（保留率）：用户记住单词的概率。
    - Interval（间隔）：单词下次复现的时间距离。
    - Ease Factor（简易度）：单词难易程度的修正值。
- 实体：
    - Word：单词原文、释义、音标。
    - ReviewLog：用户复习记录（时间、评分）。

## 重要约束
- 性能：Windows 11 启动 < 1s；内存占用 < 50MB；稳定运行 24h 不闪退。
- 兼容性：Windows 10/11，macOS 12+，Ubuntu 24+。
- 安全与合规：用户数据仅存本地 %APPDATA%，不上传隐私信息。

## 外部依赖
- 服务/API：无（本地应用）。
- 受限第三方库：避免使用重量级图形库。
