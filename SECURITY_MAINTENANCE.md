# 🛡️ Space and Time: Security Maintenance Guide

This project includes automated tools to protect the repository from bounty-hijacking and impersonation spam.

## 🤖 Bounty Spam Sentinel (Automatic)
The GitHub Action located at `.github/workflows/bounty-spam-sentinel.yml` automatically triggers on every new comment. It:
1.  Detects patterns consistent with the **BossChaos impersonation attack**.
2.  Specifically blacklists the known hijack wallet: `0xdaE5d307339074A24F579dB48e7c639359D94904`.
3.  Automatically deletes the comment to prevent maintainer confusion and misrouted payouts.

## 🧹 Mass Cleanup Script (Manual)
If the repository is hit by a large wave of spam, maintainers can run the cleanup script locally:

```bash
# Requirements: GitHub CLI (gh) installed and authenticated
python3 scripts/mass_spam_cleanup.py
```

This script will:
- Search for all PRs containing the blacklisted wallet.
- Identify and delete the specific impersonation comments.
- Restore the cleanliness of the PR review threads.

---
*Maintained by the Strategic Contributor Lab*
