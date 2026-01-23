# Word Card

## 项目简介
Word Card 是一个基于 Tauri + Vue 3 + TypeScript 的本地单词卡片学习工具，主打轻量、离线、低干扰的学习体验。应用以本地词库为核心，支持导入单词数据与学习进度跟踪，并通过托盘驻留实现随时开启的学习节奏。

## 使用场景
- 备考英语等级/留学考试，进行日常词汇记忆与复习
- 上班/学习间隙的碎片化刷词，快速进入与退出学习
- 个人词库管理：按清单整理、导入、复习、标记难词
- 离线环境下的稳定学习，不依赖网络或账号

## 技术栈
本项目为100% vibe coding，使用codex结合vibe-kanban完成
- 桌面端：Tauri
- 前端：Vue 3 + TypeScript
- 构建：Vite

## 平台兼容性
基于 Tauri Bundler 的官方支持范围，本项目面向桌面平台：
- Windows（已测试）
- macOS
- Linux

说明：
- 当前为桌面应用配置，未提供移动端（iOS/Android）适配
- `src-tauri/tauri.conf.json` 中 `bundle.targets` 为 `all`，生成的包类型随构建机平台而定

## 源码启动
1. 安装依赖：
   ```bash
   npm install
   ```
2. 启动开发模式（Tauri + Vite）：
   ```bash
   npm run tauri dev
   ```

## 打包构建
```bash
npm run tauri build
```

## 启动方式
- 开发模式启动：执行 `npm run tauri dev`
- 生产包启动：在 `src-tauri/target/release/bundle/` 目录中找到对应平台的安装包或可执行文件并运行
