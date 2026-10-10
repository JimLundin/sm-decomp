#!/usr/bin/env python3
"""Run exactly the requested number of dependency-ready, exercised routine jobs."""
import argparse
import datetime
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--job', default='next-random')
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--accepted', type=Path, default=ROOT / 'out/decomp/ticket06/offline/decomp-acceptance.json')
    parser.add_argument('--output', type=Path, default=ROOT / 'out/decomp/jobs')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    verdict = {'status': 'error', 'requested_count': args.count, 'accepted_jobs': [], 'remaining_C_routines': 'untranslated; no complete routine census', 'remaining_CPU_fallback': 'not-measured'}
    try:
        if args.job != 'next-random': raise ValueError('unknown job; no candidate selected')
        if args.count != 1: raise ValueError('requested count must be exactly 1; only one dependency-ready job exists')
        from decomp_next_random import run_job
        result = run_job(args)
        verdict.update(result)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        verdict['error'] = str(error)
    (args.output / 'progress.json').write_text(json.dumps(verdict, indent=2) + '\n')
    print(json.dumps(verdict))
    return 0 if verdict['status'] == 'accepted' else 1
if __name__ == '__main__': sys.exit(main())
