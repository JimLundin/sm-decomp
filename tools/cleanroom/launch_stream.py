"""Capture a CLI stream until all inherited stdout writers close, with bounded cleanup."""

import argparse
import json
import math
import os
import queue
import subprocess
import sys
import threading
import time


def capture(command: list[str], cwd: str, transcript: str, stderr: str, timeout: float) -> int:
    events: queue.Queue[bytes | OSError | None] = queue.Queue(maxsize=16)
    with open(transcript, "wb") as output, open(stderr, "wb") as errors:
        try:
            child = subprocess.Popen(command, cwd=cwd, stdout=subprocess.PIPE, stderr=errors)
        except OSError as error:
            print(f"launch: could not start CLI: {error}", file=sys.stderr)
            return 1

        assert child.stdout is not None
        stdout = child.stdout

        def read() -> None:
            try:
                while chunk := os.read(stdout.fileno(), 65536):
                    events.put(chunk)
            except OSError as error:
                events.put(error)
            finally:
                events.put(None)

        threading.Thread(target=read, daemon=True).start()
        deadline = None
        ended = False
        try:
            while True:
                status = child.poll()
                if status is not None and deadline is None:
                    deadline = time.monotonic() + timeout
                if status is not None and ended:
                    break
                if deadline is not None and time.monotonic() >= deadline:
                    print(f"launch: output still open after CLI exit {status}; completion timed out", file=sys.stderr)
                    return 1
                try:
                    item = events.get(timeout=0.05)
                except queue.Empty:
                    continue
                if item is None:
                    ended = True
                elif isinstance(item, OSError):
                    print(f"launch: stream read failed: {item}", file=sys.stderr)
                    return 1
                else:
                    output.write(item)
                    output.flush()

        finally:
            if child.poll() is None:
                child.kill()
                child.wait()

    if status != 0:
        print(f"launch: CLI exited {status}; see {stderr}", file=sys.stderr)
        return status if status is not None and 0 < status < 256 else 1
    try:
        with open(transcript, encoding="utf-8") as stream:
            last = None
            for line in stream:
                last = json.loads(line)
        if not isinstance(last, dict) or last.get("type") != "result":
            raise ValueError("missing terminal result event")
        if last.get("is_error") is True or str(last.get("subtype", "")).startswith("error"):
            raise ValueError("terminal result reports failure")
    except (OSError, UnicodeError, ValueError) as error:
        print(f"launch: invalid completed stream: {error}", file=sys.stderr)
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cwd", required=True)
    parser.add_argument("--transcript", required=True)
    parser.add_argument("--stderr", required=True)
    parser.add_argument("--timeout", type=float, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if not math.isfinite(args.timeout) or args.timeout <= 0 or not args.command:
        parser.error("a positive finite timeout and command are required")
    command = args.command[1:] if args.command[0] == "--" else args.command
    return capture(command, args.cwd, args.transcript, args.stderr, args.timeout)


if __name__ == "__main__":
    sys.exit(main())
