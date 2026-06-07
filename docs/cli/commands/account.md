# claux account

Displays your Claude account information and skill usage table, read from your local `~/.claude.json` file.

## Usage

```bash
claux account
```

## Output

```
  Account

  Name:             Snow
  Email:            snow@example.com
  Plan:             Claude Pro
  Organization:     Personal
  Role:             admin
  Billing:          stripe_subscription
  Rate tier:        default_claude_ai
  Account since:    2024-03-15
  Subscribed since: 2024-03-15
  Extra usage:      enabled

  Skills

  ┌─────────────────┬──────┬───────────┬──────────┐
  │ Skill           │ Uses │ Last used │ Rating   │
  ├─────────────────┼──────┼───────────┼──────────┤
  │ run             │  42  │ 2h ago    │ ★★★★★   │
  │ code-review     │  18  │ 1d ago    │ ★★★★☆   │
  │ ultrareview     │   3  │ 5d ago    │ ★★★☆☆   │
  └─────────────────┴──────┴───────────┴──────────┘
```

## Account fields

| Field | Source | Description |
|---|---|---|
| **Name** | `~/.claude.json` | Your display name |
| **Email** | `~/.claude.json` | Account email address |
| **Plan** | `~/.claude.json` | Claude plan (Free, Pro, Max, etc.) |
| **Organization** | `~/.claude.json` | Org name or "Personal" |
| **Role** | `~/.claude.json` | Your role in the organization |
| **Billing** | `~/.claude.json` | Billing method |
| **Rate tier** | `~/.claude.json` | API rate tier |
| **Account since** | `~/.claude.json` | Account creation date |
| **Subscribed since** | `~/.claude.json` | Subscription start date |
| **Extra usage** | `~/.claude.json` | Whether extra usage beyond plan limits is enabled |

All data is read from `~/.claude.json` — the local file Claude Code maintains. Nothing is fetched from the network.

## Skill table

The skill table shows all skills tracked in the `skillUsage` block of `~/.claude.json`, ranked by usage count:

| Column | Description |
|---|---|
| **Skill** | Skill name |
| **Uses** | Total invocation count across all sessions |
| **Last used** | Relative time of last invocation |
| **Rating** | 1–5 star rating based on usage count |

### Rating thresholds

| Uses | Stars |
|---|---|
| 0 | ★☆☆☆☆ |
| 1–2 | ★★☆☆☆ |
| 3–9 | ★★★☆☆ |
| 10–29 | ★★★★☆ |
| 30+ | ★★★★★ |

## Data source

`claux account` is entirely read-only and offline. It reads `~/.claude.json` which Claude Code writes and maintains. If the file doesn't exist or is empty, `claux account` will show an empty state.
