# Add Prompt Workspace

## Summary

把 prompt 从 generation event 的字段提升为可管理的一等资产, 新增 Prompt Workspace, Prompt Library, immutable prompt versions, template variables, parameter presets, prompt notes, generation from prompt, prompt-to-output history, 以及 asset detail 的 prompt lineage.

## Motivation

当前系统可以追溯 generation prompt, 但 prompt 仍只是 generation event 的事实字段. 这可以满足事实审计, 但无法支撑 "管理 prompt 实验" 的核心产品定位. 用户需要管理 prompt draft, 保存可复现版本, 从 prompt 发起 generation, 并从 output asset 反查 prompt 来源.

Prompt Workspace 将 prompt lifecycle 从一次性 generation input 中分离出来, 让 prompt document, version, notes, variables 和 preset 拥有明确 ownership, 同时继续保留 generation event 作为不可变 run record.

## Scope

- 新增 prompt document 和 prompt version 持久化模型.
- 新增 Prompt Workspace desktop workflow, 与 Gallery, Albums, Review, Queue, Settings 同级.
- 支持 template variables, default values 和 run-time rendering.
- 支持 reusable negative prompt 和 style prompt, 作为 prompt version 的一等字段.
- 支持 parameter preset 作为 generation defaults.
- 支持从 prompt version 发起 image generation task.
- generation event 新增 nullable prompt version link, 同时继续保存 rendered prompt snapshot.
- prompt version detail 支持 prompt-to-output history.
- asset detail 支持 prompt lineage link 和 legacy Save as Prompt.
- 旧 generation event 继续可读, migration 不批量创建 prompt documents.

## Non-Goals

- 不实现 prompt diff viewer.
- 不实现 prompt folders, collections 或 advanced taxonomy.
- 不实现 AI-assisted prompt improvement.
- 不实现 multi-prompt composition graph.
- 不批量回填历史 generation events 为 prompt documents.
- 不引入 cloud shared prompt library 或 multi-user collaboration.
- 不改变 asset metadata review-first 语义.
- 不把 image-to-image 的 source asset 或 input file 固化进 prompt version.
