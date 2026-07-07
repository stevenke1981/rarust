# Model Routing Workflow

## Goal

用適合的模型處理適合的任務，降低 token 浪費與重工。

## Suggested Routing

| Task | Suggested Model Level |
|---|---|
| File search, todo organization, copy edits | Mini / medium |
| Normal bug fix, small refactor, UI tweak | High |
| Build/test/UI validation, multi-step work | GPT-5.5 high |
| Repeated failures, architecture redesign, tests keep failing | GPT-5.5 xhigh |

## Rule

升級模型的理由不是「想更強」，而是：

- 多次失敗
- 跨檔案架構錯誤
- 測試持續過不了
- 需要重新設計 workflow
- 風險高且重工成本高
