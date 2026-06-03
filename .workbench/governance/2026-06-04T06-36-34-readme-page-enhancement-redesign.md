# README Page Enhancement Redesign

## Scope
- Rewrite README positioning around Claude Desktop page enhancement instead of only CC Switch model-name bridging.
- Use user-provided screenshots from Desktop: 01.png, 02.png, 03.png, 04.png.
- Remove internal workflow explanation from README.
- Add GitHub topic `claude-code` and labels `claude`, `claude-code`.
- Keep Chinese and English content synchronized.

## Validation Plan
- Confirm README does not include sponsor/donation wording or the removed workflow explanation.
- Run `npm run build`.
- Read back GitHub topics and labels with `gh`.

## Rollback
- Revert the commit for README and image changes.
- Remove `claude-code` topic or labels with `gh` if needed.

## Validation Results
- README forbidden-text check passed: no sponsor/donation wording and no internal workflow explanation.
- `npm run build` passed.
- GitHub topics include `claude-code`.
- GitHub labels include `claude` and `claude-code`.
