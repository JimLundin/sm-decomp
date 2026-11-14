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
from typing import BinaryIO


def capture(command: list[str], cwd: str, transcript: str, stderr: str, timeout: float,
            *, validate_result: bool = True, environment: dict[str, str] | None = None,
            runtime_timeout: float | None = None) -> int:
    events: queue.Queue[tuple[str, bytes | OSError | None]] = queue.Queue(maxsize=16)
    with open(transcript, "wb") as output, open(stderr, "wb") as errors:
        try:
            child = subprocess.Popen(command, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                     env=environment)
        except OSError as error:
            print(f"launch: could not start CLI: {error}", file=sys.stderr)
            return 1

        assert child.stdout is not None
        assert child.stderr is not None
        def read(pipe: BinaryIO, label: str) -> None:
            try:
                while chunk := os.read(pipe.fileno(), 65536):
                    events.put((label, chunk))
            except OSError as error:
                events.put((label, error))
            finally:
                pipe.close()
                events.put((label, None))

        threading.Thread(target=read, args=(child.stdout, "stdout"), daemon=True).start()
        threading.Thread(target=read, args=(child.stderr, "stderr"), daemon=True).start()
        deadline = None
        runtime_deadline = time.monotonic() + runtime_timeout if runtime_timeout else None
        ended: set[str] = set()
        try:
            while True:
                status = child.poll()
                if runtime_deadline is not None and status is None and time.monotonic() >= runtime_deadline:
                    print("launch: command runtime timed out; descendants uncertified", file=sys.stderr)
                    return 124
                if status is not None and deadline is None:
                    deadline = time.monotonic() + timeout
                if status is not None and len(ended) == 2:
                    break
                if deadline is not None and time.monotonic() >= deadline:
                    print(f"launch: output still open after CLI exit {status}; completion timed out", file=sys.stderr)
                    return 124
                try:
                    label, item = events.get(timeout=0.05)
                except queue.Empty:
                    continue
                if item is None:
                    ended.add(label)
                elif isinstance(item, OSError):
                    print(f"launch: stream read failed: {item}", file=sys.stderr)
                    return 124
                else:
                    destination = output if label == "stdout" else errors
                    destination.write(item)
                    destination.flush()

        finally:
            if child.poll() is None:
                child.kill()
                child.wait()

    if status != 0:
        print(f"launch: CLI exited {status}; see {stderr}", file=sys.stderr)
        return status if status is not None and 0 < status < 256 else 1
    if not validate_result:
        return 0
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
