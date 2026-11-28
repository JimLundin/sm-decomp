import contextlib
import socket
import subprocess
import sys
import threading

USAGE = """usage: box.py box PORT SOCKET SECONDS -- ARGV

The first process of `cleanroom run`'s box, run by the host's python3 inside the box, which
has no network and only the standard library. Listens on 127.0.0.1:PORT, forwards each
connection to the relay's Unix SOCKET, runs ARGV, stops it after SECONDS and exits with its
status."""

LOOPBACK = "127.0.0.1"
BACKLOG = 64
CHUNK = 65536
DEADLINE_PASSED = 124
COMMAND_WORDS = 5


def pump(source: socket.socket, sink: socket.socket) -> None:
    try:
        while data := source.recv(CHUNK):
            sink.sendall(data)
    except OSError:
        pass
    finally:
        for end in (source, sink):
            with contextlib.suppress(OSError):
                end.shutdown(socket.SHUT_RDWR)


def listening(port: int) -> socket.socket:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind((LOOPBACK, port))
    listener.listen(BACKLOG)
    return listener


def forward(listener: socket.socket, sock_path: str) -> None:
    while True:
        client = listener.accept()[0]
        relay = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            relay.connect(sock_path)
        except OSError:
            client.close()
            continue
        for source, sink in ((client, relay), (relay, client)):
            threading.Thread(
                target=pump, args=(source, sink), daemon=True
            ).start()


def box(port: int, sock_path: str, seconds: float, *argv: str) -> int:
    listener = listening(port)
    threading.Thread(
        target=forward, args=(listener, sock_path), daemon=True
    ).start()
    child = subprocess.Popen(argv)
    try:
        return child.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        print(
            f"box_relay: the session passed its {seconds:g} s deadline; stopped",
            file=sys.stderr,
        )
        child.kill()
        _ = child.wait()
        return DEADLINE_PASSED


def main() -> int:
    args = sys.argv[1:]
    if len(args) > COMMAND_WORDS and args[0] == "box" and args[4] == "--":
        return box(int(args[1]), args[2], float(args[3]), *args[COMMAND_WORDS:])
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
