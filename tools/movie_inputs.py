#!/usr/bin/env python3
"""Import clean-start LSMV/BK2 movies to hexadecimal SNES controller words.

Adapted from crates/movie in archive/pre-transpile-2026-10-17.
"""
import argparse
import csv
from dataclasses import dataclass
from enum import Enum, auto
import hashlib
import json
import struct
import tomllib
from itertools import islice, zip_longest
from pathlib import Path
from typing import Any, cast, overload
from collections.abc import Iterable, Iterator, Mapping, Sequence
import zipfile

BUTTON_BITS = dict(zip(
    ('B', 'Y', 'Select', 'Start', 'Up', 'Down', 'Left', 'Right', 'A', 'X', 'L', 'R'),
    (0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010),
))

# A capture CSV row, and the same row with every field but `kind` as an integer.
Row = dict[str, str]
Event = dict[str, int]
# Video rows are either whole CSV rows or the two retained counters (possibly absent).
Video = Sequence[Mapping[str, Any]]


def recorded_eof(movie: Path, source: list[int], path: Path, metadata: Path,
                 expected_sha256: str | None) -> Any:
    """Authenticate an explicitly recorded lsnes read-only continuation."""
    data = metadata.read_bytes()
    if not expected_sha256 or hashlib.sha256(data).hexdigest() != expected_sha256:
        raise ValueError('EOF capture metadata SHA-256 mismatch')
    with zipfile.ZipFile(movie) as archive:
        if 'input' not in archive.namelist():
            raise ValueError('recorded lsnes EOF continuation requires an LSMV source')
    info = json.loads(data)
    if (info.get('version') != 1 or info.get('origin') != 'emu-readonly-continuation'
            or info.get('status') != 'complete'
            or info.get('movie_sha256') != hashlib.sha256(movie.read_bytes()).hexdigest()
            or info.get('declared_frames') != len(source)
            or info.get('first_movie_frame') != len(source) + 1
            or not isinstance(info.get('last_movie_frame'), int)
            or info['last_movie_frame'] <= len(source)):
        raise ValueError('unsupported or incomplete EOF capture')
    # This policy and flat controller layout belong to the audited rr2-beta23
    # binary/source, not arbitrary movies or another emulator's EOF behavior.
    if (info.get('emulator_sha256') != 'd2324a06e11881592227440790be95b01136a94a89c07fb28299bd8985173323'
            or info.get('source_archive_sha256') != 'e9c0336218a47586e85b7c38341114995b3b7cdd727baf7c09c307d82072ef25'):
        raise ValueError('unsupported EOF producer')
    for suffix in ('', '.polls.csv', '.events.csv', '.eof-polls.csv', '.eof-hardware.csv',
                   '.phases.csv', '.milestones.csv'):
        file = Path(str(path) + suffix)
        pin = info.get('files', {}).get(file.name, {})
        with file.open('rb') as stream:
            actual = hashlib.file_digest(stream, 'sha256').hexdigest()
        if file.stat().st_size != pin.get('size') or actual != pin.get('sha256'):
            raise ValueError('EOF capture file hash/size mismatch: ' + file.name)
    return info


def seeded_wait_slice(source: list[int], path: Path, seed: Path, count: int,
                      continuation: dict[str, Any] | None = None) -> list[int]:
    """Locate the captured NMI's completed wait, then reuse strict input alignment."""
    if not seed.name.endswith('-wram.bin') or count <= 0:
        raise ValueError('seed requires a -wram.bin file and a positive frame count')
    ram = seed.read_bytes()
    if len(ram) != 131072 or int.from_bytes(ram[0x998:0x99a], 'little') != 8:
        raise ValueError('seed must contain gameplay WRAM')
    with seed.with_name('rooms.csv').open(newline='') as file:
        matches = [row for row in csv.DictReader(file) if row['file'] == seed.name]
    if len(matches) != 1 or hashlib.sha256(ram).hexdigest() != matches[0]['sha256']:
        raise ValueError('seed is absent from its capture index or its hash differs')
    row = matches[0]
    frame = int(row['movie_frame'])
    room = int.from_bytes(ram[0x79b:0x79d], 'little')
    nmi = int.from_bytes(ram[0x5b6:0x5b8], 'little')
    with seed.with_name(seed.name.replace('-wram.bin', '-registers.csv')).open(newline='') as file:
        metadata = {row['name']: int(row['value']) for row in csv.DictReader(file)}
    if (metadata.get('seed_movie_frame') != frame or metadata.get('seed_room') != room
            or metadata.get('seed_nmi') != nmi):
        raise ValueError('seed register metadata does not match WRAM and capture index')
    with path.open(newline='') as file:
        video_frame = next(islice(csv.DictReader(file), max(0, frame - 1), max(0, frame)), None)
    instruction_boundary = metadata.get('seed_boundary_pc')
    if (video_frame is None or int(row['room'], 16) != room
            or int(video_frame['room']) != room
            or (not instruction_boundary and int(video_frame['nmi_after']) != nmi)):
        raise ValueError('seed does not match the supplied video capture')
    # The room observation occurs at the video boundary inside NMI. Complete
    # that NMI exactly as the verifier does, without guessing a video offset.
    if instruction_boundary:
        if instruction_boundary != 0x808348 or metadata.get('pbpc') != 0x808349 or metadata.get('e') != 0:
            raise ValueError('unsupported seed opcode-fetch boundary')
        boundary_frame = metadata['seed_boundary_frame']
        boundary_nmi = nmi
        after = metadata['seed_boundary_event']
    else:
        boundary_frame, boundary_nmi = frame + 1, (nmi + 1) % 65536
    boundary, first = None, 0
    with Path(str(path) + '.events.csv').open(newline='') as file:
        for index, event in enumerate(csv.DictReader(file)):
            first += event['kind'] in ('joypad_read', 'vblank_wait_exit', 'no_update_exit')
            reached = int(event['event']) > after if instruction_boundary else int(event['movie_frame']) > frame
            if reached and event['kind'] == 'wait_exit':
                boundary = index
                break
    if boundary is None:
        raise ValueError('capture has no completed wait after this room seed')
    if (int(event['movie_frame']) != boundary_frame or int(event['room']) != room
            or int(event['nmi']) != boundary_nmi):
        raise ValueError('unsupported room seed phase; no matching next NMI boundary')
    print(f'Seed input starts after original native read/work ordinal {first}, captured event {event["event"]}')
    return aligned_wait_prefix(source, path, count, boundary + 1, continuation)


@overload
def aligned_wait_prefix(source: list[int], path: Path, count: int | None, after_event: int = 0,
                        continuation: dict[str, Any] | None = None, map_output: None = None,
                        provenance: dict[str, Path] | None = None) -> list[int]: ...
@overload
def aligned_wait_prefix(source: list[int], path: Path, count: int | None, after_event: int = 0,
                        continuation: dict[str, Any] | None = None, *, map_output: Path,
                        provenance: dict[str, Path] | None = None) -> int: ...
def aligned_wait_prefix(source: list[int], path: Path, count: int | None, after_event: int = 0,
                        continuation: dict[str, Any] | None = None, map_output: Path | None = None,
                        provenance: dict[str, Path] | None = None) -> list[int] | int:
    """Replay observed wait-entry input; reject gaps instead of guessing lag removal."""
    def rows(suffix: str) -> Iterator[Row]:
        with Path(str(path) + suffix).open(newline='') as file:
            yield from csv.DictReader(file)
    if (Path(str(path) + '.error.txt').exists()
            or (count is None and map_output is None) or (count is not None and count <= 0)):
        raise ValueError('failed capture or invalid prefix length')
    video: list[dict[str, Any]] = []
    eof_polls: Iterator[Row] = rows('.eof-polls.csv') if continuation else iter(())
    for index, (frame, poll) in enumerate(zip_longest(rows(''), rows('.polls.csv'))):
        if frame is None or poll is None or (index >= len(source) and not continuation):
            raise ValueError('incomplete video/input capture')
        if (int(frame['input_frame']) != index or int(frame['movie_frame']) != index + 1
                or int(frame['input_callbacks']) != 1 or int(poll['movie_frame']) != index + 1
                or int(poll['callback']) != 1 or int(poll['subframe']) != 0):
            raise ValueError(f'unsupported or mismatched movie input at {index}')
        if index < len(source):
            if (int(poll['movie_subframe_index']) != index
                    or parse_lsmv(poll['movie_subframe']) != [source[index]]):
                raise ValueError(f'unsupported or mismatched movie input at {index}')
        else:
            observed = next(eof_polls, None)
            # blank_frame(true): frame-sync axis 0 is set, reset/delay axes
            # 1..3 and gamepad axes 4..15 are released. No input is appended.
            axes = ';'.join(f'{axis}={int(axis == 0)}' for axis in range(16))
            if (observed is None or poll['movie_subframe'] != ''
                    or int(poll['movie_subframe_index']) != len(source)
                    or int(observed['declared_frames']) != len(source)
                    or observed['readonly'] != '1' or observed['selected_axes'] != axes
                    or any(observed[key] != poll[key] for key in (
                        'movie_frame', 'callback', 'subframe', 'movie_subframe_index',
                        'nmi', 'joypad_latched'))):
                raise ValueError(f'unsupported or unobserved EOF input at {index}')
        # Only counter resets are used by the alignment state machine. Keeping
        # full video/poll/event dictionaries made a 100% import exceed 2 GiB.
        video.append({'nmi_before': frame.get('nmi_before'), 'nmi_after': frame.get('nmi_after')})
    if not video:
        raise ValueError('incomplete video/input capture')
    if continuation and (len(video) != continuation['last_movie_frame']
                         or next(eof_polls, None) is not None):
        raise ValueError('EOF capture window does not match its metadata')
    def observed_events() -> Iterator[Row]:
        hardware: Iterator[Row] = rows('.eof-hardware.csv') if continuation else iter(())
        for event in rows('.events.csv'):
            if (continuation and event['kind'] == 'joypad_read'
                    and int(event['movie_frame']) > len(source)):
                observed = next(hardware, None)
                if (observed is None
                        or any(observed[key] != event[key] for key in ('event', 'movie_frame', 'nmi'))
                        or observed['joypad_latched'] != event['joypad']
                        or int(observed['a']) != int(event['joypad'])
                        or int(observed['pbpc']) != 0x80946b):
                    raise ValueError('EOF hardware controller observation mismatch')
            yield event
        if next(hardware, None) is not None:
            raise ValueError('extra EOF hardware controller observations')
    events = observed_events()
    if map_output is not None:
        if after_event or count is not None or not provenance:
            raise ValueError('event maps require full boot capture and authenticated provenance')
        if not any(event['kind'] == 'joypad_begin' for event in rows('.events.csv')):
            raise ValueError('event map requires complete controller read boundaries')
        return write_work_event_map(video, events, path, map_output, provenance)
    # Checked above: a prefix without an event map has a count.
    assert count is not None
    def complete(result: list[int]) -> list[int]:
        # The previous eager read parsed the unused suffix too. Preserve its
        # CSV error detection without retaining a second whole capture.
        for _ in events:
            pass
        return result
    previous: Event | None
    if after_event:
        # The seed normalizer has finished this wait. Start a fresh room-local
        # continuity chain instead of requiring all earlier rooms to align.
        boundary = next(islice(events, after_event - 1, after_event), None)
        if boundary is None or boundary['kind'] != 'wait_exit':
            raise ValueError('seed input must begin after a completed normal wait')
        previous = {key: int(value) for key, value in boundary.items() if key != 'kind'}
        previous['nmi'] = (previous['nmi'] - 1) % 65536
        return complete(aligned_work_prefix(video, events, count, after_event, previous))
    if any(event['kind'] == 'joypad_begin' for event in rows('.events.csv')):
        return complete(aligned_work_prefix(video, events, count))
    result: list[int] = []
    pending: Event | None = None
    previous = None
    last_frame = 0
    for number, raw in enumerate(events, 1):
        kind = raw['kind']
        event = {key: int(value) for key, value in raw.items() if key != 'kind'}
        movie_frame = event['movie_frame']
        if event['event'] != number or not last_frame <= movie_frame <= len(video):
            raise ValueError('nonsequential boundary capture')
        last_frame = movie_frame
        if kind == 'joypad_read':
            continue
        if kind == 'wait_enter' and pending is None:
            if previous is None:
                continuous = event['nmi'] == 0 and event['state'] == 0
            else:
                continuous = event['nmi'] == previous['nmi']
                # The observed boot reset clears the counters between logo and title.
                reset = (previous['state'] == 0 and event['state'] == 1 and event['nmi'] == 0
                         and any(int(row['nmi_after']) < int(row['nmi_before'])
                                 for row in video[previous['movie_frame'] - 1:movie_frame]))
                continuous |= reset
            if not continuous or not 0 <= event['joypad'] <= 0xffff:
                raise ValueError(f'unobserved update before native step {len(result)}')
            pending = event
        elif kind == 'wait_exit' and pending is not None:
            if event['nmi'] != (pending['nmi'] + 1) % 65536:
                raise ValueError('wait did not contain exactly one acknowledged NMI')
            result.append(pending['joypad'])
            previous, pending = event, None
            if len(result) == count:
                return complete(result)
        else:
            raise ValueError(f'unsupported boundary {kind}; prefix remains unaligned')
    raise ValueError('capture ends before the requested aligned prefix')



@dataclass
class PendingWork:
    """Observed boundaries whose completion is required before a span settles.

    Door placement survives individual settled suspension boundaries until more_exit.
    """
    read: Event | None = None
    wait: Event | None = None
    lag: Event | None = None
    vblank: Event | None = None
    irq: Event | None = None
    no_update: Event | None = None
    wait_word: int | None = None
    no_update_nmis: int = 0
    door: bool = False
    gap_read: bool = False

    @property
    def settled(self) -> bool:
        return all(value is None for value in
                   (self.read, self.wait, self.lag, self.vblank, self.irq, self.no_update))

    @property
    def pending(self) -> bool:
        return not self.settled or self.door


class WorkStatus(Enum):
    INCOMPLETE = 'incomplete'
    GAP = 'gap'
    BLOCKED = 'blocked'
    SUPPORTED = 'supported'
    EVIDENCE = 'evidence'

    @property
    def rejects_prefix(self) -> bool:
        return self in (WorkStatus.GAP, WorkStatus.BLOCKED, WorkStatus.INCOMPLETE)


@dataclass(frozen=True)
class WorkSpan:
    """Inclusive event/video ranges and half-open native read ordinals.

    A word and verifier ordinal exist only within the uninterrupted qualified prefix.
    """
    event_begin: int
    event_end: int
    video_begin: int
    video_end: int
    native_read_begin: int
    native_read_end: int
    boundary: str
    status: WorkStatus
    verifier_step: int | None
    settled: bool
    word: int | None

    def as_row(self) -> dict[str, Any]:
        row: dict[str, Any] = dict(
            event_begin=self.event_begin, event_end=self.event_end,
            video_begin=self.video_begin, video_end=self.video_end,
            native_read_begin=self.native_read_begin, native_read_end=self.native_read_end,
            boundary=self.boundary, status=self.status.value,
            verifier_step='' if self.verifier_step is None else self.verifier_step,
            settled=self.settled)
        if self.word is not None:
            row['word'] = self.word
        return row


@dataclass
class WorkRanges:
    """Own the un-emitted range and the irreversible supported-prefix frontier."""
    reads: int = 0
    read_begin: int = 0
    first: Event | None = None
    last: Event | None = None
    step: int = 0
    blocked: bool = False

    def observe(self, event: Event, controller_read: bool) -> None:
        self.last = event
        if self.first is None:
            self.first = event
        self.reads += controller_read

    def span(self, boundary: str, settled: bool, word: int | None = None,
             incomplete: bool = False) -> WorkSpan:
        assert self.first is not None and self.last is not None
        gap = boundary == 'requested_nmi_work' or incomplete
        self.blocked |= gap
        supported = word is not None and not gap
        qualified = supported and not self.blocked
        status = (WorkStatus.INCOMPLETE if incomplete else WorkStatus.GAP if gap else
                  WorkStatus.BLOCKED if self.blocked else
                  WorkStatus.SUPPORTED if supported else WorkStatus.EVIDENCE)
        result = WorkSpan(
            self.first['event'], self.last['event'],
            self.first['movie_frame'], self.last['movie_frame'],
            self.read_begin, self.reads, boundary, status,
            self.step if qualified else None, settled, word if qualified else None)
        if qualified:
            self.step += 1
        self.first, self.read_begin = None, self.reads
        return result


def mapped_work_spans(video: Video, events: Iterable[Row], after_event: int = 0,
                       previous: Event | None = None, frame_start: int = 1,
                       native_read_start: int = 0) -> Iterator[WorkSpan]:
    """Map contiguous original event ranges; unsupported cadence never makes a step.

    Event/video endpoints are inclusive. Native read endpoints are zero-based,
    half-open original ordinals. No captured events or input words are rewritten.
    """
    last_frame = 0
    pending = PendingWork()
    ranges = WorkRanges(reads=native_read_start, read_begin=native_read_start)
    for number, raw in enumerate(events, after_event + 1):
        kind = raw['kind']
        event = {key: int(value) for key, value in raw.items() if key != 'kind'}
        frame = event['movie_frame']
        if (event['event'] != number or not last_frame <= frame <= frame_start + len(video) - 1
                or frame < frame_start):
            raise ValueError('nonsequential work-boundary capture')
        last_frame = frame
        ranges.observe(event, kind == 'joypad_read')
        if not 0 <= event['joypad'] <= 0xffff:
            raise ValueError('invalid observed input')
        if kind == 'no_update_enter' and not pending.door and pending.wait is None and pending.no_update is None:
            pending.no_update, pending.no_update_nmis = event, 0
        elif kind == 'lag_nmi' and pending.no_update is not None:
            pending.no_update_nmis += 1
        elif kind == 'no_update_exit' and pending.no_update is not None:
            if (pending.no_update_nmis != 1
                    or any(event[key] != pending.no_update[key] for key in ('nmi', 'state', 'room', 'joypad'))
                    or (event['including_lag'] - pending.no_update['including_lag']) % 65536 != 1):
                raise ValueError('no-update wait did not contain one unchanged-input lag NMI')
            word, pending.no_update = pending.no_update['joypad'], None
            yield ranges.span('no_update', pending.settled, word)
        elif kind in ('irq_enter', 'lag_nmi', 'door_dma', 'cre_decode_enter', 'cre_decode_exit',
                    'tiles_decode_enter', 'tiles_decode_exit', 'palette_decode_enter', 'palette_decode_exit'):
            pass  # Read-only ordering evidence; these hooks do not define input consumption.
        elif kind == 'wait_enter' and pending.wait is None and pending.read is None and pending.no_update is None:
            pending.wait, pending.wait_word = event, None
        elif kind == 'wait_exit' and pending.wait is not None:
            if (event['nmi'] != (pending.wait['nmi'] + 1) % 65536
                    or pending.read is not None or pending.wait_word is None):
                raise ValueError('normal wait did not contain one complete NMI')
            word, pending.wait, pending.wait_word = pending.wait_word, None, None
            yield ranges.span('normal_wait', pending.settled, word)
        elif kind == 'place_enter' and not pending.door and pending.wait is None:
            if event['state'] != 11:
                raise ValueError('placement outside door transition')
            pending.door = True
        elif kind == 'vblank_wait_enter' and pending.door and pending.vblank is None and pending.wait is None:
            pending.vblank = event
        elif kind == 'vblank_wait_exit' and pending.vblank is not None:
            if any(event[key] != pending.vblank[key] for key in ('nmi', 'including_lag', 'joypad')):
                raise ValueError('unsupported vblank wait timing')
            pending.lag, pending.vblank = event, None
            yield ranges.span('door_vblank', pending.settled, event['joypad'])
        # Requested NMIs outside supported suspension boundaries are work gaps.
        elif kind == 'joypad_begin' and pending.read is None and (
                pending.wait is not None or pending.door
                or all(value is None for value in (pending.lag, pending.vblank, pending.irq, pending.no_update))):
            continuous = previous is None and event['nmi'] == 0 and event['state'] == 0
            if previous is not None:
                continuous = event['nmi'] == (previous['nmi'] + 1) % 65536
                continuous |= (previous['state'] == 0 and event['state'] == 1
                               and event['nmi'] == 0
                               and any(int(row['nmi_after']) < int(row['nmi_before'])
                                       for row in video[previous['movie_frame'] - frame_start:frame - frame_start + 1]))
            if (not continuous or pending.lag is not None
                    or (pending.wait is not None and event['joypad'] != pending.wait['joypad'])):
                raise ValueError('unobserved NMI or input change')
            pending.gap_read = pending.wait is None and not pending.door
            pending.read = event
        elif kind == 'joypad_read' and pending.read is not None:
            # A video boundary can split one uninterrupted controller read.
            crossed = frame == pending.read['movie_frame'] + 1
            if (any(event[key] != pending.read[key] for key in ('nmi', 'including_lag'))
                    or frame not in (pending.read['movie_frame'], pending.read['movie_frame'] + 1)
                    or (crossed and event['event'] != pending.read['event'] + 1)):
                raise ValueError('incomplete controller read')
            word = pending.read['joypad']
            previous, pending.read = event, None
            if pending.gap_read:
                yield ranges.span('requested_nmi_work', pending.settled)
            elif pending.wait is not None:
                if pending.wait_word is not None:
                    raise ValueError('normal wait contains multiple controller reads')
                pending.wait_word = word
            else:
                yield ranges.span('door_controller', pending.settled, word)
        elif kind == 'scroll_irq_enter' and pending.door and pending.irq is None:
            if pending.lag is not None:
                if (any(event[key] != pending.lag[key] for key in ('nmi', 'x', 'y', 'joypad'))
                        or (event['including_lag'] - pending.lag['including_lag']) % 65536 != 1):
                    raise ValueError('loading did not contain exactly one lag NMI')
                pending.lag = None
            pending.irq = event
        elif kind == 'scroll_irq_exit' and pending.irq is not None:
            if event['scroll_frame'] != (pending.irq['scroll_frame'] + 1) % 65536:
                raise ValueError('unsupported door scrolling update')
            pending.irq = None
        elif kind in ('place_exit', 'more_enter', 'scroll_wait_enter', 'scroll_wait_exit') and pending.door:
            pass
        elif kind == 'more_exit' and pending.door and pending.lag is None and pending.irq is None:
            pending.door = False
        else:
            raise ValueError(f'unsupported work boundary {kind}')
        if (ranges.first is not None and kind == 'scroll_irq_exit'
                and pending.settled):
            yield ranges.span('door_settlement', pending.settled)
    if ranges.first is None and pending.pending:
        # A terminal marker owns no events when EOF immediately follows a
        # supported suspension boundary; its range is explicitly empty.
        # Something is pending only after an event, so `last` is set.
        assert ranges.last is not None
        ranges.first = dict(ranges.last, event=ranges.last['event'] + 1)
    if ranges.first is not None:
        yield ranges.span('pending_eof' if pending.pending else 'evidence', pending.settled, incomplete=pending.pending)


def mapped_work_events(video: Video, events: Iterable[Row], after_event: int = 0,
                       previous: Event | None = None, frame_start: int = 1,
                       native_read_start: int = 0) -> Iterator[dict[str, Any]]:
    """Compatibility rows for callers of the capture mapper."""
    for span in mapped_work_spans(video, events, after_event, previous,
                                  frame_start, native_read_start):
        yield span.as_row()


def aligned_work_prefix(video: Video, events: Iterable[Row], count: int, after_event: int = 0,
                        previous: Event | None = None, frame_start: int = 1) -> list[int]:
    """Publish only complete supported steps before the first native work gap."""
    result: list[int] = []
    for span in mapped_work_spans(video, events, after_event, previous, frame_start):
        if span.status.rejects_prefix:
            raise ValueError(f"unsupported native cadence at events {span.event_begin}.."
                             f"{span.event_end}, video {span.video_begin}..{span.video_end}; "
                             f"qualified prefix has {len(result)} steps")
        if span.word is not None:
            result.append(span.word)
        if len(result) >= count and span.settled:
            return result[:count]
    raise ValueError('capture ends before the requested complete work prefix')


def file_sha256(path: str | Path) -> str:
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write_work_event_map(video: Video, events: Iterable[Row], path: Path, output: Path,
                         provenance: dict[str, Path]) -> int:
    """Write lossless identity ranges, with separate original and supported-prefix pins.

    This is an import artifact, not replay evidence. In particular, supported
    rows after a gap cannot qualify an input or checkpoint prefix through it.
    """
    required = {'movie', 'rom', 'native', 'native_source'}
    if set(provenance) != required:
        raise ValueError('event map requires original movie/ROM/native/source handles')
    files = dict(provenance, video=path, polls=Path(str(path) + '.polls.csv'),
                 events=Path(str(path) + '.events.csv'))
    pins = {key + '_sha256': file_sha256(value) for key, value in files.items()}
    pins['importer_sha256'] = file_sha256(__file__)
    with zipfile.ZipFile(provenance['movie']) as archive:
        with archive.open('input') as stream:
            # ZipFile.open returns a ZipExtFile, which typeshed widens to IO[bytes].
            member = cast(zipfile.ZipExtFile, stream)
            pins['movie_input_sha256'] = hashlib.file_digest(member, 'sha256').hexdigest()
    prefix = hashlib.sha256()
    pending_prefix = prefix.copy()
    pending_steps = 0
    steps = reads = event_end = 0
    first_gap: WorkSpan | None = None
    fields = ('event_begin', 'event_end', 'video_begin', 'video_end',
              'native_read_begin', 'native_read_end', 'boundary', 'status',
              'verifier_step', 'settled')
    # Publish only after successful parsing and a second original identity check.
    temporary = output.with_name(output.name + '.pending')
    try:
        with temporary.open('w', encoding='ascii', newline='') as stream:
            writer = csv.DictWriter(stream, fields, extrasaction='ignore', lineterminator='\n')
            writer.writeheader()
            for span in mapped_work_spans(video, events):
                if span.event_begin != event_end + 1 or span.native_read_begin != reads:
                    raise ValueError('event map omits an original event or controller read')
                writer.writerow(span.as_row())
                event_end, reads = span.event_end, span.native_read_end
                if span.word is not None:
                    pending_prefix.update(f"{span.word:04X}\r\n".encode('ascii'))
                    pending_steps += 1
                if first_gap is None and span.status in (WorkStatus.GAP, WorkStatus.INCOMPLETE):
                    first_gap = span
                if first_gap is None and span.settled:
                    prefix, steps = pending_prefix.copy(), pending_steps
        if any(file_sha256(value) != pins[key + '_sha256'] for key, value in files.items()):
            raise ValueError('original capture changed during event map generation')
        pins['map_sha256'] = file_sha256(temporary)
        pins['canonical_input_prefix_sha256'] = prefix.hexdigest()
        lines = ['version = 1', f'qualified_steps = {steps}', f'original_events = {event_end}',
                 f'original_native_reads = {reads}', f'original_video_frames = {len(video)}',
                 f'map = {json.dumps(output.name)}',
                 f'complete = {str(first_gap is None).lower()}']
        lines += [f'{key} = "{value}"' for key, value in pins.items()]
        if first_gap is not None:
            lines += [f'first_gap_{key} = {getattr(first_gap, key)}' for key in
                      ('event_begin', 'event_end', 'video_begin', 'video_end',
                       'native_read_begin', 'native_read_end')]
        output.with_suffix(output.suffix + '.toml').write_text('\n'.join(lines) + '\n',
                                                             encoding='ascii', newline='\n')
        temporary.replace(output)
    finally:
        temporary.unlink(missing_ok=True)
    return steps


def snes9x_capture_inputs(movie: Path, rom: Path, capture: Path, expected_sha256: str,
                          seed: Path, count: int) -> list[int]:
    """Reuse the work-boundary aligner for the reviewed five-port SMV capture."""
    if count <= 0 or hashlib.sha256(capture.read_bytes()).hexdigest() != expected_sha256:
        raise ValueError('invalid Snes9x capture length or metadata SHA-256')
    info = json.loads(capture.read_text())
    data = movie.read_bytes()
    if (info.get('version') != 1 or info.get('status') != 'complete'
            or info.get('origin') != 'snes9x-v17-exact-exec'
            or info.get('emulator_sha256') != '5a51cbca6430516f6ca4611a453a4dc41c7c34992769cb50d9d0e340c153d882'
            or info.get('movie_sha256') != 'a92e393a6b65ffb7b0358f168579c8d7b4ed624cb3a3445a3d4be75469da5e04'
            or hashlib.sha256(data).hexdigest() != info['movie_sha256']
            or hashlib.sha256(rom.read_bytes()).hexdigest() != '12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72'):
        raise ValueError('unsupported original Snes9x capture')
    magic, version, _, _, last_frame = struct.unpack_from('<4s4I', data)
    state_at, inputs_at = struct.unpack_from('<II', data, 24)
    if (magic != b'SMV\x1a' or version != 1 or data[20:22] != bytes([31, 1])
            or not 32 <= state_at < inputs_at or inputs_at + (last_frame + 1) * 10 > len(data)):
        raise ValueError('unsupported five-port SMV layout')
    # Each declared record, including frame zero, retains all five ports.
    # Only the observed non-debug hardware read selects port 1 below.
    def ports(frame: int) -> tuple[int, ...]:
        if not 0 <= frame <= last_frame:
            raise ValueError('capture exceeds declared SMV frames')
        return struct.unpack_from('<5H', data, inputs_at + frame * 10)
    tables: dict[str, list[Row]] = {}
    for name in ('snes9x.frames.csv', 'snes9x.events.csv'):
        path = capture.with_name(name)
        pin = info['files'][name]
        if path.stat().st_size != pin['size'] or hashlib.sha256(path.read_bytes()).hexdigest() != pin['sha256']:
            raise ValueError('Snes9x capture companion hash mismatch')
        with path.open(newline='') as file:
            tables[name] = list(csv.DictReader(file))
    video, events = tables['snes9x.frames.csv'], tables['snes9x.events.csv']
    first, last = info['first_frame'], info['last_frame']
    if (first != 186800 or len(video) != last - first + 1
            or any(int(row['movie_frame']) != first + i or int(row['debug']) != 0 for i, row in enumerate(video))):
        raise ValueError('incomplete video capture or unsupported debug input')
    for number, row in enumerate(events, 1):
        frame = int(row['movie_frame'])
        if (int(row['event']) != number or not first <= frame <= last + 1
                or int(row['debug']) != 0):
            raise ValueError('nonsequential event capture or unsupported debug input')
        if row['kind'] == 'joypad_read' and int(row['joypad']) != ports(frame)[0]:
            raise ValueError('observed controller read differs from the original five-port SMV')
    ram = seed.read_bytes()
    if len(ram) != 131072 or hashlib.sha256(ram).hexdigest() != info['seed_wram_sha256']:
        raise ValueError('capture does not belong to this room seed')
    with seed.with_name(seed.name.replace('-wram.bin', '-registers.csv')).open(newline='') as file:
        regs = {row['name']: int(row['value']) for row in csv.DictReader(file)}
    room = int.from_bytes(ram[0x79b:0x79d], 'little')
    nmi = int.from_bytes(ram[0x5b6:0x5b8], 'little')
    if (regs.get('pbpc') != 0x8dc535 or regs.get('seed_movie_frame') != first
            or regs.get('seed_nmi') != nmi or regs.get('seed_room') != room
            or int(video[0]['pc']) != regs['pbpc'] or int(video[0]['nmi']) != nmi
            or int(video[0]['room']) != room):
        raise ValueError('seed does not match the original frame observation')
    boundary = next((row for row in events if row['kind'] == 'wait_exit'), None)
    if (boundary is None or int(boundary['movie_frame']) != first + 1
            or int(boundary['nmi']) != (nmi + 1) % 65536 or int(boundary['room']) != room):
        raise ValueError('no matching completed NMI after the actual seed PC')
    after = int(boundary['event'])
    previous = {key: int(value) for key, value in boundary.items() if key != 'kind'}
    previous['nmi'] = (previous['nmi'] - 1) % 65536
    result = aligned_work_prefix(video, (row for row in events[after:] if int(row['movie_frame']) <= last),
                                 count, after, previous, frame_start=first)
    normalization_reads = [row for row in events[:after] if row['kind'] == 'joypad_read']
    if len(normalization_reads) != 1 or int(normalization_reads[0]['joypad']) != result[0]:
        raise ValueError('normalization input must equal the first imported controller word')
    return result


def load(path: Path, rom: Path) -> list[int]:
    with zipfile.ZipFile(path) as movie:
        names = movie.namelist()
        if 'input' in names:
            for name in names:
                if name in ('savestate', 'savestate.anchor', 'movie.bin') or name.startswith(
                    ('moviesram.', 'ramcontent.', 'setting.')
                ) or (name.startswith('port') and name[4:].isdigit()):
                    raise ValueError(f'unsupported LSMV initialization: {name}')
            if movie.read('gametype').strip() != b'snes_ntsc' or movie.read('controlsversion').strip() != b'0':
                raise ValueError('unsupported LSMV game or controls version')
            expected = movie.read('rom.sha256').decode().strip().lower()
            if hashlib.sha256(Path(rom).read_bytes()).hexdigest() != expected:
                raise ValueError('movie ROM SHA-256 does not match')
            frames = parse_lsmv(movie.read('input').decode())
        else:
            if {'Core.bin', 'SaveRAM.bin'} & set(names):
                raise ValueError('stateful BK2 initialization is unsupported')
            frames = parse_bk2(movie.read('Input Log.txt').decode())
    if not frames:
        raise ValueError('empty movie')
    return frames


def parse_lsmv(text: str) -> list[int]:
    buttons = 'BYsSudlrAXLR'
    frames: list[int] = []
    for number, line in enumerate(text.splitlines(), 1):
        header, pad = line.split('|', 1)
        if header not in ('F.', 'F. 0 0') or len(pad) != 12:
            raise ValueError(f'LSMV line {number}: unsupported reset, subframe or controller')
        word = 0
        for index, (value, button) in enumerate(zip(pad, buttons)):
            if value not in ('.', ' ', button):
                raise ValueError(f'LSMV line {number}: invalid button {value!r}')
            if value == button:
                word |= 0x8000 >> index
        frames.append(word)
    return frames


def parse_bk2(text: str) -> list[int]:
    key = next(line[len('LogKey:'):] for line in text.splitlines() if line.startswith('LogKey:'))
    groups = [[name for name in group.split('|') if name] for group in key.split('#') if group]
    allowed = {'Reset', 'Power'} | {f'P{port} {name}' for port in (1, 2) for name in BUTTON_BITS}
    if any(name not in allowed for group in groups for name in group):
        raise ValueError('unsupported BK2 control')
    frames: list[int] = []
    for line in text.splitlines():
        if not line.startswith('|'):
            continue
        fields = line[1:-1].split('|')
        if not line.endswith('|') or len(fields) != len(groups):
            raise ValueError('invalid BK2 frame groups')
        word = 0
        for group, field in zip(groups, fields):
            if len(group) != len(field):
                raise ValueError('invalid BK2 button group')
            for name, value in zip(group, field):
                if value in ('.', ' '):
                    continue
                if not name.startswith('P1 '):
                    raise ValueError(f'active reset, power or extra controller: {name}')
                word |= BUTTON_BITS[name[3:]]
        frames.append(word)
    return frames


# Source body inserted into existing tools/movie_inputs.py, not another runner.
def check_native_phase_capture(receipt: Path, receipt_sha256: str, movie: Path, full_inputs: Path,
                               rom: Path, native: Path, native_source: Path, phase_file: Path,
                               stdout: Path, logger: Path, predecessor: Path) -> dict[str, Any]:
    """Authenticate native observations; never turn them into engine-step inputs.

    Root supplies the SHA-bound actual native job receipt after collecting exit0,
    closed handles and post-provenance. File syntax alone cannot prove that job.
    Original full input identities remain separate from any qualified work map.
    """
    if file_sha256(receipt) != receipt_sha256:
        raise ValueError('native phase receipt identity differs')
    with receipt.open('rb') as stream:
        info = tomllib.load(stream)
    paths = dict(movie=movie, full_inputs=full_inputs, rom=rom, native=native,
                 native_source=native_source, phase_file=phase_file, stdout=stdout,
                 logger=logger, predecessor=predecessor)
    if (info.get('version') != 1 or info.get('native_exit_code') != 0
            or info.get('active_processes') != 0 or info.get('native_closed') is not True
            or info.get('supervisor_closed') is not True or info.get('post_provenance') is not True):
        raise ValueError('native phase job has no successful closed post-provenance receipt')
    pins = {role: file_sha256(path) for role, path in paths.items()}
    if any(info.get(role + '_sha256') != pin for role, pin in pins.items()):
        raise ValueError('native phase original/source/input/loader/output identity differs')
    originals = {
        'e53b1ce15b3f01516eb64c68a6db2911f91bd97197eb05960779561be5c7cbe9':
            ('7c41d3b87288467bd0d737262e0ba8d33a4094bd8e37777084249c25cc5ae6cb', 129712),
        '1bd065d89b70c16efb6f9276e82b1c07fd57ec40095f030cc6efe6663458929c':
            ('80aeef2665bac31156312f205fdb6b03aec2738d36a7857f02938eb45c1e4de7', 222788),
        '1d217bb073309ae44ea3d972c5ec3201be0ec434c60f034e1e958f94fd1bcda8':
            ('049d86bc515ecf3bdb4e65024aa21364e99074ce87c342ff3d2345eafb7f846b', 167797),
    }
    original = originals.get(pins['movie'])
    if original is None or pins['full_inputs'] != original[0]:
        raise ValueError('native phase does not belong to a complete original any/100/low input stream')
    # Count independently: a video ordinal is never relabelled as an engine step.
    with full_inputs.open('rb') as stream:
        words = sum(1 for _ in stream)
    if words != original[1] or info.get('full_input_words') != words:
        raise ValueError('native phase full original input identity/count differs')
    # Receipt values stay untyped until the int checks below.
    scope: list[Any] = [info.get(name) for name in ('declared_frames', 'first', 'through', 'events')]
    declared, first, through, events = scope
    if (any(type(value) is not int for value in (declared, first, through, events))
            or declared != words or not 1 <= first <= through or events <= 0):
        raise ValueError('native phase selected original scope is invalid')
    header = (f"# native_phase=1 parent={pins['movie']} native={pins['native']} "
              f"source={pins['native_source']} full_input={pins['full_inputs']} "
              f"declared={declared} first={first} through={through}\n")
    observed: Any = info.get('observed_native_frames')
    if type(observed) is not int or observed < declared or through > observed:
        raise ValueError('native phase original EOF has not actually been observed')
    selected_end = through
    last_index: Any = info.get('last_frame_first_subframe_index')
    if type(last_index) is not int or last_index < 0:
        raise ValueError('native phase lacks its observed original EOF input cursor')
    footer = (f'# native_collection_complete=true original_frames={declared} observed_native_frames={observed} selected_first={first} '
              f'selected_through={through} selected_events={events} last_frame_first_subframe_index={last_index} '
              'full_original_input_end=true physical_plan_admitted=false END_NATIVE_PHASE_COLLECTION\n')
    closed = (f"NATIVE_PHASE_WRITER_CLOSED parent={pins['movie']} native={pins['native']} "
              f"source={pins['native_source']} original_frames={declared} frames={observed} first={first} through={through} events={events}\n")
    with stdout.open('r', encoding='utf8', newline='') as text:
        if sum(line in (closed, closed[:-1] + '\r\n') for line in text) != 1:
            raise ValueError('native phase writer has no exact successful flush/close receipt')
    count = 0
    frames: set[int] = set()
    latch: tuple[int, list[int], tuple[str, str]] | None = None
    complete_latches = 0
    unknown_origins = 0
    native_ports: set[tuple[str, str]] = set()
    expected_columns = ['sequence', 'kind', 'address', 'value', 'movie_frame', 'input_callback',
        'frame_first_subframe_index', 'port0_type', 'port1_type', 'port2_type', 'frame_start_serialized',
        'api', 'source_hash', 'native_port1_type', 'native_port2_type', 'origin', 'reset_generation',
        'completed_master_clocks', 'frame_boundaries', 'pbpc', 'hcounter', 'vcounter', 'field',
        'smp_relative_clock', 'ppu_relative_clock', 'port1_relative_clock', 'port2_relative_clock',
        'auto_counter', 'auto_active', 'auto_enabled', 'joy1', 'joy2', 'joy3', 'joy4']
    def integer(row: Row, key: str, low: int, high: int) -> int:
        value = row[key]
        if not value or str(int(value)) != value or not low <= int(value) <= high:
            raise ValueError('native phase integer identity/range differs: ' + key)
        return int(value)
    with phase_file.open('r', encoding='ascii', newline='') as text:
        if text.readline() != header or text.readline() != ','.join(expected_columns) + '\n':
            raise ValueError('native phase exact original source/header differs')
        terminal = False
        for line in text:
            if line.startswith('#'):
                if line != footer or text.read(1):
                    raise ValueError('native phase has no exact terminal record at EOF')
                terminal = True
                break
            if len(line) > 4096 or not line.endswith('\n'):
                raise ValueError('native phase record is truncated or unbounded')
            values = next(csv.reader([line]))
            if len(values) != len(expected_columns):
                raise ValueError('native phase record shape differs')
            row = dict(zip(expected_columns, values))
            count += 1
            if integer(row, 'sequence', 1, 2**64 - 1) != count:
                raise ValueError('native phase skips or repeats an event')
            frame = integer(row, 'movie_frame', first, selected_end)
            if row['api'] != 'native-physical-v1' or row['source_hash'] != pins['native_source']:
                raise ValueError('native phase executing source/API differs')
            for key in ('smp_relative_clock', 'ppu_relative_clock', 'port1_relative_clock', 'port2_relative_clock'):
                integer(row, key, -(2**63), 2**63 - 1)
            if row['origin'] == 'actual_reset':
                for key in ('reset_generation', 'completed_master_clocks', 'frame_boundaries'):
                    integer(row, key, 1 if key == 'reset_generation' else 0, 2**64 - 1)
            elif row['origin'] == 'unserialized_origin' and all(row[key] == 'unknown_not_serialized'
                    for key in ('reset_generation', 'completed_master_clocks', 'frame_boundaries')):
                unknown_origins += 1
            else:
                raise ValueError('native phase invented an unobserved clock/frame origin')
            for key in ('hcounter', 'vcounter', 'joy1', 'joy2', 'joy3', 'joy4'):
                integer(row, key, 0, 65535)
            integer(row, 'field', 0, 1)
            pbpc = integer(row, 'pbpc', 0, 0xffffff)
            kind = row['kind']
            if kind == '0' and (integer(row, 'address', 0, 0xffffff) != 0x808028 or pbpc != 0x808028):
                raise ValueError('native upload phase is not genuine before-fetch808028')
            if kind == 'eof_input' and (frame <= declared or row['frame_start_serialized']):
                raise ValueError('native EOF input was fabricated as an original serialized record')
            if kind == 'input' and (frame > declared or not row['frame_start_serialized']):
                raise ValueError('native input callback lost its original all-port record')
            if kind == 'frame':
                if frame in frames:
                    raise ValueError('native phase repeats a selected original frame')
                frames.add(frame)
            if kind == '1':
                if latch is not None:
                    raise ValueError('native automatic latch is incomplete before the next latch')
                policy = integer(row, 'value', 0, 1)
                if integer(row, 'address', 0, 1) != policy:
                    raise ValueError('native latch raw counter/policy differs')
                latch = (policy, [], (row['native_port1_type'], row['native_port2_type']))
            if kind == '2':
                if latch is None:
                    raise ValueError('native automatic sample lacks an actual captured latch')
                policy, samples, ports = latch
                counter = integer(row, 'address', 0, 34)
                if counter != (len(samples) if policy == 0 else 3 + 2 * len(samples)):
                    raise ValueError('native automatic samples are missing/reordered/duplicated')
                samples.append(integer(row, 'value', 0, 15))
                if (row['native_port1_type'], row['native_port2_type']) != ports:
                    raise ValueError('native controller allocation type changed within a latch')
                if len(samples) == 16:
                    if any(port not in ('gamepad', 'none') for port in ports):
                        raise ValueError('native controller data device has no admitted two-port policy')
                    native_ports.add(ports)
                    complete_latches += 1
                    latch = None
            if kind == '3':
                address = integer(row, 'address', 0, 0xffffff)
                bank, offset = address >> 16, address & 0xffff
                if not (bank < 0x40 or 0x80 <= bank < 0xc0) or offset not in (0x4016, 0x4017, *range(0x4218, 0x4220)):
                    raise ValueError('native CPU controller read is not an actual port address')
                integer(row, 'value', 0, 255)
            if kind not in ('0', '1', '2', '3', '4', 'input', 'eof_input', 'frame'):
                raise ValueError('native phase event kind is unsupported')
        if not terminal or count != events or frames != set(range(first, selected_end + 1)) or latch is not None:
            raise ValueError('native phase selected frame/event/terminal/port scope is incomplete')
    if complete_latches == 0:
        raise ValueError('native phase contains no complete actual two-port latch')
    if any(file_sha256(path) != pins[role] for role, path in paths.items()) or file_sha256(receipt) != receipt_sha256:
        raise ValueError('native phase original/source/output changed during admission')
    return dict(original_input_words=words, selected_native_events=count,
                complete_two_port_latches=complete_latches, unknown_absolute_origins=unknown_origins,
                whole_original_observation=(first == 1 and selected_end >= declared),
                actual_eof_continuation_frames=observed - declared,
                physical_plan_admitted=False)


class MovieOperation(Enum):
    NATIVE_PHASE = auto()
    SNES9X = auto()
    EVENT_MAP = auto()
    ALIGN = auto()
    IMPORT = auto()


def movie_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('movie', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--rom', type=Path, required=True)
    parser.add_argument('--lag-log', type=Path, help='lsnes CSV with input and boundary companions')
    parser.add_argument('--frames', type=int, help='number of observed wait steps to import')
    parser.add_argument('--event-map-only', action='store_true',
                        help='write full original event ranges and supported-prefix manifest, not inputs')
    parser.add_argument('--native', type=Path, help='original capture executable for event-map provenance')
    parser.add_argument('--native-source', type=Path, help='original capture source archive for provenance')
    parser.add_argument('--seed', type=Path, help='start after this captured room WRAM seed')
    parser.add_argument('--eof-capture', type=Path, help='authenticated recorded read-only EOF continuation metadata')
    parser.add_argument('--eof-capture-sha256', help='required pinned EOF metadata SHA-256')
    parser.add_argument('--snes9x-capture', type=Path, help='authenticated original five-port SMV work capture')
    parser.add_argument('--snes9x-capture-sha256', help='required Snes9x metadata SHA-256')
    parser.add_argument('--native-phase-receipt', type=Path,
                        help='check authentic original native phase observations, without emitting inputs')
    parser.add_argument('--native-phase-receipt-sha256')
    parser.add_argument('--native-phase-inputs', type=Path)
    parser.add_argument('--native-phase-stdout', type=Path)
    parser.add_argument('--native-phase-logger', type=Path)
    parser.add_argument('--native-phase-predecessor', type=Path)
    return parser


def movie_operation(args: argparse.Namespace, parser: argparse.ArgumentParser) -> MovieOperation:
    """Resolve one operation after checking the existing CLI admission contract."""
    if args.native_phase_receipt:
        if (not all((args.native_phase_receipt_sha256, args.native_phase_inputs,
                     args.native_phase_stdout, args.native_phase_logger,
                     args.native_phase_predecessor, args.native, args.native_source))
                or args.lag_log or args.frames is not None or args.event_map_only
                or args.seed or args.eof_capture or args.eof_capture_sha256
                or args.snes9x_capture or args.snes9x_capture_sha256):
            parser.error('--native-phase-receipt requires actual closed native/source/input/output handles only')
        return MovieOperation.NATIVE_PHASE
    if any((args.native_phase_receipt_sha256, args.native_phase_inputs, args.native_phase_stdout,
            args.native_phase_logger, args.native_phase_predecessor)):
        parser.error('native phase options require --native-phase-receipt')
    if args.event_map_only:
        if (not args.lag_log or not args.native or not args.native_source or args.seed
                or args.frames is not None or args.snes9x_capture
                or args.snes9x_capture_sha256):
            parser.error('--event-map-only requires --lag-log, --native and --native-source, from boot')
    elif args.native or args.native_source:
        parser.error('--native and --native-source require --event-map-only')
    if args.snes9x_capture or args.snes9x_capture_sha256:
        if (not args.snes9x_capture or not args.snes9x_capture_sha256 or not args.seed
                or args.frames is None or args.lag_log or args.eof_capture
                or args.eof_capture_sha256):
            parser.error('--snes9x-capture requires its SHA-256, --seed and --frames only')
        return MovieOperation.SNES9X
    if bool(args.eof_capture) != bool(args.eof_capture_sha256) or (args.eof_capture and not args.lag_log):
        parser.error('--eof-capture requires --eof-capture-sha256 and --lag-log')
    if args.seed and not args.lag_log:
        parser.error('--seed requires --lag-log and --frames')
    if ((args.lag_log and args.frames is None and not args.event_map_only)
            or (args.frames is not None and not args.lag_log)):
        parser.error('aligned input import requires both --lag-log and --frames')
    if args.event_map_only:
        return MovieOperation.EVENT_MAP
    if args.lag_log:
        return MovieOperation.ALIGN
    return MovieOperation.IMPORT


def run_movie_operation(operation: MovieOperation, args: argparse.Namespace) -> None:
    """Execute an admitted request; parsing and flag conflicts require no file reads."""
    if operation is MovieOperation.NATIVE_PHASE:
        result = check_native_phase_capture(args.native_phase_receipt, args.native_phase_receipt_sha256,
            args.movie, args.native_phase_inputs, args.rom, args.native, args.native_source,
            args.output, args.native_phase_stdout, args.native_phase_logger, args.native_phase_predecessor)
        print(f"Native observation checked: {result['original_input_words']} original inputs; "
              f"{result['selected_native_events']} selected events; "
              f"{result['complete_two_port_latches']} actual two-port latches; physical plan not admitted")
        return
    if operation is MovieOperation.SNES9X:
        frames = snes9x_capture_inputs(args.movie, args.rom, args.snes9x_capture,
                                      args.snes9x_capture_sha256, args.seed, args.frames)
        args.output.write_text(''.join(f'{word:04X}\n' for word in frames),
                               encoding='ascii', newline='\r\n')
        print(f'Imported {len(frames)} observed controller frames from the five-port SMV')
        return
    frames = load(args.movie, args.rom)
    continuation = (recorded_eof(args.movie, frames, args.lag_log, args.eof_capture, args.eof_capture_sha256)
                    if args.eof_capture else None)
    if operation is MovieOperation.EVENT_MAP:
        steps = aligned_wait_prefix(frames, args.lag_log, None, continuation=continuation,
            map_output=args.output, provenance=dict(movie=args.movie, rom=args.rom,
                                                   native=args.native, native_source=args.native_source))
        print(f'Mapped original capture; supported input prefix has {steps} steps; no inputs emitted')
        return
    if operation is MovieOperation.ALIGN:
        frames = (seeded_wait_slice(frames, args.lag_log, args.seed, args.frames, continuation)
                  if args.seed else aligned_wait_prefix(frames, args.lag_log, args.frames, continuation=continuation))
    args.output.write_text(''.join(f'{word:04X}\n' for word in frames),
                           encoding='ascii', newline='\r\n')
    print(f'Imported {len(frames)} controller frames')


def main(argv: Sequence[str] | None = None) -> None:
    parser = movie_parser()
    args = parser.parse_args(argv)
    operation = movie_operation(args, parser)
    run_movie_operation(operation, args)


if __name__ == '__main__':
    main()
