## 1. Baseline 和测试保护

- [x] 1.1 对照 `docs/PERFORMANCE_REVIEW.md` 复核仍适用于当前代码的 finding, 标记已覆盖, deferred 或不适用.
- [x] 1.2 为 Gallery query 增加包含多个 assets, versions, tags, generation events 和 pending suggestions 的回归测试.
- [x] 1.3 为 manual album filter 和 album_order sort 增加回归测试.
- [x] 1.4 为 checksum digest 增加 SHA-256 和 MD5 known-input 测试.
- [x] 1.5 为 PNG, JPEG 和 WebP dimension parsing 增加 bounded header read 测试.
- [x] 1.6 为 Settings app logs 增加不扫描 system temp root 的测试.
- [x] 1.7 为 daemon scheduler no-work tick 和 health/capabilities responsiveness 增加测试或可验证 harness.

## 2. Core Gallery/Search Hot Path

- [x] 2.1 在 SQLite schema migration 中添加 Gallery/Search/Album/Metadata 热路径索引, 并保持 migration idempotent.
- [x] 2.2 将 Gallery base asset load 与 current version, latest generation event, version count, tags 和 pending review count 改为 batch preload.
- [x] 2.3 将 manual album membership filter 改为一次性加载 album asset id set.
- [x] 2.4 将 album_order sort 改为排序前一次性加载 album item sort_order map.
- [x] 2.5 将 Search text/tag/provider filter 改为 SQL filter, batch preload 或复用 batch read model, 移除 per-asset 查询.
- [x] 2.6 确认 Gallery/Search 返回字段和排序 tie-break 与现有 spec 一致.

## 3. Resource Library Bounded IO

- [x] 3.1 将 SHA-256 reader 改为 streaming digest, 避免完整文件读入内存.
- [x] 3.2 将 MD5 reader 改为 streaming digest 或至少 bounded chunk processing.
- [x] 3.3 将 image dimensions 读取改为 bounded prefix/header read, 不为尺寸解析读取完整图片.
- [x] 3.4 确认 import, generation, repair 和 integrity check 仍使用正确 checksum algorithm.

## 4. Desktop Tauri 和 Frontend Performance

- [x] 4.1 将 legacy `generate_image` Tauri command 移到 blocking execution boundary, 避免同步阻塞 command thread.
- [ ] 4.2 按 workflow 从 `apps/desktop/src/main.tsx` 抽出 Gallery, Albums, Review, Task, Settings, Inspector 和 shared UI 组件.
- [x] 4.3 抽出必要 data hooks 或纯 helper, 让 IPC orchestration 与 rendering boundary 更清晰.
- [x] 4.4 为 Gallery image 增加 lazy loading, async decoding 和稳定尺寸约束.
- [x] 4.5 为 Gallery query refresh 和 smart album preview 增加 debounce 或 memoized derivation.
- [x] 4.6 Memoize provider list, queue count, filtered gallery 等传入大型子组件的 derived data.
- [x] 4.7 将语义独立的 refresh waterfall 改为并发执行, detail refresh 保持条件化.
- [x] 4.8 为 polling 和 delayed task wait 增加 timeout ref cleanup, 覆盖 unmount 和 library switch.

## 5. Daemon 和 Operational Cleanup

- [x] 5.1 调整 daemon HTTP handling, 在 request parse/auth 后再按需获取 mutable state lock.
- [x] 5.2 让 health 和 capabilities 响应尽可能不依赖长时间 mutable state lock.
- [x] 5.3 为 scheduler 增加 no opened library 和 no runnable task 的 cheap no-work path.
- [x] 5.4 调整 daemon client timeout, 区分 health, task detail 和 log preview 等请求类型.
- [x] 5.5 为 transient daemon connection failure 增加合理 backoff 或避免 tight retry.
- [x] 5.6 将 Settings app logs listing 限制到 app-owned roots 和已知 provider log roots.
- [x] 5.7 确认 app log content read 仍拒绝非允许目录或非已知 pattern 路径.

## 6. SQLite Sufficiency Checkpoint 和文档

- [x] 6.1 增加 synthetic library benchmark 或可重复测试脚本, 至少覆盖 10k assets 的 Gallery/Search 数据形态.
- [x] 6.2 记录 Gallery query, Search query, import/write latency 和 daemon/desktop concurrent write 的观察结果.
- [x] 6.3 在 `docs/PERFORMANCE_REVIEW.md` 或后续 performance notes 中记录已修复项, deferred 项和 SQLite sufficiency 结论.
- [x] 6.4 如果 checkpoint 显示 SQLite 不足, 创建后续 OpenSpec change, 不在本 change 中直接替换 storage backend.

## 7. 验证

- [x] 7.1 运行 `cargo fmt --all --check`.
- [x] 7.2 运行 `cargo test --offline -p imglab-core -p imglab-provider-codex -p imglab-cli`.
- [x] 7.3 运行 `cargo test --offline -p imglab-daemon`.
- [x] 7.4 运行 `cargo check --offline -p imglab-core -p imglab-cli -p imglab-provider-codex -p imglab-provider-grok`.
- [x] 7.5 如果本地 Tauri dependencies 可用, 运行 `cargo check --offline -p imglab-desktop`.
- [x] 7.6 在 `apps/desktop` 运行 `npm run test` 和 `npm run build`.
- [ ] 7.7 手工 smoke test Gallery search, album detail/order, review accept/regenerate, queue task detail, Settings logs 和 lightbox.
