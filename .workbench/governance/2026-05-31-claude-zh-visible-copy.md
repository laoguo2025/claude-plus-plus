# Claude zh-CN visible-copy completion

## Context
The user clarified that "complete localization" means user-visible Claude Desktop UI text should be translated, while invisible code, identifiers, paths, variables, product names, and technical markers do not need forced translation.

## Change reason
The reference zh-CN pack still had current-version visible strings that fell back to English. A direct rewrite of the large upstream pack would be hard to review, so this slice adds a zh-CN visible-copy override pack applied after the base translation pack.

## Non-change constraints
- Do not translate hidden resource keys solely because they contain English.
- Preserve ICU placeholders, XML-like tags, keyboard shortcuts, paths, URLs, model/product names, currencies, and technical identifiers.
- Keep installation and rollback behavior from the previous localization slice.

## Verification plan
- `npm run audit:claude-zh` should report `count: 0` for visible English fallback candidates on the current installed Claude resources.
- `npm run build`
- `.\build.bat check`
- `.\build.bat test`
- No live install is performed unless explicitly requested because it changes the external Claude Desktop installation and restarts Claude.

## Rollback
Revert the local commit. If the override was already installed into Claude Desktop, use the app's restore action to revert Claude resources from `.zh-cn-backups`.
