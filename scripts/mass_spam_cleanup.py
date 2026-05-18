import os
import subprocess
import json

def get_spam_comments():
    print("🔎 Searching for spam comments across all PRs...")
    # Search for the blacklisted wallet
    query = "0xdaE5d307339074A24F579dB48e7c639359D94904"
    cmd = ["gh", "search", "prs", query, "--json", "number,repository", "--limit", "100"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    return json.loads(result.stdout)

def clean_pr(pr_number, repo):
    print(f"🧹 Cleaning PR #{pr_number} in {repo}...")
    # Get all comments for this PR
    cmd = ["gh", "pr", "view", str(pr_number), "--repo", repo, "--json", "comments"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    comments = json.loads(result.stdout).get("comments", [])
    
    for comment in comments:
        if "0xdaE5d307339074A24F579dB48e7c639359D94904" in comment["body"]:
            print(f"  🚨 Found spam comment ID: {comment['id']}")
            # Delete the comment
            # Note: gh CLI doesn't have a direct 'comment delete' yet, so we use API
            api_cmd = ["gh", "api", "-X", "DELETE", f"repos/{repo}/issues/comments/{comment['id']}"]
            subprocess.run(api_cmd)
            print("  ✅ Deleted.")

def main():
    prs = get_spam_comments()
    for pr in prs:
        clean_pr(pr["number"], f"{pr['repository']['owner']['login']}/{pr['repository']['name']}")

if __name__ == "__main__":
    main()
