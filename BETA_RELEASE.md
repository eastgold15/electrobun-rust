# Electrobun Beta 版本发布

## 用户指南

### 安装 Beta 版本

```bash
# 安装最新的 beta 版本
npm install electrobun@beta

# 安装特定的 beta 版本
npm install electrobun@0.0.19-beta.1

# 查看可用版本
npm view electrobun versions --json
```

### 在稳定版和 Beta 版之间切换

```bash
# 切换到稳定版
npm install electrobun@latest

# 切换到 beta 版
npm install electrobun@beta
```

## 维护者指南

### 发布 Beta 版本

1. **更新版本号：**
   ```bash
   # 新版本的首个 beta
   npm version 0.0.19-beta.1
   
   # 递增 beta 编号
   bun npm:version:beta
   ```

2. **创建 GitHub Release：**
   ```bash
   git push origin v0.0.19-beta.1
   ```
   或者手动触发带有 beta 标签的工作流。

3. **发布到 npm：**
   ```bash
   bun npm:publish:beta
   ```

### Beta 版本发布流程

1. Beta 版本使用语义化版本控制：`MAJOR.MINOR.PATCH-beta.NUMBER`
2. GitHub Actions 自动将带有 `-beta` 的发布标记为预发布版本
3. npm 发布到 `beta` 分发标签（而不是 `latest`）
4. 使用稳定版本的用户不会收到 beta 更新

### 将 Beta 版提升为稳定版

```bash
# 将版本更新为稳定版
npm version 0.0.19

# 作为 latest 发布
bun npm:publish
```