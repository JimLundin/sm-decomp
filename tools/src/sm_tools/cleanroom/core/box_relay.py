import hashlib
import json
import re
import tomllib
import urllib.parse
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from typing import TypeIs, cast

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

USAGE = """The API path out of `cleanroom run`'s box.

The box (a container with no network) holds no credential. Its CLI talks plain HTTP to
127.0.0.1 inside the box; the box's first process forwards each connection there to a Unix
socket mounted from the export's lane folder, where `serve`, outside the box, adds the real
credential and sends the request on to the API over TLS. So the key never enters the box, and
the API is the one place the box can reach.

What passes is an allow-list, since the box chooses every byte it sends: a POST of
`v1/messages` or `v1/messages/count_tokens` under the base URL's path, with no query but
`beta=true`, whose body is a JSON object of the members a coding session sends. Its tools are
the session's own, run in the box; a server tool is refused, as is an MCP server or a
container. Its content blocks are text, inline images, tool calls, tool results and thinking;
a document, a file, a container upload or an image by URL or file id is refused. So the API
runs nothing and fetches nothing for the box. The request goes to the configured host with
its own Host header and only the headers a client of the API sends (content type, accept,
user agent, anthropic-*, x-stainless-*, x-claude-*, x-app), and anthropic-beta keeps only the
betas Claude Code 2.1.331 sent from the box, since a beta is how a request turns on a server
feature. A body whose object repeats a member and a folded header are refused, since the API
could read either otherwise than the relay does.

  serve SOCKET AUTH [PARENT] [--scan SCANNER REPO MODEL]
                              outside: AUTH (a file, or - for stdin, so it touches no disk) is an
                              object of Claude Code's top-level settings members (apiKeyHelper,
                              env); this process's ANTHROPIC_API_KEY counts too.
                              It exits when PARENT (a PID) does. With --scan it answers a POST of
                              /cleanroom/scan itself, never forwarding it: `SCANNER names LANE`
                              runs in REPO on the request, with LANE the folder of SOCKET, and the
                              box gets each file's hit count alone, never the lexicon's words, and
                              the first SCAN_LINES hit lines of a file under MODEL's src/ or
                              tests/. A request holds each file's path and head; the scanner reads
                              its start from the lane's baseline of the export's first commit and
                              reports the bytes of the head's lines the start doesn't hold. A call
                              may cost at most SCAN_CALL_BYTES of those and SCAN_FILES files, and a
                              session SCAN_SESSION_BYTES and SCAN_SESSION_FILES over at most
                              SCAN_CALLS calls; a file asked about again as it was costs nothing.
                              It logs each call and the session's total in scan.log in LANE.
  env AUTH PORT               prints the environment the session gets instead, a KEY=VALUE a
                              line: the box's base URL, a placeholder credential and the model
                              and CLAUDE_CODE_DISABLE_* lines; the other ANTHROPIC_* lines
                              (custom headers among them) stay with the relay.
  image NAME                  makes the box's root image NAME unless docker's NAME is exactly
                              it: one layer holding the folders the mounts go in and the
                              /usr-merged symlinks and an empty configuration, made here, so no
                              registry is reached."""

NAME = "box_relay"
DEFAULT_BASE = "https://api.anthropic.com"
BASE_URL = "ANTHROPIC_BASE_URL"
API_KEY = "ANTHROPIC_API_KEY"
BEARER_SETTING = "ANTHROPIC_AUTH_TOKEN"
CUSTOM_HEADERS = "ANTHROPIC_CUSTOM_HEADERS"
PLACEHOLDER_KEY = "cleanroom-relay"
CREDENTIALS = FrozenList((API_KEY, BEARER_SETTING))
SETTINGS_THE_BOX_GETS = re.compile(
    r"(ANTHROPIC_(MODEL|SMALL_FAST_MODEL|DEFAULT_[A-Z]+_MODEL)|CLAUDE_CODE_DISABLE_[A-Z_]+)"
)
CLIENT_HEADERS = re.compile(
    r"(content-type|accept|accept-encoding|accept-language|user-agent|x-app|anthropic-[a-z0-9-]+|x-stainless-[a-z0-9-]+|x-claude-[a-z0-9-]+)"
)
REPLACED_HEADERS = FrozenOrderedSet((
    "host",
    "content-length",
    "connection",
    "authorization",
    "x-api-key",
))
HOP_BY_HOP = FrozenOrderedSet((
    "connection",
    "keep-alive",
    "transfer-encoding",
    "content-length",
))
BETA = "anthropic-beta"
ENDPOINTS = FrozenList((
    FrozenList(("v1", "messages")),
    FrozenList(("v1", "messages", "count_tokens")),
))
QUERIES = FrozenList(("", "beta=true"))
CLI_BETAS = FrozenOrderedSet((
    "claude-code-20250219",
    "interleaved-thinking-2025-05-14",
    "context-management-2025-06-27",
    "per-turn-control-2026-07-01",
    "mid-conversation-tool-changes-2026-07-01",
    "thinking-token-count-2026-05-13",
    "effort-2025-11-24",
    "thinking-display-updates-2026-08-18",
))
MEMBERS = FrozenOrderedSet((
    "model",
    "messages",
    "system",
    "tools",
    "tool_choice",
    "metadata",
    "max_tokens",
    "thinking",
    "context_management",
    "output_config",
    "stream",
    "temperature",
    "top_p",
    "top_k",
    "stop_sequences",
))
BLOCKS = FrozenOrderedSet((
    "text",
    "image",
    "tool_use",
    "tool_result",
    "thinking",
    "redacted_thinking",
))
RESULT_BLOCKS = FrozenOrderedSet(("text", "image"))
SYSTEM_BLOCKS = FrozenOrderedSet(("text",))
HELPER_SECONDS = 300
MAX_BODY = 64 << 20
SCAN_TARGET = "/cleanroom/scan"
SCAN_CALLS = 100
SCAN_BYTES = 4 << 20
SCAN_CALL_BYTES = 256 << 10
SCAN_SESSION_BYTES = 2 << 20
SCAN_FILES = 64
SCAN_SESSION_FILES = 1024
SCAN_LINES = 200
SCAN_PATH = re.compile(r"crates/clean/[A-Za-z0-9_./-]+")
SCAN_ANSWER = re.compile(r"(\S+)\t([0-9]+)\t([0-9]+(?:,[0-9]+)*)?\t([0-9]+)")
SCANNER_EXITS = FrozenOrderedSet((0, 1))
OK = 200
BAD_REQUEST = 400
TOO_LARGE = 413
TOO_MANY = 429
FAILED = 500

type Table = Mapping[str, object]


def is_table(value: object) -> TypeIs[Table]:
    return isinstance(value, Mapping)


def is_list(value: object) -> TypeIs[list[object]]:
    return isinstance(value, list)


@dataclass(frozen=True, slots=True)
class ScanFile:
    path: str
    head: str


@dataclass(frozen=True, slots=True)
class ScanAnswer:
    path: str
    hits: str
    lines: str
    new_bytes: int


@dataclass(frozen=True, slots=True)
class ScanSession:
    calls: int = 0
    spent: int = 0
    files: int = 0
    answered: FrozenOrderedSet[str] = field(
        default_factory=FrozenOrderedSet[str]
    )


@dataclass(frozen=True, slots=True)
class Charge:
    session: ScanSession
    cost: int
    spent: int
    counted: int
    fresh: int


@dataclass(frozen=True, slots=True)
class ScanReply:
    status: int
    text: str
    note: str


@dataclass(frozen=True, slots=True)
class Auth:
    env: FrozenOrderedDict[str, str]
    helper: str | None


@dataclass(frozen=True, slots=True)
class Upstream:
    scheme: str
    host: str
    port: int | None
    netloc: str
    path: str


@dataclass(frozen=True, slots=True)
class BoxEnv:
    lines: FrozenOrderedDict[str, str]
    kept: FrozenList[str]
    refusal: str | None


@dataclass(frozen=True, slots=True)
class Forwarded:
    headers: FrozenOrderedDict[str, str]
    unlisted_betas: FrozenList[str]
    dropped: FrozenList[str]


class RepeatedMemberError(ValueError):
    def __init__(self, member: str) -> None:
        super().__init__(member)
        self.member: str = member


def scan_file(value: object) -> ScanFile | None:
    if not is_table(value) or sorted(value) != ["head", "path"]:
        return None
    path = value["path"]
    head = value["head"]
    if not isinstance(path, str) or not isinstance(head, str):
        return None
    if not SCAN_PATH.fullmatch(path) or ".." in path.split("/"):
        return None
    return ScanFile(path, head)


def scan_files(body: bytes) -> FrozenList[ScanFile] | None:
    try:
        request = tomllib.loads(body.decode("utf-8"))
    except UnicodeDecodeError, tomllib.TOMLDecodeError:
        return None
    files = request.get("file")
    if list(request) != ["file"] or not is_list(files):
        return None
    found = FrozenList(scan_file(item) for item in files)
    valid = FrozenList(item for item in found if item is not None)
    return valid if len(valid) == len(found) else None


def asked(file: ScanFile) -> str:
    return hashlib.sha256(
        json.dumps([file.path, file.head]).encode()
    ).hexdigest()


def hit_line_prefixes(model: str) -> FrozenList[str]:
    root = model.rstrip("/")
    return FrozenList((f"{root}/src/", f"{root}/tests/"))


def admitted(call: int, body: bytes) -> FrozenList[ScanFile] | ScanReply:
    files = scan_files(body)
    if call > SCAN_CALLS:
        return ScanReply(
            TOO_MANY,
            f"the name scan answers {SCAN_CALLS} calls a session",
            f"call {call}: refused, over {SCAN_CALLS} a session",
        )
    if files is None:
        return ScanReply(
            BAD_REQUEST,
            "not a scan request",
            f"call {call}: refused, not a scan request",
        )
    if len(files) > SCAN_FILES:
        return ScanReply(
            TOO_LARGE,
            f"a scan call holds at most {SCAN_FILES} files",
            f"call {call}: refused, {len(files)} files, over {SCAN_FILES} a call",
        )
    return files


def scanner_answers(
    call: int,
    files: Sequence[ScanFile],
    returncode: int,
    stdout: bytes,
    stderr: bytes,
) -> FrozenList[ScanAnswer] | ScanReply:
    lines = stdout.decode("utf-8", "replace").splitlines()
    matched = FrozenList(
        found for line in lines if (found := SCAN_ANSWER.fullmatch(line))
    )
    answers = FrozenList(
        ScanAnswer(
            found.group(1),
            found.group(2),
            found.group(3) or "",
            int(found.group(4)),
        )
        for found in matched
    )
    paths = FrozenList(file.path for file in files)
    if (
        returncode not in SCANNER_EXITS
        or len(matched) != len(lines)
        or FrozenList(answer.path for answer in answers) != paths
    ):
        failure = stderr.decode("utf-8", "replace")[:200]
        return ScanReply(
            FAILED,
            "the name scan failed",
            f"call {call}: the scanner failed: {failure}",
        )
    return answers


def charge(
    session: ScanSession,
    files: Sequence[ScanFile],
    answers: Sequence[ScanAnswer],
) -> Charge:
    fresh = FrozenList(
        (file, answer)
        for file, answer in zip(files, answers, strict=False)
        if asked(file) not in session.answered
    )
    cost = sum(answer.new_bytes for _, answer in fresh)
    spent = session.spent + cost
    counted = session.files + len(fresh)
    within = (
        cost <= SCAN_CALL_BYTES
        and spent <= SCAN_SESSION_BYTES
        and counted <= SCAN_SESSION_FILES
    )
    after = (
        ScanSession(
            session.calls,
            spent,
            counted,
            session.answered
            | FrozenOrderedSet(asked(file) for file, _ in fresh),
        )
        if within
        else session
    )
    return Charge(after, cost, spent, counted, len(fresh))


def hit_lines(file: ScanFile, answer: ScanAnswer) -> str:
    text = file.head.splitlines()
    hit = "\n".join(
        f"    {n}: {text[int(n) - 1] if 0 < int(n) <= len(text) else ''}"
        for n in answer.lines.split(",")
        if n
    )
    return f"  {file.path}: {answer.hits} hits\n{hit}"


def over_budget(call: int, charged: Charge) -> ScanReply | None:
    if charged.cost > SCAN_CALL_BYTES:
        return ScanReply(
            TOO_LARGE,
            f"a scan call holds at most {SCAN_CALL_BYTES} bytes of new lines",
            f"call {call}: refused, {charged.cost} new bytes, over {SCAN_CALL_BYTES} a call",
        )
    if charged.spent > SCAN_SESSION_BYTES:
        return ScanReply(
            TOO_MANY,
            f"the name scan reads {SCAN_SESSION_BYTES} bytes of new lines a session",
            f"call {call}: refused, {charged.cost} new bytes, over {SCAN_SESSION_BYTES} a session",
        )
    if charged.counted > SCAN_SESSION_FILES:
        return ScanReply(
            TOO_MANY,
            f"the name scan reads {SCAN_SESSION_FILES} new files a session",
            f"call {call}: refused, {charged.fresh} new files, over {SCAN_SESSION_FILES} a session",
        )
    return None


def scan_reply(
    call: int,
    answered: Sequence[tuple[ScanFile, ScanAnswer]],
    body_bytes: int,
    charged: Charge,
    hit_lines_for: FrozenList[str],
) -> ScanReply:
    if refused := over_budget(call, charged):
        return refused
    note = [
        (
            f"call {call}: {len(answered)} files, {body_bytes} bytes, {charged.cost} new, {charged.spent} new this session"
        )
    ]
    reply: list[str] = []
    for file, answer in answered:
        numbers = (
            answer.lines.split(",")[:SCAN_LINES]
            if answer.lines and file.path.startswith(hit_lines_for)
            else []
        )
        reply.append(f"{answer.path}\t{answer.hits}\t{','.join(numbers)}\n")
        if answer.hits != "0":
            note.append(hit_lines(file, answer))
    return ScanReply(OK, "".join(reply), "\n".join(note))


def blocks_refusal(
    blocks: object, allowed: FrozenOrderedSet[str]
) -> str | None:
    if not is_list(blocks):
        return "content that is not a list"
    for block in blocks:
        kind = block.get("type") if is_table(block) else None
        if kind not in allowed:
            return f"the content block {kind!r}"
        if not is_table(block):
            continue
        source = block.get("source")
        if kind == "image" and not (
            is_table(source) and source.get("type") == "base64"
        ):
            return "an image that is not inline"
        content = block.get("content")
        if (
            kind == "tool_result"
            and not isinstance(content, str)
            and content is not None
            and (why := blocks_refusal(content, RESULT_BLOCKS))
        ):
            return why
    return None


def unique(pairs: Sequence[tuple[str, object]]) -> dict[str, object]:
    keys = FrozenList(key for key, _ in pairs)
    if len(FrozenOrderedSet(keys)) != len(keys):
        raise RepeatedMemberError(next(k for k in keys if keys.count(k) > 1))
    return dict(pairs)


def decoded_body(body: bytes) -> object:
    return cast("object", json.loads(body, object_pairs_hook=unique))


def tools_refusal(tools: object) -> str | None:
    if not is_list(tools):
        return "tools that are not a list"
    for tool in tools:
        kind = tool.get("type", "custom") if is_table(tool) else None
        if kind != "custom":
            return f"the tool type {kind!r}"
    return None


def messages_refusal(messages: object) -> str | None:
    if not is_list(messages):
        return "messages that are not a list"
    for message in messages:
        content = message.get("content") if is_table(message) else None
        if not isinstance(content, str) and (
            why := blocks_refusal(content, BLOCKS)
        ):
            return why
    return None


def request_object(body: bytes) -> Table | str:
    try:
        request = decoded_body(body)
    except RepeatedMemberError as repeated:
        return f"the repeated member {repeated.member!r}"
    except ValueError:
        return "a body that is not JSON"
    return request if is_table(request) else "a body that is not a JSON object"


def refusal(body: bytes) -> str | None:
    request = request_object(body)
    if isinstance(request, str):
        return request
    if extra := sorted(key for key in request if key not in MEMBERS):
        return f"the request member {extra[0]!r}"
    if why := tools_refusal(request.get("tools", [])):
        return why
    system = request.get("system")
    if (
        not isinstance(system, str)
        and system is not None
        and (why := blocks_refusal(system, SYSTEM_BLOCKS))
    ):
        return why
    return messages_refusal(request.get("messages", []))


def auth_of(settings: object, process_key: str | None) -> Auth | str:
    if not is_table(settings):
        return f"{NAME}: AUTH is not a JSON object"
    env = settings.get("env", {})
    helper = settings.get("apiKeyHelper")
    if not is_table(env):
        return f"{NAME}: AUTH's env is not an object"
    if helper is not None and not isinstance(helper, str):
        return f"{NAME}: AUTH's apiKeyHelper is not a string"
    lines = {key: str(value) for key, value in env.items()}
    if API_KEY not in lines and process_key:
        lines[API_KEY] = process_key
    return Auth(FrozenOrderedDict(lines), helper)


def upstream(env: Mapping[str, str]) -> Upstream | str:
    url = urllib.parse.urlsplit(env.get(BASE_URL, DEFAULT_BASE))
    extras = (url.query, url.fragment, url.username, url.password)
    if url.scheme not in {"https", "http"} or not url.hostname or any(extras):
        return f"{NAME}: the base URL is not a plain http(s) URL"
    return Upstream(url.scheme, url.hostname, url.port, url.netloc, url.path)


def box_env(auth: Auth, base: Upstream, port: int) -> BoxEnv:
    env = auth.env
    lines = {
        key: value
        for key, value in env.items()
        if SETTINGS_THE_BOX_GETS.fullmatch(key)
    }
    kept = FrozenList(
        sorted(
            key
            for key in env
            if key not in lines and key not in CREDENTIALS and key != BASE_URL
        )
    )
    lines[BASE_URL] = f"http://127.0.0.1:{port}{base.path.rstrip('/')}"
    lines[API_KEY] = PLACEHOLDER_KEY
    credential = auth.helper or any(key in env for key in CREDENTIALS)
    refused = (
        None
        if credential
        else f"{NAME}: no credential: no apiKeyHelper, {API_KEY} or {BEARER_SETTING}"
    )
    return BoxEnv(FrozenOrderedDict(sorted(lines.items())), kept, refused)


def kept_line(kept: Sequence[str]) -> str:
    return f"{NAME}: kept outside the box: {', '.join(kept)}"


def custom_headers(env: Mapping[str, str]) -> FrozenOrderedDict[str, str]:
    found: dict[str, str] = {}
    for line in env.get(CUSTOM_HEADERS, "").splitlines():
        name, sep, value = line.partition(":")
        if sep and name.strip():
            found[name.strip()] = value.strip()
    return FrozenOrderedDict(found)


def credential_headers(
    custom: Mapping[str, str], env: Mapping[str, str], helper_value: str | None
) -> FrozenOrderedDict[str, str]:
    out = dict(custom)
    if helper_value is not None:
        out.update({
            "x-api-key": helper_value,
            "authorization": f"Bearer {helper_value}",
        })
        return FrozenOrderedDict(out)
    if BEARER_SETTING in env:
        out["authorization"] = f"Bearer {env[BEARER_SETTING]}"
    if API_KEY in env:
        out["x-api-key"] = env[API_KEY]
    return FrozenOrderedDict(out)


def base_prefix(base: Upstream) -> FrozenList[str]:
    return FrozenList(segment for segment in base.path.split("/") if segment)


def allowed_target(target: str, prefix: Sequence[str]) -> str | None:
    path, _, query = target.partition("?")
    if (
        any(
            path == "/" + "/".join((*prefix, *endpoint))
            for endpoint in ENDPOINTS
        )
        and query in QUERIES
    ):
        return target
    return None


def betas(values: Sequence[str]) -> tuple[FrozenList[str], FrozenList[str]]:
    seen = FrozenList(
        beta.strip()
        for value in values
        for beta in value.split(",")
        if beta.strip()
    )
    return (
        FrozenList(beta for beta in seen if beta in CLI_BETAS),
        FrozenList(beta for beta in seen if beta not in CLI_BETAS),
    )


def folded(headers: Sequence[tuple[str, str]]) -> bool:
    return any("\r" in value or "\n" in value for _, value in headers)


def has_header(headers: Sequence[tuple[str, str]], name: str) -> bool:
    return any(key.lower() == name for key, _ in headers)


def first_header(headers: Sequence[tuple[str, str]], name: str) -> str | None:
    return next((value for key, value in headers if key.lower() == name), None)


def explicit_length(
    headers: Sequence[tuple[str, str]], limit: int
) -> int | None:
    length = int(first_header(headers, "content-length") or 0)
    if length < 0 or length > limit or has_header(headers, "transfer-encoding"):
        return None
    return length


def forwarded(headers: Sequence[tuple[str, str]]) -> Forwarded:
    out = {
        key.lower(): value
        for key, value in headers
        if CLIENT_HEADERS.fullmatch(key.lower())
    }
    kept, unlisted = betas(
        FrozenList(value for key, value in headers if key.lower() == BETA)
    )
    _ = out.pop(BETA, None)
    if kept:
        out[BETA] = ",".join(kept)
    dropped = FrozenList(
        sorted(
            FrozenOrderedSet(
                key.lower()
                for key, _ in headers
                if not CLIENT_HEADERS.fullmatch(key.lower())
            )
            - REPLACED_HEADERS
        )
    )
    return Forwarded(FrozenOrderedDict(out), unlisted, dropped)


def with_credential(
    headers: Mapping[str, str], credential: Mapping[str, str]
) -> FrozenOrderedDict[str, str]:
    return FrozenOrderedDict({**headers, **credential})


def relayed_response_headers(
    headers: Sequence[tuple[str, str]],
) -> FrozenList[tuple[str, str]]:
    return FrozenList(
        (key, value) for key, value in headers if key.lower() not in HOP_BY_HOP
    )


def layer_id(tar: bytes) -> str:
    return f"sha256:{hashlib.sha256(tar).hexdigest()}"


def root_image_matches(inspected: str, layer: str) -> bool:
    layers, _, config = inspected.partition("\n")
    settings = cast("object", json.loads(config or "null"))
    empty = not settings or (is_table(settings) and not any(settings.values()))
    return cast("object", json.loads(layers or "null")) == [layer] and empty
