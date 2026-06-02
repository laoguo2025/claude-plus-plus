# Enhance Dialog Labels

## Request
- Rename `对话栏增强` to `对话增强`.
- Rename `导出对话为 Markdown` to `对话导出Markdown`.
- Rename `显示对话时间线` to `对话时间线` and move it into `对话增强`.

## Change
- Updated Claude++ page-enhance card labels in both the frontend preview data and Rust status data.
- Kept feature ids, markers, install behavior, and descriptions unchanged.

## Verification
- `npm run build`
- `cargo test --lib`

## Rollback
- Revert the local commit for this change.
