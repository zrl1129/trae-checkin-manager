# TRAE 签到管理器

基于 Tauri 2 + React 的 TRAE 多账号自动签到桌面工具。

## 架构设计

本项目参考以下两个开源项目的架构重新实现：

- **BlueChonk/trae-daily-checkin** — CDP 签到流程（DOM 操作完成签到）
- **xhrxgr/Trae-Work-CN-Account-Manager** — 账号/实例管理架构（Tauri 2 + Rust）

### 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 18 + TypeScript + Vite + TailwindCSS |
| 后端 | Rust + Tauri 2 |
| CDP 通信 | tokio-tungstenite (WebSocket) + reqwest (HTTP) |
| 数据存储 | 本地 JSON 文件 |

### 项目结构

```
trae-checkin-manager/
├── src/                          # React 前端
│   ├── App.tsx                   # 主应用（侧边栏导航）
│   ├── api.ts                    # Tauri 命令封装
│   ├── types.ts                  # 类型定义
│   ├── index.css                 # 全局样式
│   └── pages/
│       ├── AccountsPage.tsx      # 账号管理页
│       ├── InstancesPage.tsx     # 实例管理页
│       └── CheckinPage.tsx       # 签到中心页
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # Tauri 命令注册 + AppState
│   │   ├── state.rs              # 应用状态
│   │   ├── storage.rs            # JSON 持久化
│   │   ├── account/
│   │   │   ├── types.rs          # 账号类型定义
│   │   │   └── manager.rs        # 账号 CRUD
│   │   ├── instance/
│   │   │   ├── types.rs          # 实例类型定义
│   │   │   └── manager.rs        # 实例 CRUD + 端口分配
│   │   ├── checkin/
│   │   │   ├── cdp.rs            # CDP WebSocket 客户端
│   │   │   ├── flow.rs           # 签到流程（DOM 操作）
│   │   │   └── types.rs          # 签到记录类型
│   │   └── trae/
│   │       ├── path.rs           # TRAE 可执行文件定位
│   │       └── process.rs        # 进程启动/关闭/调试端口
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## CDP 签到流程

签到流程参考 BlueChonk 项目的实现：

1. **连接 CDP** — HTTP GET `/json` 获取 page target → WebSocket 连接 `webSocketDebuggerUrl`
2. **启用 Runtime** — 发送 `Runtime.enable` 命令
3. **打开账户菜单** — 点击 `[class*="accountTrigger"]` 元素，等待 `[class*="accountPopover"]` 出现
4. **读取签到按钮** — 查找 `[class*="accountCheckinButton"]` 和 `[class*="accountCheckinButtonLabel"]`
5. **判断状态**：
   - 按钮文字含"已签" → 今日已签
   - 按钮文字为"签到" → 点击签到
   - 按钮未找到 → 检查是否未登录
6. **验证结果** — 等待 2 秒后再次读取按钮文字，包含"已签"则签到成功
7. **记录结果** — 保存状态、详情、积分、时间到本地 JSON

## 实例管理

每个账号拥有独立的 TRAE 实例：

- **独立 user-data-dir** — `%APPDATA%\TRAE SOLO CN_{name}`
- **独立调试端口** — 从 9222 开始自动分配
- **进程管理** — `taskkill` / `tasklist` 管理 TRAE 进程
- **启动参数** — `--remote-debugging-port` + `--user-data-dir`

## 功能清单

### 第一阶段（已完成）

- [x] Tauri 2 + React + TypeScript 项目骨架
- [x] 账号管理（增删改查、本地 JSON 存储）
- [x] 独立实例管理（user-data-dir 隔离、端口分配）
- [x] TRAE 进程启动/关闭
- [x] CDP WebSocket 连接
- [x] 单账号签到流程（DOM 操作）
- [x] 签到状态记录（成功/失败/已签/未登录）
- [x] 前端签到状态实时显示

### 后续阶段（计划）

- [ ] 批量签到
- [ ] 只处理当天未签到账号
- [ ] Windows 定时任务
- [ ] 自动判断今日已签（跳过）
- [ ] 登录状态检测优化

## 开发

### 环境要求

- Node.js 18+
- Rust (stable)
- Visual Studio Build Tools (C++ 工作负载)

### 运行

```bash
npm install
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## License

MIT
