#!/usr/bin/env python3
# Tell a scheduled run to stop when an earlier scheduled run passed this commit.
import json, os, subprocess
skip = False
if os.environ["GITHUB_EVENT_NAME"] == "schedule":
    path = (f"repos/{os.environ['GITHUB_REPOSITORY']}/actions/workflows/{os.environ['WORKFLOW']}/runs"
            f"?event=schedule&status=success&head_sha={os.environ['GITHUB_SHA']}&per_page=1")
    skip = json.loads(subprocess.check_output(["gh", "api", path]))["total_count"] > 0
with open(os.environ["GITHUB_OUTPUT"], "a") as out:
    out.write(f"skip={'true' if skip else 'false'}\n")
print(f"skip={skip}")
