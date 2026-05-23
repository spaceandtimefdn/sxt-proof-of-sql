#!/bin/bash
# Script to detect Algora impersonation spam comments in PRs
# Searches for the fake wallet address: 0xdaE5d307339074A24F579dB48e7c639359D94904

REPO="spaceandtimefdn/sxt-proof-of-sql"
FAKE_WALLET="0xdaE5d307339074A24F579dB48e7c639359D94904"
OUTPUT_FILE="algora-spam-report.txt"

echo "Searching for Algora impersonation spam in $REPO..."
echo "Fake wallet: $FAKE_WALLET"
echo "========================================" > "$OUTPUT_FILE"
echo "Algora Impersonation Spam Report" >> "$OUTPUT_FILE"
echo "Generated: $(date)" >> "$OUTPUT_FILE"
echo "Fake wallet: $FAKE_WALLET" >> "$OUTPUT_FILE"
echo "========================================" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Search for PRs containing the fake wallet
echo "Fetching list of all PRs..."
prs=$(gh pr list --repo "$REPO" --state all --limit 500 --json number,title,author --jq '.[] | "\(.number) \(.title) @\(.author.login)"')

echo "Scanning PRs for spam comments..."
count=0
while IFS= read -r pr_info; do
    pr_num=$(echo "$pr_info" | awk '{print $1}')
    
    # Check PR comments for fake wallet
    comments=$(gh api "repos/$REPO/pulls/$pr_num/comments" --jq '.[].body' 2>/dev/null)
    
    if echo "$comments" | grep -q "$FAKE_WALLET"; then
        echo "FOUND SPAM in PR #$pr_num: $pr_info"
        echo "PR #$pr_num: $pr_info" >> "$OUTPUT_FILE"
        echo "  Comments with fake wallet:" >> "$OUTPUT_FILE"
        echo "$comments" | grep -n "$FAKE_WALLET" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
        ((count++))
    fi
done <<< "$prs"

echo ""
echo "========================================" >> "$OUTPUT_FILE"
echo "Total PRs with spam: $count" >> "$OUTPUT_FILE"
echo "========================================" >> "$OUTPUT_FILE"

echo "Scan complete. Found $count PRs with spam."
echo "Report saved to: $OUTPUT_FILE"
echo ""
echo "Next steps for maintainers:"
echo "1. Review the report: cat $OUTPUT_FILE"
echo "2. For each PR, hide the spam comment:"
echo "   gh api -X PATCH repos/$REPO/pulls/comments/COMMENT_ID -f reaction --- '{\"content\": \"eyes\"}'  # First, add eyes reaction to identify"
echo "   # Then hide via web UI: PR -> Comment -> ... -> Hide -> Off-topic"
echo "3. Block the spam user: Settings -> Moderation -> Block users -> @BossChaos"
