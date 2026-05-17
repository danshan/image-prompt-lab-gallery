# Provider 配置

本 MVP 将 provider 分为两类:

- Experimental CLI provider: 通过本机命令复用本机授权状态.
- Stable native provider: 通过公开 API 和显式 credential 调用.

## Codex CLI imagegen provider

当前可用 provider 名称:

- `codex-cli`
- `codex`

Codex provider 不指定图片模型, 也不指定输出路径. 它调用本机 `codex exec` 并要求 Codex 使用 `imagegen` skill 完成图片生成.

Adapter 执行协议:

```bash
codex exec --cd <library-or-job-dir> --sandbox workspace-write --json <prompt>
```

Prompt 使用共享 schema:

```text
Use case: ai-agent-image-prompt-lab
Asset type: managed library image
Primary request: <prompt>
Input images: <input image role or none>
Scene/backdrop: infer from primary request
Subject: infer from primary request
```

Codex CLI 生成完成后, 输出文本中通常包含复制后的最终图片路径, 例如 `/tmp/example.png`, 原始文件保留在 `$HOME/.codex/generated_images/...`.

Adapter 会:

- 解析 stdout 和 stderr 中的绝对图片路径.
- 只接受常见图片扩展名.
- 校验文件存在.
- 读取文件 bytes, 再交给 core 导入 managed library.
- 将 command, prompt, stdout 和 stderr 保存到 generation event raw payload.
- 通过 core 的 generation request builder 复用 CLI 和 desktop 的 provider 名称归一化, operation 推断和 input image loading.
- 由 core 写入当前标准 checksum metadata: `SHA-256` algorithm 和 64 位十六进制 digest.

失败场景会归一化为 domain error:

- 无法执行 `codex` 命令.
- `codex exec` 返回非零 exit status.
- 输出中没有可解析图片路径.
- 图片路径不存在或不可读.

## Fake provider

`fake` provider 用于测试和本地 smoke flow, 不依赖外部凭证.

```bash
cargo run --offline -p imglab-cli -- generate --library /tmp/imglab-library --provider fake --prompt "test image" --json
```

## OpenAI API 和 Grok provider

OpenAI API stable provider 和 Grok native provider 仍是后续任务. 实现前需要基于当前官方文档确认:

- Endpoint.
- Authentication header.
- Request 参数.
- 返回图片格式.
- 图生图输入格式.
- 错误响应结构.

Credential resolution 应通过 `ProviderCredentialStore` trait 注入, 不应读取 Codex 内部授权文件.

建议环境变量命名:

```bash
export OPENAI_API_KEY=<key>
export XAI_API_KEY=<key>
```

这些 native provider 完成后, CLI 和桌面端仍应只依赖统一的 `ImageProvider` trait 和 normalized `GenerationResult`.
