# Security Policy

## Reporting Security Issues

For security issues related to sxt-proof-of-sql, please open an issue with label `security` or contact the maintainers directly.

## Brand Impersonation Spam - Algora Bounty Scams

We've observed a pattern of **brand impersonation spam** targeting our bounty program. Fake accounts post comments mimicking official Algora notifications to redirect payouts to fraudulent wallets.

### Identifying Legitimate Algora Notifications

Official Algora bounty notifications come **only** from:
- **Bot account**: `algora-pbc[bot]` (note the `[bot]` suffix)
- **Format**: Structured header `Code Review — Bounty #XXX`, with `PR / ✅ / Wallet` block
- **Verified wallet**: Payments go to the PR author's registered Stripe/wallet via Algora's automated system

### Fake/Impersonator Accounts

We've seen impersonators using accounts like `@BossChaos` (no `[bot]` suffix) posting comments with:
- **Fake wallet address**: `0xdaE5d307339074A24F579dB48e7c639359D94904`
- **Mimicked formatting**: They copy the official `algora-pbc[bot]` style but the wallet is hardcoded
- **Multiple PRs**: The same fake wallet appears across 94+ PRs in this repo

### Recommended Actions for Maintainers

If you encounter impersonation spam:

1. **Block the user**:
   - Go to repo **Settings → Moderation → Interaction limits / Block users**
   - Block the impersonator account (e.g., `@BossChaos`)

2. **Hide spam comments**:
   - For each affected PR, click `...` on the comment → **Hide → Off-topic**
   - This removes the comment from public view while preserving PR history

3. **Verify bounty claims**:
   - Only trust bounty claims made via:
     - Comment: `/claim #XXX` in the PR body
     - Official bot response: `algora-pbc[bot]` adding bounty-claim label
   - Payouts are automated — no manual wallet transfers needed

### Known Fake Wallets

- `0xdaE5d307339074A24F579dB48e7c639359D94904` (appears in 94+ PRs, zero transaction history on Etherscan)

### Detection Script

We provide a script to help identify PRs with known spam wallets:

```bash
# Run from repo root
./scripts/detect-algora-spam.sh
```

This generates `algora-spam-report.txt` with all affected PR numbers and comments.

### Questions?

Reach out to @stuarttimwhite, @iajoiner, or @tlovell-sxt for clarification.

---

**Note**: The actual Algora payout system is secure — this is third-party spam attempting to intercept manual payouts. Your automated Algora payouts remain safe.
