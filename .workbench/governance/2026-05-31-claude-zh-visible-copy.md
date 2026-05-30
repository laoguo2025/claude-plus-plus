# Claude zh-CN visible-copy completion

## Context
The user clarified that "complete localization" means user-visible Claude Desktop UI text should be translated, while invisible code, identifiers, paths, variables, product names, and technical markers do not need forced translation.

## Change reason
The reference zh-CN pack still had current-version visible strings that fell back to English. A direct rewrite of the large upstream pack would be hard to review, so this slice adds a zh-CN visible-copy override pack applied after the base translation pack.

Later user verification found Developer -> Configure third-party inference still had English copy. Those strings were not in normal i18n JSON; they were hardcoded in the frontend bundle, so this slice also extends the existing hardcoded frontend replacement pack for that settings surface.

Further screenshot review showed the same hardcoded surface spans Sandbox/workspace, connectors/extensions, diagnostics/updates, usage limits, plugins/skills, and egress requirements. The hardcoded replacement pack now treats Configure third-party inference as one settings surface instead of fixing a single tab.

## Non-change constraints
- Do not translate hidden resource keys solely because they contain English.
- Preserve ICU placeholders, XML-like tags, keyboard shortcuts, paths, URLs, model/product names, currencies, and technical identifiers.
- Keep installation and rollback behavior from the previous localization slice.
- Preserve product/provider names such as Anthropic API, AWS, Vertex AI, Bedrock, and Foundry where they function as names rather than prose.

## Verification plan
- `npm run audit:claude-zh` should report `count: 0` for visible English fallback candidates on the current installed Claude resources.
- `npm run audit:claude-zh` also simulates `frontend-hardcoded-zh-CN.json` against the third-party inference frontend bundle and checks tracked hardcoded strings from the reported tabs.
- Simulate `frontend-hardcoded-zh-CN.json` replacements against the installed frontend JS and confirm the reported third-party inference English strings are no longer present.
- `npm run build`
- `.\build.bat check`
- `.\build.bat test`
- Live install is performed only with explicit permission because it changes the external Claude Desktop installation and restarts Claude.

## Rollback
Revert the local commit. If the override was already installed into Claude Desktop, use the app's restore action to revert Claude resources from `.zh-cn-backups`.
