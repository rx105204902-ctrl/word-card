# Change: 主界面无词库提示与添加词库按钮

## Why
当用户未选择任何词库时，主界面仅显示学习错误，缺少明确引导，容易导致无法继续使用。

## What Changes
- 主界面在无激活词库时显示提示文案。
- 中心展示“添加词库”按钮，点击后跳转到设置页的词库导航。

## Impact
- Affected specs: main-learning
- Affected code: src/App.vue
