# Privacy

Lil Buddy is designed as a local-first desktop helper.

## Local-first by default

The core app should work without a ed Lil Buddy account.

Data that may be stored locally includes:

- settings
- expert definitions
- chat sessions
- workspace paths
- companion preferences
- Telegram gateway configuration

## Provider behavior

Lil Buddy may send your prompts, files, or workspace context to whichever provider/CLI you configure. Review your provider's privacy terms.

## Telegram gateway

If you use the self-hosted Telegram gateway, Telegram messages pass through your bot and your configured webhook/tunnel.

Do not share your bot token.

## Secrets

Lil Buddy should not sync API keys, bot tokens, or private secrets by default.