#!/usr/bin/env python3
"""Import clean-start LSMV/BK2 movies to hexadecimal SNES controller words.

Adapted from crates/movie in archive/pre-transpile-2026-10-17.
"""
import argparse
import csv
import hashlib
from pathlib import Path
import zipfile

BUTTON_BITS = dict(zip(
    ('B', 'Y', 'Select', 'Start', 'Up', 'Down', 'Left', 'Right', 'A', 'X', 'L', 'R'),
    (0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010),
))


def aligned_wait_prefix(source, path, count):
    """Replay observed wait-entry input; reject gaps instead of guessing lag removal."""
    def rows(suffix):
        with Path(str(path) + suffix).open(newline='') as file:
            return list(csv.DictReader(file))
    if Path(str(path) + '.error.txt').exists() or count <= 0:
        raise ValueError('failed capture or invalid prefix length')
    video, polls = rows(''), rows('.polls.csv')
    if not video or len(video) != len(polls) or len(video) > len(source):
        raise ValueError('incomplete video/input capture')
    for index, (frame, poll) in enumerate(zip(video, polls)):
        if (int(frame['input_frame']) != index or int(frame['movie_frame']) != index + 1
                or int(frame['input_callbacks']) != 1 or int(poll['movie_frame']) != index + 1
                or int(poll['callback']) != 1 or int(poll['subframe']) != 0
                or int(poll['movie_subframe_index']) != index
                or parse_lsmv(poll['movie_subframe']) != [source[index]]):
            raise ValueError(f'unsupported or mismatched movie input at {index}')
    events = rows('.events.csv')
    if any(event['kind'] == 'joypad_begin' for event in events):
        return aligned_work_prefix(video, events, count)
    result, pending, previous = [], None, None
    last_frame = 0
    for number, event in enumerate(events, 1):
        kind = event['kind']
        event = {key: int(value) for key, value in event.items() if key != 'kind'}
        frame = event['movie_frame']
        if event['event'] != number or not last_frame <= frame <= len(video):
            raise ValueError('nonsequential boundary capture')
        last_frame = frame
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
                                 for row in video[previous['movie_frame'] - 1:frame]))
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
                return result
        else:
            raise ValueError(f'unsupported boundary {kind}; prefix remains unaligned')
    raise ValueError('capture ends before the requested aligned prefix')


def aligned_work_prefix(video, events, count):
    """Preserve observed normal updates and the loading lag suspension."""
    result, last_frame = [], 0
    read = wait = lag = vblank = irq = previous = None
    door = False
    for number, raw in enumerate(events, 1):
        kind = raw['kind']
        event = {key: int(value) for key, value in raw.items() if key != 'kind'}
        frame = event['movie_frame']
        if event['event'] != number or not last_frame <= frame <= len(video):
            raise ValueError('nonsequential work-boundary capture')
        last_frame = frame
        if not 0 <= event['joypad'] <= 0xffff:
            raise ValueError('invalid observed input')
        if kind in ('irq_enter', 'lag_nmi', 'door_dma', 'cre_decode_enter', 'cre_decode_exit',
                    'tiles_decode_enter', 'tiles_decode_exit', 'palette_decode_enter', 'palette_decode_exit'):
            pass  # Read-only ordering evidence; these hooks do not define input consumption.
        elif kind == 'wait_enter' and wait is None:
            wait = event
        elif kind == 'wait_exit' and wait is not None:
            if event['nmi'] != (wait['nmi'] + 1) % 65536 or read is not None:
                raise ValueError('normal wait did not contain one complete NMI')
            wait = None
        elif kind == 'place_enter' and not door and wait is None:
            if event['state'] != 11:
                raise ValueError('placement outside door transition')
            door = True
        elif kind == 'vblank_wait_enter' and door and vblank is None and wait is None:
            vblank = event
        elif kind == 'vblank_wait_exit' and vblank is not None:
            if any(event[key] != vblank[key] for key in ('nmi', 'including_lag', 'joypad')):
                raise ValueError('unsupported vblank wait timing')
            lag, vblank = event, None
            result.append(event['joypad'])
        elif kind == 'joypad_begin' and read is None and (wait is not None or door):
            continuous = previous is None and event['nmi'] == 0 and event['state'] == 0
            if previous is not None:
                continuous = event['nmi'] == (previous['nmi'] + 1) % 65536
                continuous |= (previous['state'] == 0 and event['state'] == 1
                               and event['nmi'] == 0
                               and any(int(row['nmi_after']) < int(row['nmi_before'])
                                       for row in video[previous['movie_frame'] - 1:frame]))
            if (not continuous or lag is not None
                    or (wait is not None and event['joypad'] != wait['joypad'])):
                raise ValueError('unobserved NMI or input change')
            read = event
        elif kind == 'joypad_read' and read is not None:
            # A video boundary can split one uninterrupted controller read.
            crossed = frame == read['movie_frame'] + 1
            if (any(event[key] != read[key] for key in ('nmi', 'including_lag'))
                    or frame not in (read['movie_frame'], read['movie_frame'] + 1)
                    or (crossed and event['event'] != read['event'] + 1)):
                raise ValueError('incomplete controller read')
            result.append(read['joypad'])
            previous, read = event, None
        elif kind == 'scroll_irq_enter' and door and irq is None:
            if lag is not None:
                if (any(event[key] != lag[key] for key in ('nmi', 'x', 'y', 'joypad'))
                        or (event['including_lag'] - lag['including_lag']) % 65536 != 1):
                    raise ValueError('loading did not contain exactly one lag NMI')
                lag = None
            irq = event
        elif kind == 'scroll_irq_exit' and irq is not None:
            if event['scroll_frame'] != (irq['scroll_frame'] + 1) % 65536:
                raise ValueError('unsupported door scrolling update')
            irq = None
        elif kind in ('place_exit', 'more_enter', 'scroll_wait_enter', 'scroll_wait_exit') and door:
            pass
        elif kind == 'more_exit' and door and lag is None and irq is None:
            door = False
        else:
            raise ValueError(f'unsupported work boundary {kind}')
        if len(result) >= count and all(value is None for value in (read, wait, lag, vblank, irq)):
            return result[:count]
    raise ValueError('capture ends before the requested complete work prefix')


def load(path, rom):
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


def parse_lsmv(text):
    buttons = 'BYsSudlrAXLR'
    frames = []
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


def parse_bk2(text):
    key = next(line[len('LogKey:'):] for line in text.splitlines() if line.startswith('LogKey:'))
    groups = [[name for name in group.split('|') if name] for group in key.split('#') if group]
    allowed = {'Reset', 'Power'} | {f'P{port} {name}' for port in (1, 2) for name in BUTTON_BITS}
    if any(name not in allowed for group in groups for name in group):
        raise ValueError('unsupported BK2 control')
    frames = []
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


# Original any% movie in movies/manifest.toml (TASVideos publication3653).
ROUTE_SOURCE_SHA256 = 'e53b1ce15b3f01516eb64c68a6db2911f91bd97197eb05960779561be5c7cbe9'
ROUTES = {
    'ceres-escape': (20989, '0f906b2af9e5fa5856b14efe4197a76399b3cb6f5cd20795c9b30eabede5cf0b'),
    'morph-ball': (28301, 'a1470d7c61e371e42d2ef8c6a874d09f5c7d6ab2350c809812ea94d35f4870a5'),
    'missile-pickup': (32591, '6b6153b5691992b341a85f744dcd0fbcddc1499f7b9be8309fc587c159053e86'),
    'parlor-return': (42651, '109ebae783ee29eaac7b22c7e87fbcd3050dff1bf3ff8a4d29dfff61cde43d50'),
    'bomb-corridor-entry': (46133, '9176ce688474956cd965f9780220db3b05204ed23980ae3d8d205c5e6a24cd4b'),
    'bomb-pickup': (50329, '245a5f8add96661d26982360b233d9f682ff44e529f4a24ee3f0e70e7dc4bad9'),
    'bomb-torizo': (52820, 'c133839513c05faa67d69057eb22db32d03a3ae6cd0ec89e0ae6fa25d48727dd'),
    'parlor-hill-top': (56786, '5e6e409166f00cc5c40d82cbd4ddc5ec4af6bb165016b174f3ef34190103c9ac'),
    'gunship-refilled': (60402, '091688e6bda5853a376ddc49d237a107a05a34623482de6be5a888045554cd16'),
    'green-brinstar-arrival': (67278, '847d107b875dfd80251a53b2d53cee2cd41ea9f91cc1b8e19e21fbfb773d7777'),
    'early-supers-door': (68096, 'cc6c83bd7f3a3925a520bd4bbda2bb8c2cefc3287f1625d91d9d95d9faf35125'),
    'super-missiles': (69700, '26ceb1e7ee4bcd20821a09d80d518566755626bf5d4b344b9ea8e2dbb0ae846a'),
    'green-brinstar-save': (71800, 'e060c7f2408b95180cdd98c091821775e3c0c279c89131256962ddd68d0b4554'),
    'save-death-continue': (76064, 'af79d24c791c1414cd4b1b83f040da0787a2a69bd0d2c4d6b73fc667b7d74467'),
    'charge-beam': (78380, '9e7a8563adf61d0b1a62f864664df066d64fc7abcfa3cd3c08be60926c36d983'),
    'red-tower': (85800, '45bebd810937cf03267f5a053c7e5dd9cf002ad3f46f34cab2eb5bc68a3d411e'),
    'redtower-shaft-open': (86483, 'bebda37ee6cf6d8236f36719cb9e04b5ab65abefe070e49088d9e1df175947cb'),
    'spazer': (90706, 'f1e1f18b3ce11974f5049d2429960a4f07d5728307cd5dad7d202efe223f41f5'),
    'hi-jump': (99191, 'c14c52e9860a306076f660edad39907deb3ffaabfd6d0670ead1d9f203492948'),
    'kraid-refilled': (115663, '0959e0716ab17f1386612db2be86f8444a76d7f8fe06cfadc90ab96953ccfcc8'),
    'kraid-phase2': (119823, '5f9574353373aecf84ecd3b44eb4cc9205478fbcad9bb6b3e851fae41728b971'),
    'kraid-defeated': (120820, 'a4a03b12d7491dba63661997e9db2a190ebbe15d8ce94bdf54760fea629c25ec'),
    'varia-suit': (122732, '53b375049fd8012820edd0d224df29782891893f553d5f3d690b5a3f079a3648'),
    'warehouse-return': (128547, '2fba8c0b64d929b6206ccb12d349d965d2f8bec60ebd504dfb5db36ded390d8a'),
    'warehouse-energy': (129802, '18f230ec4d4891261df5552bb84af789d3a2345d4e135f93c086d996c19dee5d'),
    'bubble-mountain': (138749, '06344dfe6f82874f0bca8b4fac2e7e712abb6f21540b2997edfd0599f85558da'),
    'ice-beam': (161613, 'd730b6de3ceef1bcff05d07af3775b679bd70c8c731ae128ed5b573e21be8d21'),
    'ice-return-business': (166082, '4b4090a970f52658d27d4f69fe0e6d30da43739519cf781ef722dacbecdb5721'),
    'ice-beam-settled': (161733, 'd5700c8c8bfd3bcbc9fa5d4d05aa805dd594e98130a0c9d0ade163695201e093'),
    'speed-booster': (148406, 'bc907fc429252418216b7f575e26be148f7cf863a71c3947b4ffa3c6c6027de6'),
    'speed-booster-settled': (148526, '9548c119742b6e2ffec65c5c0871403c96915671b83fe8b4c3930cc4bf60a00d'),
    'crocomire': (171694, '90223d25514f15bb92cfe255402f6561fe598b46dd7a07435dcecff682f3dabe'),
    'crocomire-shot': (167295, 'e46b9a61eeded83064cc3a9fc29cf05d6a1cd18151c06cd6c9b78930af41766b'),
    'wave-beam': (155607, '2f07dc07523b7b542412604337d961292c3c07e3c4f0577d06dd131e1e879c4f'),
    'wave-gate': (153648, '1ce640c865d508ff2aab0f79c043c6da8aff9b74d6a1444d3a994e20d68db123'),
    'power-bomb': (176055, 'b6c0c74ccfe38b10c1ae284472e60bc483060ca0b3fb6f359d57e894283decf7'),
    'power-bomb-room-entry': (174992, '34ab4055b6904d0544e490632fbbb81b87f18d163e55c7c6208ee9e5bd2a7dfb'),
    'norfair-reserve': (147887, '968f37d5218ea77d18ba81e5022ceca78b0c53be6b22f3602895e08904901c2d'),
}


def _buttons(names):
    return sum(BUTTON_BITS[name] for name in names.split())


def _hold(names, frames):
    return [_buttons(names)] * frames


def _holds(spec):
    # Original authored holds: B+Right+A:25 None:15; None releases all buttons.
    result = []
    for hold in spec.split():
        names, count = hold.split(':')
        result.extend(_hold('' if names == 'None' else names.replace('+', ' '), int(count)))
    return result


def _early_route(any_inputs):
    # Original session 01a14be2-7bfa-76f2-96e3-b9b914c73f46 ordinals 471/488/503/545/565.
    a = any_inputs[:11627] + (_hold('B Right A', 25) + _hold('B Right', 15))*80
    a = a[:12500] + _hold('B Left', 60) + _hold('B Right', 60) + _hold('B Left', 200) + _hold('B Right', 80) + _hold('B Left', 700)
    a = a[:12850] + (_hold('B Right A', 25) + _hold('B Right', 15))*75 + _hold('', 1500)
    a = a[:14000] + _hold('', 5000)
    a = a[:16600] + (_hold('B Left A', 25) + _hold('B Left', 15))*80
    # Session 01a14bf4-4ff1-7402-86ed-4aafaeb8c05c ordinal 236.
    a = a[:17685] + _hold('Right A', 35) + _hold('Right', 25) + _hold('Right A', 55) + _hold('Right', 20) + _hold('Left A', 65) + _hold('Left', 20) + _hold('Right A', 80) + _hold('Right', 80)
    a = a[:17865] + _hold('B Left A', 45) + _hold('B Left', 85) + _hold('B Left A', 40) + _hold('B Left', 150) + _hold('B Right', 30) + _hold('B Right A', 80) + _hold('B Right', 180) + _hold('', 120)
    # Session 01a14bfb-e6b5-7d42-9501-22158522a610; ordinals below.
    a = a[:18085] + _hold('B Left A', 315) + _hold('Left A', 500)  #106 return
    a = a[:18392]; a[-5:] = _hold('B Left', 5)
    a += _hold('B Left A', 65) + _hold('A', 65) + _hold('', 10)  #168 ledge-18392
    a = a[:18472]; a[-5:] = _hold('', 5)
    a += _hold('B Right', 16) + _hold('', 15) + _hold('A', 20) + _hold('B Right A', 45) + _hold('A', 35)  #328 center-stand-16
    a = a[:18580] + _hold('', 5) + _hold('A', 16) + _hold('B Left A', 35) + _hold('A', 45)  #365 left400-16
    a = a[:18665] + _hold('B Left', 7) + _hold('', 8) + _hold('B Left A', 16) + _hold('B Right A', 38) + _hold('A', 45)  #446 center-spin-rev-16
    a = a[:18745] + _hold('B Right', 7) + _hold('', 35) + _hold('A', 20) + _hold('B Left A', 40) + _hold('A', 40)  #520 left192-wait-20
    a = a[:18865] + _hold('', 5) + _hold('A', 20) + _hold('B Right A', 44) + _hold('A', 55) + _hold('', 2000)  #550 escape-20
    yield 'ceres-escape', list(a)
    a += _hold('B Left X', 600) + _hold('', 100)  #648 left
    a = a[:21590] + _hold('B Left X', 1200) + _hold('', 100)  #685 west
    a = a[:22005]; a[-5:] = _hold('B Left', 5)
    a += _hold('B Left A X', 240) + _hold('B Left X', 200) + _hold('', 50)  #736 parlor-22005
    a = a[:22250] + _hold('', 590)  #834 pit-stop
    a = a[:22300] + (_hold('B Left X', 35) + _hold('B Right X', 45))*15 + _hold('', 100)  #882 descent2
    a = a[:23500] + _hold('B Left', 20) + _hold('', 20) + _hold('A', 12) + _hold('Down A X', 80) + _hold('', 900)  #1063 floor-air
    a = a[:23800] + _hold('Left A', 50) + _hold('Left', 30) + _hold('Right', 60) + (_hold('Left', 45) + _hold('Right', 45))*10 + _hold('', 100)  #1139 mother-drop-left
    a = a[:24850] + _hold('B Right X', 400) + _hold('', 100)  #1176 bottom-right
    a = a[:25200] + _hold('', 5) + _hold('B Right A X', 200) + _hold('B Right X', 250) + _hold('', 100)  #1225 corridor-jump
    a = a[:25600] + (_hold('', 5) + _hold('B Right A X', 65) + _hold('B Right X', 40))*5 + _hold('', 100)  #1264 jumps
    a += _hold('Right', 30) + _hold('', 20) + _hold('Down', 1) + _hold('', 1200)  #1304 elevator-30
    a = a[:27200] + _hold('B Left X', 180) + _hold('', 700) + _hold('A', 1) + _hold('', 100)  #1350 morph-left
    a = a[:27255] + _hold('', 5) + _hold('B Right A', 22) + _hold('B Left A', 85) + _hold('', 700) + _hold('A', 1) + _hold('', 100)  #1488 wallloop-27255
    a = a[:27350] + _hold('', 5) + _hold('B Left A', 100) + _hold('', 600) + _hold('A', 1) + _hold('', 120)  #1527 morph-pedestal-27350
    a = a[:27500] + _hold('B Left A', 80) + _hold('', 600) + _hold('A', 1) + _hold('', 120)  #1571 collect-jump
    yield 'morph-ball', a


def _missile_route(morph):
    a = morph + _hold('', 5) + _hold('B Right A', 65) + _hold('B Right', 40) + _hold('', 100) #1729 pedestal-right-0
    a = a[:28410] + _hold('', 8) + _hold('B Right A', 80) + _hold('Right', 300) + _hold('', 80) #1778 edge-jump
    a = a[:28700] + _hold('Down', 20) + _hold('Down X', 100) + _hold('', 5) + _hold('Down', 5) + _hold('Right', 500) + _hold('', 100) #1851 crouch-shot
    a = a[:29350] + _hold('Up', 8) + _hold('', 8) + _hold('Up', 8) + _hold('', 15) + _hold('B Right X', 400) + _hold('', 80) #1888 exit-walk
    a = a[:29500] + _hold('', 5) + _hold('Right A', 55) + _hold('Right X', 400) + _hold('', 80) #1939 hop-right
    a = a[:29900] + _hold('', 5) + _hold('Right A X', 80) + _hold('Right X', 400) + _hold('', 80) #1984 door-hop
    a = a[:30350] + _hold('', 5) + _hold('Right A X', 90) + _hold('Right X', 280) + _hold('', 80) #2023 hall-hop
    a = a[:30700] + _hold('', 5) + _hold('A', 10) + _hold('Down A X', 120) + _hold('', 100) + _hold('Right X', 200) + _hold('', 80) #2072 hall-down
    a = a[:31150] + _hold('Left X', 180) + _hold('', 100) #2116 hall-left
    a = a[:31400] + _hold('', 5) + _hold('Down', 5) + _hold('', 5) + _hold('Down', 5) + _hold('Left', 180) + _hold('', 80) #2270 lower-roll
    a = a[:31600] + _hold('Up', 8) + _hold('', 8) + _hold('Up', 8) + _hold('', 10) + _hold('Left X', 100) + _hold('', 80) #2309 door-shoot
    a = a[:31780] + _hold('Left X', 90) + _hold('', 600) + _hold('A', 1) + _hold('', 120) #2365 a107-left
    yield 'missile-pickup', a



def extend_missile(a):
    # Authored append calls, session01a14c20-ee41-7072-98ad-153710b09812.
    a += _holds('Right+X:200 None:60')  #665 exit-right
    a += _holds('Down:8 None:8 Down:8 None:8 Right:30 None:8 Up:8 None:8 Up:8 None:15 Up+A:30 Left+A+X:25 X:40 None:15')  #700 roll-shaft
    a += _holds('Left+A:60 None:20')  #738 column-left
    a += _holds('Left+A:28 A:42 None:20')  #770 shelf-left
    a += _holds('Right:11 Right+A:65 None:20')  #900 center-edge-hold
    a += _holds('Up+X:60 None:8 Right+A:70 None:20')  #982 right-open-up
    a += _holds('Up+X:180 None:8 A:27 Left+A+X+L:35 None:20')  #1211 upper-left-downshot
    a += _holds('Left+A+X:65 Left+X:180 None:60')  #1286 upper-exit-jump
    a += _holds('B+Left:150 None:60')  #1320 morph-return-west
    a += _holds('Left+A:35 None:30')  #1352 elevator-hop-left
    a += _holds('Left:8 None:20 Up:8 None:800')  #1391 elevator-up-final
    a += _holds('None:100 Left+X:140 X+R:80 Left+X:40 None:20')  #1438 west-entry
    a += _holds('Right:8 A+X+R:65 X+R:60')  #1498 wall-aim-jump
    a += _holds('None:8 Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:60 None:20')  #1542 walkers-west
    a += _holds('Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:8 Left+A+X:65 Left+X:120 None:30')  #1574 last-pirate
    a += _holds('Left:23 None:20')  #1607 normalize-left
    a += _holds('Left:6 None:20')  #1646 normalize-fine
    a += _holds('Right:1 None:15 None:5 A:25 Right+A:12 Left+A:6 A+X:55 None:15')  #1746 face-right-first
    a += _holds('Left:1 None:15')  #1802 first-ledge-nudge
    # Reusable authored ascent suffixes, session01a14c25-820e-74a1-9f92-8579dd24ab56.
    first = _holds('A:23 Right+A:29 Left+A:4 X:55 None:20') #337, skips original118-frame center-right
    first += _holds('Left+A:55 Right+A:4 X:45 None:20') #374
    first += _holds('Left+A:55 Right+A:4 X:55 None:20') #406
    first += _holds('Right+A:55 Left+A:4 X:55 None:20') #433
    first += _holds('Left+A:32 Right+A:4 X:75 None:20') #493
    first += _holds('Right+A:32 Left+A:4 X:75 None:20') #518
    cycle = _holds('Left+A:55 Right+A:4 X:65 None:20') #542
    cycle += _holds('Right+A:60 Left+A:4 X:65 None:20') #578
    cycle += _holds('Left+A:36 Right+A:4 X:75 None:20') #609
    cycle += _holds('Right+A:32 Left+A:4 X:75 None:20') #642
    tail = _holds('Left+A:55 Right+A:4 X:65 None:20') #706
    tail += _holds('Right+A:60 Left+A:4 X:65 None:20') #738
    tail += _holds('Left+A:46 Right+A:4 X:70 None:20') #763
    tail += _holds('A:25 Left+A:13 Right+A:4 X:65 None:20') #787
    tail += _holds('A:25 Right+A:30 Left+A:4 X:65 None:20') #818
    tail += _holds('Up+X:60 None:5 Up+A+X:80 None:200') #852
    # Source01a14c20-ee41 original splice at1841, truncate37541 at1894.
    a = (a + first)[:37541] + cycle
    a += _holds('Right:14 None:20 Left:1 None:20') #1983
    a += _holds('Left:1 None:20') #2013
    a += cycle #2192
    a += cycle*3 + tail #2242
    a = a[:40411] + cycle + tail #2310
    a += _holds('Right:1 None:15 A:25 Left+A:13 Right+A:4 X:65 None:20') #2419
    a += _holds('A:25 Right+A:30 Left+A:4 X:65 None:20') #2450
    a += _holds('Up+X:60 None:5 Up+A+X:80 None:200') #2479
    yield 'parlor-return', a


def extend_parlor(a):
    # Authored reusable Parlor ascent, session01a14c25; suffix only.
    s = []
    s += _holds('Right+A:70 Left+A:4 X:65 None:20') #936 first-right
    s += _holds('None:5 Right+A:40 Left+A:4 X:65 None:20') #970 first-right-edge
    s += _holds('Left+A:8 Right+A:10 None:5 A:40 Left+A:38 Right+A:4 X:65 None:20') #1031 second-right
    s += _holds('Right+A:55 Left+A:4 X:65 None:20') #1061 third-right
    s += _holds('B+Left:5 B+Left+A:45 Right+A:4 X:65 None:20') #1145 fourth-left-run
    s += _holds('B+Left:5 B+Left+A:38 Right+A:4 X:65 None:20') #1168 fifth-left
    s += _holds('B+Right:5 B+Right+A:45 Left+A:4 X:65 None:20') #1199 sixth-right
    s += _holds('B+Left:5 B+Left+A:36 Right+A:4 X:65 None:20') #1238 seventh-left
    s += _holds('B+Right:5 B+Right+A:46 Left+A:4 X:65 None:20') #1261 eighth-right
    s += _holds('B+Left:5 B+Left+A:40 Right+A:4 X:65 None:20') #1288 ninth-left
    s += _holds('B+Right:5 B+Right+A:48 Left+A:4 X:65 None:20') #1313 tenth-right
    s += _holds('B+Left:5 B+Left+A:38 Right+A:4 X:65 None:20') #1345 eleventh-left
    s += _holds('B+Right:5 B+Right+A:41 Left+A:4 X:65 None:20') #1368 twelfth-right
    s += _holds('B+Left:5 B+Left+A:24 Right+A:4 X:65 None:20') #1402 thirteenth-left
    s += _holds('A:25 Right+A:16 Left+A:4 X:65 None:20') #1463 fourteenth-right
    s += _holds('A:8 Left+A:45 Right+A:4 X:65 None:20') #1561 fifteenth-left8
    s += _holds('Right:5 None:15 A:25 Right+A:45 Left+A:4 X:65 None:20') #1626 upper-align
    s += _holds('Right:200 None:20') #1656 upper-walk
    # Archived session01a14c95-3ad8-7260-a733-fdd4312fb27c.
    a += s #86
    a = a[:42940] + _hold('Left X', 8) + _hold('X', 30) + s[293:] #221
    a = a[:44650] + _hold('Left', 1) + _hold('', 20) + [x | BUTTON_BITS['X'] for x in s[1307:]] #335
    a = a[:45210] + _hold('Right X', 14) + _hold('X', 120) + s[1850:] #434
    yield 'bomb-corridor-entry', list(a)
    a += _holds('Down:1 None:8 Down:1 None:12 Right+X:80 Down+X:80 None:120') #488 bomb-morph-entry
    a += _holds('Left:35 None:80 Right:90 None:120') #542 bomb-left-drop
    a += _holds('Up:1 None:30 Right+X:80 None:160') #594 bomb-left-unmorph
    a += _holds('Left+X:100 None:180') #712 bomb-left-run
    a += _holds('Right+X:180 None:120') #747 bomb-door-right
    a += _holds('Right+X:220 None:120') #790 bomb-red-approach
    a += _holds('Right:5 B+Right+X:200 None:120') #824 bomb-red-run
    a += _holds('None:5 Right+A+X:50 None:120') #860 bomb-cap-jump
    a += _holds('Select:1 None:30 X:1 None:30 X:1 None:30 X:1 None:30 X:1 None:30 X:1 None:60 Right:100 None:120') #900 bomb-five-missiles
    a += _holds('Select:1 None:10 Right+X:70 None:700 A:1 None:120') #930 bomb-orb-contact
    a += _holds('Left+X:100 None:5 Left+A+X:70 None:120') #982 bomb-exit-left
    a += _holds('None:5 A+X:20 Left+A+X:60 None:160') #1034 bomb-exit-up
    yield 'bomb-pickup', a


def extend_bombs(a):
    # Archived session01a14c95, original combat and traversal commands.
    a += _holds('Right:1 X:1200 None:120') #1201 fight-beam-inspect
    cycle = _hold('B Right A X', 65) + _hold('Left X', 1) + _hold('X', 230) + _hold('B Left A X', 65) + _hold('Right X', 1) + _hold('X', 230)
    a = a[:50400] + cycle*8 + _hold('', 120) #1241
    a = a[:50700] + (_hold('X', 1) + _hold('', 9))*200 + _hold('', 120) #1291 fight-right-tap
    yield 'bomb-torizo', list(a)
    a += _holds('Left+X:55 None:5 A+X:20 Left+A+X:70 None:140') #1345 torizo-exit
    a += _holds('None:5 Left+A+X:90 None:120') #1379 torizo-exit-hop
    a += _holds('Left+X:250 None:5 Left+A+X:70 None:120') #1425 torizo-return-parlor
    a += _holds('None:5 Left+A+X:90 None:140') #1452 torizo-return-door
    a += _holds('None:5 Left+A+X:35 None:120') #1552 parlor-return-hop
    a += _holds('None:5 Left+A+X:60 None:120') #1609 parlor-return-shelf2
    a += _holds('None:5 Right+A+X:55 None:120') #1665 parlor-return-shelf3
    a += _holds('B+Left+X:6 B+Left+A+X:30 None:120') #1829 parlor-return-run
    a += _holds('B+Right+X:6 B+Right+A+X:32 None:120') #1948 parlor-return-right416
    a += _holds('B+Right+X:6 B+Right+A+X:25 None:120') #2091 parlor-return-right368
    a += _holds('B+Left+X:6 B+Left+A+X:32 None:120') #2141 parlor-return-left352
    a += _holds('B+Right+X:6 B+Right+A+X:25 None:120') #2339 parlor-return-upper-right
    a += _holds('None:5 A+X:28 Down+A:1 A:2 Down+A:1 A:10 Right+A:15 A+X:1 X:140 None:120 Up:1 None:120') #2699
    a = a[:-121] #2876 ball185
    a += _holds('X:1 None:90 Right+X:40 None:60 Up:1 None:20 None:5 Right+A+X:50 None:120') #2876
    a += _holds('X:1 None:90 Right:20 Right+X:1 Right:89 Right+X:1 Right:89 Right+X:1 Right:89 None:120') #3092
    a = a[:56330] #3148
    a += _holds('Up:1 None:20 B+Left+X:6 B+Left+A+X:100 None:120') #3148 hall-west
    a += _holds('None:5 A+X:24 Left+A+X:60 None:120') #3234 hill-top
    yield 'parlor-hill-top', a


def extend_hill(a):
    # Archived session01a14c95 original expressions and reusable suffixes.
    a += _hold('B Right X', 95) + _hold('', 100) + _hold('B Right X', 65) + _hold('', 120) #24726
    a += _hold('B Right X', 225) + _hold('', 90) + _hold('Down', 1) + _hold('', 600) #24790
    a += _hold('Up', 1) + _hold('', 12) + _hold('Right X', 35) + _hold('', 12) + _hold('A X', 10) + _hold('B Left A X', 85) + _hold('', 100) + _hold('Down', 1) + _hold('', 700) #24970
    a = a[:58337] + _hold('Left X', 13) + _hold('', 20) + _hold('Down', 1) + _hold('', 700) #25078
    a += _hold('A', 1) + _hold('', 600) + _hold('B Left X', 80) + _hold('', 120) #25178
    a += _hold('', 300) + _hold('B Left X', 110) + _hold('', 120) #25237
    yield 'gunship-refilled', list(a)
    a += _hold('B Left X', 190) + _hold('', 120) #26382
    a += _hold('Left A X', 22) + _hold('B Left X', 70) + _hold('', 120) #26455
    a += _hold('B Left X', 10) + _hold('B Left A X', 60) + _hold('', 120) #26526
    a += _hold('B Left X', 6) + _hold('B Left A X', 55) + _hold('', 120) #26592
    a += _hold('B Left X', 6) + _hold('B Left A X', 65) + _hold('', 120) #26689
    #26757 reuses west-door-contact minus parlor-return-before-hatch:
    west = _holds('B+Left+A+X:38 Right+X:10 X:80 None:120') #3387
    west += _holds('B+Left+X:6 B+Left+A+X:55 None:120') #3422
    west += _holds('Down:1 None:8 Down:1 None:12 Left:10 X:1 None:90 Left+X:30 None:90 X:1 None:90 None:120') #3450
    west += _holds('Left+X:45 None:20 Up:1 None:20 Left+X:100 None:120') #3483
    a += _hold('B Left X', 139) + west #26757
    a += _hold('B Left X', 410) + _hold('', 120) #26829
    a += _hold('', 300) + _hold('A', 1) + _hold('', 120) + _hold('Right X', 25) + _hold('', 60) #26993
    a += _hold('B Left X', 20) + _hold('B Left A X', 50) + _hold('B Left X', 80) + _hold('', 120) #27054
    #27134 reuses lead-99bd-bottom-left minus qa-waver-tank-xfire-exit:
    a += _hold('B Left X', 180) + _hold('', 60) #15868
    a += _hold('B Right X', 50) + _hold('B Left X', 140) + _hold('B Right X', 100) + _hold('B Left X', 140) + _hold('', 60) #16066
    a += _hold('B Right X', 90) + _hold('B Left X', 150) + _hold('', 120) #16101
    a += (_hold('B Left X', 35) + _hold('B Left A X', 35))*6 + _hold('B Left X', 100) + _hold('', 120) #27214
    a += _hold('Left', 24) + _hold('', 20) + _hold('Down', 1) + _hold('', 1100) + _hold('Left', 35) + _hold('', 90) + _hold('Right', 38) + _hold('', 120) #27297
    yield 'green-brinstar-arrival', a


def extend_green(a):
    # Archived session01a14c95: alternate-lower-door29185, land39765, east-five40562.
    a += _holds('Right+X:20 None:60 Left+X:32 None:80 Right+X:32 None:80 Left+X:30 None:120')
    a = a[:67300] + _holds('Right+X:30 None:100') #39765
    a = a[:67300] + _holds('Right:75 None:30 Select:1 None:20 A:3 A+X:1 None:90 A:3 A+X:1 None:90 A:3 A+X:1 None:90 A:3 A+X:1 None:90 A:3 A+X:1 None:90 Right:80 None:120') #40562
    yield 'early-supers-door', list(a)
    # Original authored tuples, archived session01a15330-201d-7fc0-a594-a9c60b0a2a44.
    a += _holds('Right:10 None:20 Left:7 B+Left+A:12 B+Right:3 B+Right+A:19 A:30 None:40') #585
    a += _holds('Left:14 B+Left+A:12 B+Right:3 B+Right+A:19 A:30 None:80') #600
    a += _holds('Y:1 B+Right+X:25 B+Right+A:1 B+Right:1 B+Down:1 B+A:3 B+Down+Right:1 B+Right:150 None:10') #797
    a = a[:68540] #817 shutters-crossed
    a += _holds('Up+X:20 Up+A+X:35 None:40') #824
    a += _holds('Left:9 Up+X:1 Up+A+X:35 None:20') #831
    a += _hold('', 40) #839 upper-shaft-wait
    a += _holds('Left:15 Right:3 Right+A:24 Left:3 Left+A:20 Right:3 Right+A:20 None:10') #860
    a += _holds('Left:3 Left+A:20 Right:3 Right+A:25 Left:3 Left+A:25 None:10') #869
    a += _holds('Left:5 Right:3 Right+A:25 Left:3 Left+A:35 Left:25 None:10') #879
    a += _holds('B+Left+X:135 None:10') #887
    a += _holds('B+Left+A+X:15 B+Left+X:60 None:10') #902
    a += _holds('None:180 A:1 None:60') #916
    a += _hold('', 130) + _hold('A', 1) + _hold('', 65) #938
    yield 'super-missiles', a


def extend_super(a):
    # Original archived session 01a15330-201d-7fc0-a594-a9c60b0a2a44.
    # 993 tank-left-drop.inputs
    a += _hold('Left A', 8) + _hold('A', 6) + _hold('', 60)
    # 1005 tank-return-shaft.inputs
    a += _hold('', 110)
    # 1014 green-shaft-return.inputs
    a += _hold('Left X', 70) + _hold('', 120)
    # 1029 green-shaft-exit.inputs
    a += _hold('Left A X', 30) + _hold('', 150)
    # 1131 green-shaft-descent.inputs
    a += _hold('Left X', 20) + _hold('X', 100)
    # 1141 green-shaft-mid.inputs
    a += _hold('Left X', 12) + _hold('X', 50)
    # 1155 green-ripper-shot.inputs
    a += _hold('Select', 1) + _hold('', 6) + _hold('X', 1) + _hold('', 18)
    # 1187 ripper-platform-land.inputs
    a = a[:70444] + _hold('Left', 18) + _hold('', 8)
    # 1195 ripper-health-grab.inputs
    a += _hold('Y', 1) + _hold('A', 25) + _hold('', 25)
    # 1204 green-save-entry.inputs
    for word, n in [(_buttons(''), 17), (_buttons('Select'), 1), (_buttons(''), 1), (_buttons('Left A'), 5), (_buttons('Left A X'), 1), (_buttons('Left A'), 65), (_buttons(''), 120)]:
        a += [word] * n
    # 1213 green-save-room.inputs
    a += _hold('Left', 40) + _hold('', 120)
    # 1221 green-save-stand.inputs
    a += _hold('Left', 35) + _hold('', 100)
    # 1228 green-save-prompt.inputs
    a += _hold('Left', 24) + _hold('', 100)
    # 1235 green-save-platform.inputs
    a += _hold('Left A', 14) + _hold('', 80)
    # 1242 green-save-confirm.inputs
    a += _hold('Left', 10) + _hold('', 50)
    # 1266 green-save-aligned.inputs
    a += _hold('Right', 12) + _hold('', 40)
    # 1273 green-save-used.inputs
    a += _hold('A', 1) + _hold('', 180)
    # 1281 green-save-71700.inputs
    a += _hold('A', 1) + _hold('', 162)
    # 1300 green-save-71800.inputs
    a += _hold('A', 1) + _hold('', 60) + _hold('Right', 1) + _hold('', 38)
    yield 'green-brinstar-save', list(a)
    # 1350 bigpink-save-exit.inputs
    for word, n in [(_buttons('Y'), 1), (_buttons('Right A'), 40), (_buttons('Right X'), 60), (_buttons(''), 140)]:
        a += [word] * n
    # 1357 bigpink-shaft-return.inputs
    a += _hold('Right A', 25) + _hold('Right X', 75) + _hold('', 130)
    # 1364 bigpink-shaft-bottom.inputs
    a += _hold('Right', 9) + _hold('', 80)
    # 1371 bigpink-enter.inputs
    a += _hold('Select', 1) + _hold('', 1) + _hold('X', 1) + _hold('B Right', 70) + _hold('', 130)
    # 1399 green-hill-geemers.inputs
    a += _hold('Y', 1) + _hold('B Right X', 25) + _hold('B Right A X', 60) + _hold('', 20)
    # 1414 green-hill-cross.inputs
    a += _hold('B Right X', 140) + _hold('', 10)
    # 1421 dachora-pit-drop.inputs
    a += _hold('B Right X', 62) + _hold('', 280)
    # 1438 dachora-bomb-passage.inputs
    for word, n in [(_buttons('Down'), 1), (_buttons(''), 8), (_buttons('Down'), 1), (_buttons(''), 8), (_buttons('X'), 1), (_buttons(''), 75), (_buttons('Right'), 80), (_buttons(''), 270)]:
        a += [word] * n
    # 1452 dachora-drop-open.inputs
    for _ in range(4):
        a += _hold('X', 1) + _hold('', 70) + _hold('Right', 10) + _hold('', 1)
    a += _hold('Right', 40) + _hold('', 320)
    # 1489 green-hill-east.inputs
    a += _hold('Up', 1) + _hold('', 10) + _hold('B Right X', 70) + _hold('', 10)
    # 1506 green-hill-skree.inputs
    a += _hold('Right X R', 24) + _hold('X R', 50)
    # 1522 green-hill-skree-clear.inputs
    a += _hold('Select Down', 1) + _hold('', 5) + _hold('X', 1) + _hold('', 20)
    # 1536 bigpink-door-entry.inputs
    a += _hold('Y Right', 25) + _hold('B Right X', 90) + _hold('', 170)
    # 1562 bigpink-drop-east.inputs
    a += _hold('B Right X', 95) + _hold('', 20)
    # 1569 bigpink-middle-drop.inputs
    a += _hold('B Left X', 60) + _hold('', 25)
    # 1583 bigpink-lower-east.inputs
    a += _hold('B Right X', 70) + _hold('', 20)
    # 1609 bigpink-geega-distance.inputs
    a += _hold('Left X', 20) + _hold('X', 120)
    # 1616 bigpink-geegas-clear.inputs
    a += _hold('Left X', 32) + _hold('X', 120)
    # 1625 bigpink-second-geega.inputs
    a += _hold('Left X', 13) + _hold('X', 100)
    # 1632 bigpink-bomb-floor.inputs
    for word, n in [(_buttons('Left X'), 40), (_buttons(''), 10), (_buttons('Down'), 1), (_buttons(''), 8), (_buttons('Down'), 1), (_buttons(''), 8), (_buttons('X'), 1), (_buttons(''), 190)]:
        a += [word] * n
    # 1655 bigpink-lower-cross.inputs
    a = a[:75550] + _hold('Up', 1) + _hold('B Right X R', 110) + _hold('', 10)
    # 1670 bigpink-east-ledge-hop.inputs
    a += _hold('Right A X', 25) + _hold('Right X', 45) + _hold('', 40)
    # 1677 bigpink-deep-left.inputs
    a += _hold('B Left X', 100) + _hold('', 30)
    # 1685 bigpink-statue-hop.inputs
    a += _hold('Left A X', 25) + _hold('Left X', 70) + _hold('', 50)
    # 1692 bigpink-missile-ledge.inputs
    a += _hold('Left X', 40) + _hold('', 80)
    # 1699 bigpink-missiles-picked.inputs
    a += _hold('Left X', 22) + _hold('', 400) + _hold('A', 1) + _hold('', 50)
    # 1717 charge-passage-open.inputs
    for word, n in [(_buttons('B Right X'), 50), (_buttons(''), 10), (_buttons('Down'), 1), (_buttons(''), 8), (_buttons('Down'), 1), (_buttons(''), 8), (_buttons('X'), 1), (_buttons(''), 75), (_buttons('Right'), 20), (_buttons(''), 60)]:
        a += [word] * n
    # 1724 charge-chute-enter.inputs
    a += _hold('X', 1) + _hold('', 90) + _hold('Right', 20) + _hold('', 70)
    # 1731 charge-chute-drop.inputs
    a += _hold('Right', 20) + _hold('', 90)
    # 1768 charge-chamber-drop.inputs
    a += _hold('Left', 14) + _hold('', 100)
    # 1775 charge-chamber-floor.inputs
    a += _hold('Left', 8) + _hold('', 85)
    # 1781 charge-orb-open.inputs
    a += _hold('Left', 15) + _hold('', 45) + _hold('Up', 1) + _hold('', 12) + _hold('X', 25)
    # 1794 charge-orb-hop.inputs
    a += _hold('Up', 1) + _hold('', 12) + _hold('Left A X', 20) + _hold('X', 50)
    # 1803 charge-platform-hop.inputs
    a += _hold('Left A', 35) + _hold('X', 70)
    # 1817 charge-picked.inputs
    a += _hold('X R', 35) + _hold('Left A', 18) + _hold('', 400)
    # 1824 charge-beam-78380.inputs
    a += _hold('A', 1) + _hold('', 89) + _hold('X', 90) + _hold('', 80)
    yield 'charge-beam', list(a)
    # 1894 charge-return-hop1.inputs
    a += _hold('B Right A', 60) + _hold('', 20)
    # 1903 charge-return-hop2.inputs
    a += _hold('Right A', 35) + _hold('', 25)
    # 1912 charge-return-hop3.inputs
    a += _hold('', 20) + _hold('Right A', 40) + _hold('', 25)
    # 1921 charge-return-hop4.inputs
    a += _hold('', 15) + _hold('Right A', 40) + _hold('', 25)
    # 1930 charge-return-exit.inputs
    a += _hold('', 10) + _hold('Right A', 60) + _hold('', 20)
    # 1973 charge-return-jump-morph.inputs
    a += _hold('A', 22) + _hold('Down A', 1) + _hold('Left A', 60) + _hold('', 20)
    # 1982 charge-return-passage.inputs
    a += _hold('Down', 1) + _hold('', 1) + _hold('Down', 1) + _hold('Left', 25) + _hold('', 10) + _hold('Up', 1) + _hold('', 10) + _hold('Up', 1) + _hold('', 10)
    # 1991 charge-return-clear.inputs
    a += _hold('Down', 1) + _hold('', 12) + _hold('Down', 1) + _hold('', 12) + _hold('Left', 35) + _hold('', 10) + _hold('Up', 1) + _hold('', 12) + _hold('Up', 1) + _hold('', 12)
    # 2000 bigpink-statue-return.inputs
    a += _hold('B Right', 20) + _hold('B Right A', 60) + _hold('', 20)
    # 2009 bigpink-statue-bomb-hop.inputs
    a += _hold('Down', 1) + _hold('', 12) + _hold('Down', 1) + _hold('', 12) + _hold('X', 1) + _hold('', 35) + _hold('Right', 70) + _hold('', 30)
    # 2018 bigpink-east-door.inputs
    a += _hold('Right', 60) + _hold('', 45) + _hold('Up', 1) + _hold('', 12) + _hold('Up', 1) + _hold('', 12)
    # 2027 greenhill-enter.inputs
    a += _hold('Select', 1) + _hold('', 12) + _hold('Select', 1) + _hold('', 12) + _hold('X', 1) + _hold('B Right', 75) + _hold('', 100)
    # 2036 greenhill-first-pocket.inputs
    a += _hold('', 40) + _hold('Y', 1) + _hold('B Right X', 40) + _hold('X', 70)
    # 2047 greenhill-geega-first.inputs
    a += _hold('B Right', 32) + _hold('', 8) + (_hold('X', 1) + _hold('', 8)) * 12
    # 2066 greenhill-health-cycle1.inputs
    a += _hold('Right', 18) + _hold('', 10) + _hold('Left', 18) + _hold('', 10) + _hold('Right', 1) + (_hold('X', 1) + _hold('', 8)) * 25
    # 2082 greenhill-health-cycle2.inputs
    a += _hold('Right', 11) + _hold('', 10) + _hold('Left', 24) + _hold('', 10) + _hold('Right', 1) + (_hold('X', 1) + _hold('', 8)) * 25
    # 2091 greenhill-health-cycle5.inputs
    cycle = _hold('Right', 18) + _hold('', 10) + _hold('Left', 24) + _hold('', 10) + _hold('Right', 1) + (_hold('X', 1) + _hold('', 8)) * 25
    a += cycle * 3
    # 2108 greenhill-health-cycle11.inputs
    cycle = _hold('Right', 17) + _hold('', 10) + _hold('Left', 24) + _hold('', 10) + _hold('Right', 1) + (_hold('X', 1) + _hold('', 8)) * 25
    a += cycle * 6
    # 2138 greenhill-first-descent.inputs
    a += _hold('B Right X', 90) + (_hold('X', 1) + _hold('', 8)) * 8
    # 2147 greenhill-second-descent.inputs
    a += _hold('B Right X', 100) + (_hold('X', 1) + _hold('', 8)) * 12
    # 2164 greenhill-sidehopper1.inputs
    a += _hold('B Right X', 40) + _hold('X', 50) + _hold('', 1) + (_hold('X', 1) + _hold('', 8)) * 18
    # 2174 greenhill-sidehopper2-approach.inputs
    a += _hold('B Right X', 85) + _hold('', 15)
    # 2197 greenhill-lower-descent.inputs
    a += (_hold('B Right X R', 1) + _hold('B Right R', 8)) * 8 + _hold('X', 30)
    # 2206 greenhill-gate-pillar.inputs
    a += _hold('Right A X', 65) + _hold('', 25)
    # 2215 greenhill-gate-open.inputs
    a += (_hold('Right X R', 1) + _hold('Right R', 8)) * 2 + _hold('X R', 35) + _hold('', 30)
    # 2224 greenhill-gate-fight.inputs
    a += (_hold('X', 1) + _hold('', 8)) * 18
    # 2236 greenhill-ceiling-fight.inputs
    a += _hold('B Right', 30) + (_hold('X R', 1) + _hold('R', 8)) * 20
    # 2246 greenhill-east-run.inputs
    a += _hold('B Right X', 90) + _hold('', 20)
    # 2255 noobbridge-enter.inputs
    a += (_hold('Right A X', 1) + _hold('Right A', 8)) * 7 + _hold('B Right', 60) + _hold('', 150)
    # 2325 noobbridge-upper-entry.inputs
    a += _hold('Select', 1) + _hold('', 12) + _hold('Select', 1) + _hold('', 12) + _hold('X', 1) + _hold('Y', 1) + _hold('B Right', 40) + _hold('B Right A', 60) + _hold('', 10)
    # 2336 noobbridge-run-cross.inputs
    a += _hold('B Right X', 180) + _hold('', 10)
    # 2345 noobbridge-east-boyon.inputs
    a += _hold('B Right X', 65) + _hold('X', 25) + _hold('', 1) + (_hold('X', 1) + _hold('', 8)) * 12
    # 2355 redtower-enter.inputs
    a += _hold('B Right', 45) + _hold('Select', 1) + _hold('', 12) + _hold('Select', 1) + _hold('', 12) + _hold('Right A', 10) + _hold('Right A X', 1) + _hold('Right A', 39) + _hold('B Right', 60) + _hold('', 150)
    # 2371 red-tower-85800.inputs
    a += _hold('Y', 1) + _hold('', 110)
    assert len(a) == 85800
    yield 'red-tower', list(a)
    # 2427 redtower-first-drop.inputs
    a += _hold('B Right X', 48) + _hold('', 55)
    # 2439 redtower-left-drop.inputs
    a += _hold('B Left X', 48) + _hold('', 55)
    # 2453 redtower-neck-drop.inputs
    a += _hold('B Right X', 30) + _hold('X', 60)
    # 2500 redtower-bomb-shaft.inputs
    a += _hold('Down', 1) + _hold('', 12) + _hold('Down', 1) + _hold('', 12) + _hold('Right', 8) + _hold('X', 1) + _hold('', 110)
    # 2509 redtower-half-shaft.inputs
    a += _hold('Left', 16) + _hold('', 55)
    # 2518 redtower-shaft-open.inputs
    a += _hold('X', 1) + _hold('', 170)
    yield 'redtower-shaft-open', a


def extend_redtower(a):
    """Extend the supplied 86483 integer samples in place; return the same list.

    Original archived session 01a153bb-3fca-7c60-acb7-ef6509fc3f25.
    Final route length is 148406. Milestones are prefixes of this result:
    Spazer90706, HiJump99191, KraidRefilled115663, KraidPhase2119823,
    KraidDefeated120820, Varia122732, WarehouseReturn128547,
    WarehouseEnergy129802, BubbleMountain138749 and SpeedBooster148406.
    """
    assert len(a) == 86483
    a.extend(_holds('Left:26 None:80'))  # tower-drop-left; ordinal 59
    a.extend(_holds('Right:22 None:150'))  # tower-drop-bottom; ordinal 66
    a.extend(_holds('Up:1 None:25 Left:4 Select:1 None:1 Select:1 None:1 X:1 None:20 Left:100 None:60'))  # tower-energy-door; ordinal 102
    a.extend(_holds('None:45 Y:1 Left:46 None:30'))  # energy-approach; ordinal 114
    a.extend(_holds('Left:26 None:350'))  # energy-refill; ordinal 122
    a.extend(_holds('A:1 None:90'))  # energy-safe; ordinal 130
    # energy-station-87565; ordinal 144: alias, no new samples.
    a.extend(_holds('Right+X:4 Right+A+X:60 Right+X:90 None:100'))  # energy-exit; ordinal 156
    a.extend(_holds('Right+X:80 None:130'))  # energy-exit-door; ordinal 163
    a.extend(_holds('B+Right+X:65 Right+X:40 None:150'))  # tower-east-door; ordinal 170
    a.extend(_holds('B+Right+X:12 B+Right+A+X:42 None:20'))  # batroom-center-jump; ordinal 202
    a.extend(_holds('None:20 B+Right+X:16 B+Right+A+X:55 Right+X:20 None:20'))  # batroom-long-east-jump; ordinal 216
    a.extend(_holds('Right+A+X:60 Right+X:25 None:15'))  # batroom-east-step; ordinal 223
    a.extend(_holds('Right+X:65 None:140'))  # spazer-approach-entry; ordinal 230
    a.extend(_holds('Right+A:45 Right:25 None:140'))  # spazer-approach-room; ordinal 238
    a.extend(_holds('Up+X:1 Up+A+X:55 None:30'))  # spazer-shaft-up; ordinal 246
    a.extend(_holds('Right:2 Right+A:1 Left+A:4 Right+A:4 Left+A:4 Right+A:4 Left+A:4 Right+A:4 Right:2 Left+A:20'))  # spazer-spin-neck; ordinal 314
    a.extend(_holds('Right:2 Right+A:20 Left:2 Left+A:20 Right:2 Right+A:20 Left:2 Left+A:20'))  # spazer-spin-upper; ordinal 321
    a.extend(_holds('Left+A:5 Right:2 Right+A:30 None:25'))  # spazer-top-ledge; ordinal 336
    a.extend(_holds('Down:1 None:5 Down:1 None:10 Right:10 X:1 None:90 Right:22 X:1 None:90'))  # spazer-bomb-passage; ordinal 344
    a.extend(_holds('Right:18 Right+X:1 Right:100 None:20'))  # spazer-inner-block; ordinal 351
    a.extend(_holds('Up:1 Right+X:40 Right+A+X:65 None:30'))  # spazer-green-approach; ordinal 360
    a.extend(_holds('Right+A+X:40 None:40'))  # spazer-green-ledge; ordinal 367
    a.extend(_holds('Select:1 None:1 Select:1 None:1 X:1 None:20 Right:65 None:130'))  # spazer-room-entry; ordinal 374
    a.extend(_holds('Y:1 Right:60 None:10'))  # spazer-orb-approach; ordinal 382
    a.extend(_holds('Right+A:8 Right+X:1 Right:32 None:40'))  # spazer-orb-break; ordinal 389
    a.extend(_holds('None:240 A:1 None:100 Left:6 None:20'))  # spazer-safe; ordinal 397
    a.extend(_holds('None:60 Left:12 None:20'))  # spazer-final-safe; ordinal 405
    # spazer-90706; ordinal 417: alias, no new samples.
    a.extend(_holds('None:30'))  # kraid-compat-neutral; ordinal 460
    a.extend(_holds('B+Left+X:6 B+Left+A+X:55 Left+X:15 None:40'))  # kraid-spazer-exit-ledge; ordinal 479
    a.extend(_holds('Left+X:65 None:140'))  # kraid-below-spazer-return; ordinal 484
    a.extend(_holds('B+Left+X:75 None:20'))  # kraid-upper-passage-wall; ordinal 491
    a.extend(_holds('Left+A:25 None:45'))  # kraid-upper-passage-ledge; ordinal 498
    a.extend(_holds('Down:1 None:5 Down:1 None:10 Left:40 X:1 None:90 Left:12 None:20'))  # kraid-center-shaft-bomb; ordinal 505
    a.extend(_holds('Right:4 X:1 None:180'))  # kraid-center-shaft-drop; ordinal 514
    del a[91550:]  # kraid-descent-pillar; ordinal 555: authored truncation
    a.extend(_holds('Right:40 None:30 Up:1 None:20'))  # kraid-descent-pillar; ordinal 555
    a.extend(_holds('Up:1 None:10 Right:2 Right+A+X:45 Right+X:20 None:20'))  # kraid-center-platform; ordinal 562
    a.extend(_holds('B+Right+X:14 B+Right+A+X:65 Right+X:20 None:20'))  # kraid-east-gap; ordinal 569
    a.extend(_holds('Right+A+X:40 None:40 Right+X:65 None:140'))  # kraid-lower-east-exit; ordinal 576
    a.extend(_holds('B+Right+X:90 None:140'))  # kraid-glass-tunnel; ordinal 596
    a.extend(_holds('B+Right+X:90 None:140'))  # kraid-east-tunnel; ordinal 603
    a.extend(_holds('B+Right+X:90 None:140'))  # kraid-warehouse-entry; ordinal 617
    a.extend(_holds('B+Right+X:90 None:20'))  # kraid-warehouse-wall; ordinal 625
    a.extend(_holds('Select:1 None:1 Select:1 None:1 X:1 None:25 Right:45 None:15'))  # kraid-super-wall; ordinal 653
    a.extend(_holds('X:1 None:10 Y:1 Down:1 None:5 Down:1 None:10 X:1 Right:100 None:20'))  # kraid-wall-morph-entry; ordinal 669
    a.extend(_holds('Right:70 None:20'))  # kraid-hidden-passage; ordinal 678
    a.extend(_holds('Right:50 None:30 Up:1 None:15 Up:1 None:15'))  # kraid-pit-floor; ordinal 686
    a.extend(_holds('Right:6 Right+A:25 Left:2 Left+A:5 Right+A:35 None:30'))  # kraid-pit-walljump; ordinal 693
    a.extend(_holds('Right:2 Right+A:25 Left:2 Left+A:6 Right+A:35 None:30'))  # kraid-pit-second-walljump; ordinal 700
    a.extend(_holds('Left:14 Right:8 Right+A:24 Left:2 Left+A:6 Right+A:35 None:30'))  # kraid-pit-spin-walljump; ordinal 707
    a.extend(_holds('B+Left:42 None:10'))  # kraid-pit-left-wall; ordinal 723
    a.extend(_holds('Left:20 Left+A:25 Right:2 Right+A:6 Left+A:30 None:20'))  # kraid-pit-left-climb; ordinal 730
    a.extend(_holds('None:35 A:30 Down:1 Left:12 None:40'))  # kraid-left-ledge-hop; ordinal 737
    a.extend(_holds('A:35 Left+A:25 None:30'))  # kraid-left-ledge-high-hop; ordinal 744
    a.extend(_holds('Down:1 None:5 Down:1 None:10 Left:12 X:1 Left:140 None:20'))  # kraid-hi-jump-return-hole; ordinal 831
    a.extend(_holds('Left:10 Up:1 None:20 Up:1 None:15 Down:1 None:450'))  # kraid-norfair-elevator; ordinal 838
    a.extend(_holds('None:230'))  # kraid-business-center; ordinal 847
    a.extend(_holds('None:50 Left+X:22 None:80'))  # kraid-business-drop1; ordinal 883
    a.extend(_holds('Right+X:35 None:130'))  # kraid-business-drop2; ordinal 890
    a.extend(_holds('Left+X:22 None:160'))  # kraid-business-drop3; ordinal 919
    a.extend(_holds('Right+X:40 None:80'))  # kraid-business-drop4; ordinal 926
    a.extend(_holds('Left+X:48 None:120'))  # kraid-business-map-ledge; ordinal 933
    a.extend(_holds('Right+X:35 None:80'))  # kraid-business-drop5; ordinal 942
    a.extend(_holds('Left+X:35 None:130'))  # kraid-hi-jump-door; ordinal 947
    a.extend(_holds('Select:1 None:10 X:1 None:20 X:1 None:20 X:1 None:20 X:1 None:20 X:1 None:20 Left:80 None:140'))  # kraid-hi-jump-antechamber; ordinal 962
    a.extend(_holds('Left+X:45 None:250 A:1 None:120'))  # kraid-hi-jump-energy; ordinal 969
    a.extend(_holds('None:180 A:1 None:120'))  # kraid-hi-jump-energy-safe; ordinal 977
    a.extend(_holds('Down:1 None:5 Down:1 None:10 X:1 None:150'))  # kraid-hi-jump-bomb-floor; ordinal 986
    a.extend(_holds('Right:8 X:1 None:160'))  # kraid-hi-jump-open-floor; ordinal 1000
    a.extend(_holds('X:1 None:180'))  # kraid-hi-jump-floor-drop; ordinal 1007
    a.extend(_holds('Left:90 None:140'))  # kraid-hi-jump-shaft-bottom; ordinal 1014
    a.extend(_holds('Left:45 None:10 Up:1 None:15 Up:1 None:15'))  # kraid-hi-jump-left-shaft; ordinal 1021
    a.extend(_holds('Left+A:12 Left+A+X:1 Left+X:55 None:140'))  # kraid-hi-jump-room; ordinal 1028
    a.extend(_holds('Left+A+X:40 Left+X:50 None:140'))  # kraid-hi-jump-door-hop; ordinal 1035
    a.extend(_holds('Left:20 None:15 Left+A:20 Left+A+X:1 Left+A:35 Left:20 None:30'))  # kraid-hi-jump-divider; ordinal 1052
    a.extend(_holds('X:1 None:15 Left:20 None:450 A:1 None:150'))  # kraid-hi-jump-pickup; ordinal 1059
    a.extend(_holds('Right:12 None:20'))  # hi-jump-99191; ordinal 1076
    a.extend(_holds('Right:2 Right+A:65 None:55 Right+X:85 None:150'))  # kraid-hi-jump-exit; ordinal 1104
    a.extend(_holds('Right:2 Right+A:40 None:70'))  # kraid-hi-jump-return-step1; ordinal 1113
    a.extend(_holds('Left:6 Left+A:60 None:50'))  # kraid-hi-jump-return-step2; ordinal 1120
    a.extend(_holds('Right:4 Right+A:60 None:60'))  # kraid-hi-jump-missile-ledge; ordinal 1129
    a.extend(_holds('Down:1 None:5 Down:1 None:10 Right:30 None:450 A:1 None:150'))  # kraid-hi-jump-missiles; ordinal 1136
    a.extend(_holds('X:1 Right:150 None:20'))  # kraid-hi-jump-upper-bomb; ordinal 1145
    assert len(a) == 100719  # Shared start of the authored business-return tail.
    a.extend(_holds('Up:1 None:15 Up:1 None:10 Left:2 X:250 None:30'))  # kraid-sova-charge; ordinal 1204
    # Author reused only this tail after frame 100719; keep no full prefix copy.
    business_return_tail = []
    business_return_tail.extend(_holds('X:1 Right:150 None:20'))  # kraid-hi-jump-upper-step; ordinal 1152
    business_return_tail.extend(_holds('X:1 Right:85 None:20'))  # kraid-hi-jump-upper-exit; ordinal 1159
    business_return_tail.extend(_holds('Right:32 None:30 Up:1 None:15 Up:1 None:15'))  # kraid-hi-jump-upper-safe; ordinal 1166
    business_return_tail.extend(_holds('Up:1 None:10 Right:2 Right+A+X:55 Right+X:35 None:25 Right+X:75 None:150'))  # kraid-business-return; ordinal 1173
    a.extend(_holds('Down:1 None:5 Down:1 None:10 Right:4 None:10') + business_return_tail)  # kraid-sova-return; ordinal 1215
    del business_return_tail
    a.extend(_holds('Right+X:6 Right+A+X:35 None:70'))  # kraid-business-climb1; ordinal 1222
    a.extend(_holds('Left:6 Left+A:25 Up+A:1 A:20 None:55'))  # kraid-business-climb2; ordinal 1229
    a.extend(_holds('Left:10 None:10 A:25 Right+A:20 Up+A:1 A:20 None:50'))  # kraid-business-climb3; ordinal 1238
    a.extend(_holds('Left:6 Left+A:25 Up+A:1 A:25 None:55'))  # kraid-business-climb4; ordinal 1245
    a.extend(_holds('A:20 Right+A:30 Up+A:1 A:10 None:60'))  # kraid-business-climb5; ordinal 1252
    a.extend(_holds('A:20 Right+A+X:35 Up+A:1 A:10 None:60'))  # kraid-business-climb6; ordinal 1266
    a.extend(_holds('A:18 Right+A+X:25 Up+A:1 A:20 None:60'))  # kraid-business-climb7; ordinal 1273
    a.extend(_holds('Right:2 Right+A+X:32 Up+A:1 A:25 None:60'))  # kraid-business-climb8; ordinal 1280
    a.extend(_holds('Left+X:6 Left+A+X:30 Up+A:1 A:25 None:60'))  # kraid-business-climb9; ordinal 1287
    del a[102840:]  # kraid-business-climb10; ordinal 1294: authored truncation
    a.extend(_holds('Right:6 Right+A+X:40 Up+A:1 A:20 None:55'))  # kraid-business-climb10; ordinal 1294
    a.extend(_holds('Left:6 Left+A:40 Up+A:1 A:20 None:50'))  # kraid-elevator-top; ordinal 1329
    a.extend(_holds('Left:4 Up:10 None:950'))  # kraid-elevator-up; ordinal 1335
    a.extend(_holds('Up:1 None:950'))  # kraid-elevator-up2; ordinal 1380
    a.extend(_holds('Right:42 Select:1 None:2 Select:1 None:2 X:1 None:45'))  # kraid-warehouse-open; ordinal 1388
    a.extend(_holds('Down:1 None:5 Down:1 None:15 X:1 Right:90 None:25'))  # kraid-warehouse-morph; ordinal 1412
    a.extend(_holds('X:1 Right:80 None:20'))  # kraid-warehouse-hole; ordinal 1420
    a.extend(_holds('Right:65 None:20'))  # kraid-warehouse-hole2; ordinal 1429
    a.extend(_holds('X:1 Right:100 None:20 Right:70 None:20 Right:48 None:30 Up:1 None:15 Up:1 None:15'))  # kraid-warehouse-enter; ordinal 1453
    a.extend(_holds('A:52 Right+A:40 Right:60 None:20'))  # kraid-pit-hijump; ordinal 1461
    a.extend(_holds('Right+X:90 None:130'))  # kraid-warehouse-east-door; ordinal 1472
    a.extend(_holds('Right+A:40 Right:70 None:130'))  # kraid-warehouse-zeela; ordinal 1483
    a.extend(_holds('A:8 Down+A+X:20 None:60 Right:23 None:70'))  # kraid-zeela-downshot; ordinal 1505
    a.extend(_holds('Right:10 Down:1 None:5 Down:1 Right:25 None:95 X:1 Right:95 None:25'))  # kraid-zeela-drop; ordinal 1513
    a.extend(_holds('Left:20 None:70 Right:20 None:20 X:1 Right:170 None:30'))  # kraid-zeela-bottom; ordinal 1532
    a.extend(_holds('Right:30 None:20 Up:1 None:15 Up:1 None:15 Up+A+X:100 None:150'))  # kraid-zeela-right-shaft; ordinal 1542
    a.extend(_holds('Up+A+X:110 None:160'))  # kraid-kihunter-entry; ordinal 1551
    a.extend(_holds('B+Right+X:115 None:20'))  # kraid-kihunter-hall; ordinal 1560
    a.extend(_holds('Down:1 None:5 Down:1 Right:20 None:25 X:1 Right:170 None:30'))  # kraid-kihunter-tunnel; ordinal 1575
    a.extend(_holds('Left:18 None:45 Right:100 None:30'))  # kraid-kihunter-tunnel2; ordinal 1584
    a.extend(_holds('Up:1 None:15 Up:1 None:15 Right:10 Right+A+X:100 Right+X:60 None:130'))  # kraid-fakekraid-entry; ordinal 1592
    a.extend(_holds('Left+X:90 None:130'))  # kraid-save-return; ordinal 1608
    a.extend(_holds('X:1 Left:80 None:140'))  # kraid-save-return2; ordinal 1617
    a.extend(_holds('Left:60 None:25 Down:1 None:5 Down:1 Left:90 None:20 X:1 Left:120 None:30'))  # kraid-kihunter-west-tunnel; ordinal 1628
    a.extend(_holds('Up:1 None:15 Up:1 None:15 Left+A:25 Left:72 None:30'))  # kraid-kihunter-shaft-align; ordinal 1638
    a.extend(_holds('Left+X:43 None:20'))  # kraid-kihunter-shaft-top; ordinal 1646
    a.extend(_holds('Right:8 A:8 Down+A+X:30 None:80'))  # kraid-kihunter-shaft-down2; ordinal 1662
    a.extend(_holds('Left:24 None:65 Right:5 Right+X:70 None:30'))  # kraid-kihunter-lower-door; ordinal 1670
    a.extend(_holds('None:140 X:120'))  # kraid-fakekraid-ready; ordinal 1678
    a.extend(_holds('B+Right+X:120 Right:1 B+Right+X:65 Right:1 B+Right+X:65 Right:1 B+Right+X:45'))  # kraid-fakekraid-hall; ordinal 1688
    a.extend(_holds('X:25 None:30 X:80 None:30 X:80 None:30 X:80 None:30'))  # kraid-fakekraid-charge; ordinal 1696
    a.extend(_holds('B+Right+X:66 Right+A+X:60 Right+X:50 None:140'))  # kraid-prekraid-entry; ordinal 1713
    a.extend(_holds('X:1 Right:70 None:150'))  # kraid-prekraid-room; ordinal 1724
    a.extend(_holds('Left:2 B+Left+X:80 B+Left:1 B+Left+X:80 B+Left:1 B+Left+X:60 None:25'))  # kraid-pirates-return; ordinal 1741
    a.extend(_holds('Left+X:50 X:30 None:30 X:80 None:35'))  # kraid-pirates-clear; ordinal 1749
    a.extend(_holds('Left+X:38 X:45 None:35'))  # kraid-pirates-last; ordinal 1758
    a.extend(_holds('B+Right:238 Right+A:65 Right:30 X:1 Right:75 None:140'))  # kraid-prekraid-arrival; ordinal 1767
    a.extend(_holds('Up+A+X:90 None:25 Up+A+X:70 None:70'))  # kraid-refill-shaft; ordinal 1781
    a.extend(_holds('A:24 Right+A:62 None:35'))  # kraid-refill-ledge; ordinal 1789
    a.extend(_holds('A:22 Right+A+X:75 None:30 X:1 Right:80 None:140'))  # kraid-refill-room; ordinal 1797
    a.extend(_holds('Select:1 None:2 Select:1 None:2 X:1 Y:1 Right:70 None:150'))  # kraid-refill-open; ordinal 1813
    a.extend(_holds('Right:36 None:400'))  # kraid-refill-use; ordinal 1822
    a.extend(_holds('Right:240 None:450 A:1 None:80'))  # kraid-refill-energy; ordinal 1870
    a.extend(_holds('Right+A:96 None:40 Left:240 None:450 A:1 None:70'))  # kraid-refill-missiles; ordinal 1878
    a.extend(_holds('Left+A:105 Left+X:1 Left:30 None:30 X:1 Left:80 None:150'))  # kraid-refill-return; ordinal 1914
    a.extend(_holds('Left+A+X:50 Left:80 None:150'))  # kraid-preboss-return; ordinal 1933
    a.extend(_holds('Left:95 None:40'))  # kraid-preboss-drop; ordinal 1941
    a.extend(_holds('A:8 Down+A+X:25 None:85 Right:6 Right+A+X:100 None:30'))  # kraid-preboss-bottom; ordinal 1949
    a.extend(_holds('Right+A+X:70 None:10 Right+A+X:70 None:20'))  # kraid-eye-approach; ordinal 1959
    a.extend(_holds('Right+A:70 None:30'))  # kraid-eye-platform; ordinal 1967
    a.extend(_holds('Right+A:10 Right:50 None:20'))  # kraid-eye-short-hop; ordinal 1994
    a.extend(_holds('Right+A:10 Right:48 None:15 Right+A:10 Right:38 None:20'))  # kraid-eye-right-ledge; ordinal 2002
    a.extend(_holds('Left:14 None:10 Right:2 Select:1 None:3 X:1 None:35 X:1 None:35 X:1 None:45'))  # kraid-eye-missiles; ordinal 2011
    a.extend(_holds('None:120'))  # kraid-eye-timing; ordinal 2076
    a.extend(_holds('Left:12 None:10 Right:1 None:15'))  # kraid-eye-standback; ordinal 2111
    a.extend(_holds('X:1 None:110 X:1 None:110 X:1 None:180'))  # kraid-eye-open; ordinal 2119
    a.extend(_holds('Y:1 None:2 X:1 Right:100 None:240'))  # kraid-arena-entry; ordinal 2128
    a.extend(_holds('Right+X:78 X:250'))  # kraid-arena-trigger; ordinal 2140
    a.extend(_holds('X:300 None:35'))  # kraid-arena-rise; ordinal 2150
    a.extend(_holds('X:80 A+X:13 None:65'))  # kraid-eye-charge-hit; ordinal 2256
    a.extend(_holds('Select:1 None:2 Select:1 None:28 A:7 A+X:1 None:40'))  # kraid-mouth-super; ordinal 2284
    a.extend(_holds('Y:1 X:380'))  # kraid-second-phase; ordinal 2293
    a.extend(_holds('A+X:24 Right+A+X:30 Up+A+X:1 X:45'))  # kraid-upper-platform; ordinal 2305
    a.extend(_holds('A+X:13 None:65'))  # kraid-upper-eye; ordinal 2314
    a.extend(_holds('None:30 Select:1 None:2 Select:1 None:2 A:13 A+X:1 None:40'))  # kraid-upper-super-afterclaw; ordinal 2351
    # kraid-phase2-119823; ordinal 2371: alias, no new samples.
    a.extend(_holds('Y:1 X:40 A+X:20 None:45'))  # kraid-reopen-b; ordinal 2572
    a.extend(_holds('None:35 Select:1 None:2 Select:1 None:2 A:13 A+X:1 None:35'))  # kraid-super-c; ordinal 2584
    a.extend(_holds('None:3 Select:1 None:2 A:13 A+X:1 None:60'))  # kraid-finish-d; ordinal 2598
    a.extend(_holds('Y:1 None:720'))  # kraid-death-wait; ordinal 2610
    # kraid-defeated-120820; ordinal 2630: alias, no new samples.
    a.extend(_holds('B+Right+X:70 Right+A+X:12 Right+X:45 None:90'))  # kraid-varia-approach; ordinal 2681
    a.extend(_holds('None:40 Right+X:28 Right+A+X:18 Right:12 None:120'))  # kraid-varia-pedestal; ordinal 2697
    a.extend(_holds('Left:18 None:35 Right:1 Right+X:12 None:30'))  # kraid-varia-open; ordinal 2711
    a.extend(_holds('Right+A:12 None:700 A:1 None:180'))  # kraid-varia-collect; ordinal 2725
    a.extend(_holds('None:420 Right:8 None:60'))  # kraid-varia-settle; ordinal 2737
    # varia-suit-122732; ordinal 2756: alias, no new samples.
    a.extend(_holds('B+Left+X:45 None:140'))  # ice-varia-exit; ordinal 2814
    a.extend(_holds('Left+A+X:12 Left+X:45 None:140 B+Left+X:110 Left+A+X:12 Left+X:45 None:140'))  # ice-kraid-return; ordinal 2828
    a.extend(_holds('Left+X:45 None:150'))  # ice-prekraid-return; ordinal 2842
    a.extend(_holds('B+Left+X:15 B+Left+A+X:45 B+Left+X:60 None:1 Left+X:35 None:140'))  # ice-prekraid-cross; ordinal 2854
    a.extend(_holds('Left+A+X:10 Left+X:37 None:15'))  # ice-prekraid-low-hop; ordinal 2886
    a.extend(_holds('Left+X+L:16 None:1 Left+A+X:10 Left+X:50 None:15'))  # ice-prekraid-pillar; ordinal 2898
    a.extend(_holds('Left+X:25 None:1 Left+A+X:10 Left+X:25 None:20'))  # ice-prekraid-midstep; ordinal 2928
    a.extend(_holds('Left+A+X:10 Left+X:25 None:15'))  # ice-prekraid-leftstep; ordinal 2933
    a.extend(_holds('Left+A+X:10 Left+X:45 None:1 Left+X:70 None:140'))  # ice-pirates-return; ordinal 2945
    a.extend(_holds('Left+X:65 None:140'))  # ice-pirates-entry; ordinal 2957
    a.extend(_holds('B+Left+X:70 B+Left:1 B+Left+X:70 B+Left:1 B+Left+X:70 B+Left:1 B+Left+X:70 B+Left:1 None:20'))  # ice-pirates-cross; ordinal 2969
    a.extend(_holds('B+Left+X:52 Left+A+X:12 Left+X:45 None:140'))  # ice-kihunter-return; ordinal 2981
    a.extend(_holds('Left+A+X:12 Left+X:50 None:140'))  # ice-kihunter-entry; ordinal 2993
    a.extend(_holds('Left+X:35 None:140'))  # ice-kihunter-door; ordinal 3005
    a.extend(_holds('Right+X:40 X:30 None:30'))  # ice-pirate-finish; ordinal 3031
    a.extend(_holds('X:70 None:40 X:70 None:40'))  # ice-pirate-charge2; ordinal 3052
    a.extend(_holds('Right+X:45 X:30 None:40'))  # ice-pirate-approach; ordinal 3060
    a.extend(_holds('Left+X:30 Left+A+X:12 Left+X:40 Left:1 Left+X:30 None:140'))  # ice-kihunter-clear-exit; ordinal 3072
    a.extend(_holds('B+Right+X:245 None:40'))  # ice-minikraid-return; ordinal 3084
    a.extend(_holds('B+Right+X:85 X:25 None:30'))  # ice-minikraid-approach; ordinal 3098
    a.extend(_holds('X:1 None:35 Right+X:55 None:30'))  # ice-minikraid-clear; ordinal 3117
    a.extend(_holds('B+Left+X:276 Left+A+X:12 Left+X:35 Left:1 Left+X:30 None:140'))  # ice-hall-cleared-return; ordinal 3129
    a.extend(_holds('Left+X:50 None:15'))  # ice-kihunter-shaft-base; ordinal 3141
    a.extend(_holds('Up+A+X:18 Right+A+X:20 Up+A+X:30 None:20'))  # ice-kihunter-shaft-rise; ordinal 3155
    a.extend(_holds('Up+A+X:35 None:50'))  # ice-kihunter-shaft-cap; ordinal 3169
    a.extend(_holds('B+Left+X:55 None:20'))  # ice-kihunter-west; ordinal 3185
    a.extend(_holds('Left:14 Down:1 None:1 Down:1 None:5 X:1 None:120'))  # ice-warehouse-return; ordinal 3197
    a.extend(_holds('Down:1 None:12 X:1 None:120'))  # ice-warehouse-floor; ordinal 3209
    a.extend(_holds('None:170 Up:1 None:15'))  # ice-warehouse-arrive; ordinal 3221
    a.extend(_holds('Down:1 None:12 Left+X:125 None:10'))  # ice-zeela-low-tunnel; ordinal 3233
    a.extend(_holds('X:1 None:65 Left:4 None:10 Up:1 None:12 Up:1 None:12 Up+A+X:40 None:20'))  # ice-zeela-shaft-open; ordinal 3247
    a.extend(_holds('Left:7 None:5 X:1 None:65 Left:12 None:15 Up:1 None:12 Up:1 None:12'))  # ice-zeela-left-bomb; ordinal 3260
    a.extend(_holds('Left+X:35 None:140'))  # ice-warehouse-lower-door; ordinal 3274
    a.extend(_holds('Left+X:24 Left+A+X:18 Left+X:35 None:20'))  # ice-warehouse-energy-approach; ordinal 3290
    a.extend(_holds('Down:1 None:10 Down:1 None:10') + _holds('X:1 None:1') * 40 + _holds('None:15'))  # speed-beetom-shake-bombs; ordinal 3493
    a.extend(_holds('X:1 None:1') * 60 + _holds('None:20'))  # speed-beetom-last; ordinal 3502
    a.extend(_holds('Up:1 None:12 Up:1 None:12 Up+X:1 None:12 Up+A:30 None:30'))  # speed-energy-open; ordinal 3509
    a.extend(_holds('Up:5 Up+X:1 Up:20'))  # speed-energy-reveal; ordinal 3516
    a.extend(_holds('Up+A:30 None:450 A:1 None:120'))  # speed-energy-collected; ordinal 3523
    # warehouse-energy-129802; ordinal 3532: alias, no new samples.
    a.extend(_holds('B+Right+X:45 Right+A+X:12 Right+X:25 Right:1 Right+X:25 None:140'))  # speed-energy-exit; ordinal 3546
    a.extend(_holds('Right+A+X:12 Right+X:25 Right:1 Right+X:25 None:140'))  # speed-zeela-return; ordinal 3553
    a.extend(_holds('Right:16 Up:5 Up+X:1 Up:15 Up+A+X:45 None:20'))  # speed-zeela-up-open; ordinal 3560
    a.extend(_holds('Up+A+X:15 Right+A+X:35 None:20'))  # speed-zeela-shaft-step; ordinal 3567
    a.extend(_holds('Right+A+X:30 Left+A+X:22 None:25'))  # speed-zeela-shaft-rise; ordinal 3574
    a.extend(_holds('Up+A+X:18 Left+A+X:35 None:20'))  # speed-zeela-upper-rise; ordinal 3588
    a.extend(_holds('Left:12 Up:5 Up+X:1 Up:12 Up+A:35 None:30'))  # speed-zeela-top; ordinal 3597
    a.extend(_holds('Left+X:35 None:140'))  # speed-warehouse-top; ordinal 3604
    a.extend(_holds('Left+A+X:35 Left+X:25 None:140'))  # speed-warehouse-door; ordinal 3611
    a.extend(_holds('Right:8 Up:5 Up+X:1 Up:8 Up+A+X:20 Left+A+X:25 Left+X:20 None:140'))  # speed-warehouse-exit-timed; ordinal 3618
    a.extend(_holds('Right:8 Up:5 Up+X:1 Up:8 Up+A:20 Left:8 Left+X:35 None:140'))  # speed-warehouse-short-door; ordinal 3625
    a.extend(_holds('B+Left+X:95 None:40'))  # speed-warehouse-west; ordinal 3632
    a.extend(_holds('A:47 Down:1 Left:40 None:20'))  # speed-warehouse-morph-gap; ordinal 3655
    a.extend(_holds('A:20 Down+A:1 A:10 Down+A:1 Left:90 None:20'))  # speed-warehouse-air-morph; ordinal 3662
    a.extend(_holds('X:1 None:65 Left:35 None:15'))  # speed-warehouse-cap-bomb; ordinal 3669
    a.extend(_holds('Up:1 None:12 Up:1 None:12 Select:1 None:2 Select:1 None:2 X:1 None:25 Left+X:45 None:20'))  # speed-warehouse-super-open; ordinal 3676
    a.extend(_holds('Y:1 Down:1 None:10 Down:1 None:10 X:1 Left:110 None:15'))  # speed-warehouse-bomb-through; ordinal 3698
    a.extend(_holds('Left:35 None:15 Up:1 None:12 Up:1 None:12 Down:1 None:500'))  # speed-norfair-elevator; ordinal 3705
    a.extend(_holds('None:260'))  # speed-norfair-arrival; ordinal 3714
    a.extend(_holds('None:90 Right+X:28 None:70 Left+X:28 None:65'))  # speed-business-upper; ordinal 3721
    a.extend(_holds('Right+X:45 Right:1 Right+X:35 None:140'))  # speed-cathedral-entry; ordinal 3728
    a.extend(_holds('Right:12 Down:1 None:10 Down:1 None:10 X:1 None:130'))  # speed-cathedral-floor; ordinal 3735
    a.extend(_holds('None:40 Up:1 None:12 Up:1 None:12 B+Right+X:125 None:20'))  # speed-cathedral-bottom; ordinal 3742
    a.extend(_holds('Right+X:18 Right+A+X:30 Up+A:1 A:25 None:40'))  # speed-cathedral-ledge-stop; ordinal 3756
    a.extend(_holds('Right:15 Up:1 None:40 A:55 None:40'))  # speed-cathedral-pillar; ordinal 3778
    a.extend(_holds('Up+A+X:32 Right+A+X:20 Up+A:1 None:45'))  # speed-cathedral-pillar-up; ordinal 3785
    a.extend(_holds('Up+A:30 Right+A:35 Right+X:35 Right:1 Right+X:35 None:140'))  # speed-cathedral-east-door; ordinal 3792
    a.extend(_holds('Select:1 None:2 Select:1 None:2 X:1 None:20 Y:1 Right+X:35 None:140'))  # speed-cathedral-unlock; ordinal 3799
    a.extend(_holds('B+Right+A+X:45 B+Right+X:25 None:10'))  # speed-cathedral-running; ordinal 3808
    a.extend(_holds('Right+A+X:75 Up+A:1 None:15'))  # speed-cathedral-spire-gap; ordinal 3822
    a.extend(_holds('Right+A+X:75 Right+X:20 Up:1 None:20'))  # speed-cathedral-middle; ordinal 3829
    a.extend(_holds('None:25'))  # speed-cathedral-middle-floor; ordinal 3838
    a.extend(_holds('Left+A+X:32 Up+A:1 None:65'))  # speed-cathedral-middle-step; ordinal 3852
    a.extend(_holds('Left+A+X:32 Up+A:1 None:65'))  # speed-cathedral-middle-step2; ordinal 3859
    a.extend(_holds('Left:10 None:3 Up+A:24 Right+A:35 Up+A:1 None:45'))  # speed-cathedral-upper-shelf; ordinal 3866
    a.extend(_holds('Right+X:20 Right+A+X:20 Right+X:65 Up:1 None:15'))  # speed-cathedral-second-spire; ordinal 3873
    a.extend(_holds('Right+A+X:65 Up+A:1 None:35'))  # speed-cathedral-third-step; ordinal 3887
    a.extend(_holds('None:15 Right+A+X:45 Right+X:105 Right:1 Right+X:35 None:140'))  # speed-cathedral-leave; ordinal 3894
    a.extend(_holds('Up+A+X:20 Right+A+X:45 Right+X:90 Right:1 Right+X:35 None:140'))  # speed-cathedral-last-climb; ordinal 3901
    a.extend(_holds('Right+A+X:18 Right+X:130 Right:1 Right+X:30 None:130'))  # speed-cathedral-third-over; ordinal 3908
    a.extend(_holds('Left:24 None:35 Right+X:55 Right:1 Right+X:90'))  # speed-cathedral-drop; ordinal 3934
    a.extend(_holds('Left+X:36 None:35 Right:35 Select:1 None:1 Select:1 None:5 Right+X:130'))  # speed-cathedral-under; ordinal 3959
    a.extend(_holds('None:18 Y:1 B+Right+A+X:95 B+Right+X:1 B+Right+A+X:95 B+Right+X:1 B+Right+A+X:95 B+Right+X:1 B+Right+A+X:95 B+Right+X:1'))  # speed-tide-upper; ordinal 4008
    a.extend(_holds('B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1'))  # speed-tide-hops; ordinal 4020
    a.extend(_holds('Right+X:10 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1 B+Right+A+X:60 B+Right+X:1'))  # speed-tide-second-half; ordinal 4032
    a.extend(_holds('Right+X:25 Right+A+X:70 Right+X:46 Right+A+X:80 Right:1 Right+X:135 None:20'))  # speed-tide-open-exit; ordinal 4089
    # bubble-mountain-138749; ordinal 4114: alias, no new samples.
    a.extend(_holds('Right:26 Up+A:27 Right+A+X:42 None:35'))  # speed-bubble-first-climb; ordinal 4203
    a.extend(_holds('Up+A:28 Right+A+X:36 None:42'))  # speed-bubble-upper-ledge; ordinal 4217
    a.extend(_holds('None:8 Left+A+X:87 None:20'))  # speed-bubble-left-slope; ordinal 4233
    a.extend(_holds('Left+A:25 None:35'))  # speed-bubble-left-step; ordinal 4365
    a.extend(_holds('Left:10 None:1 B+Right:10 B+Right+A:60'))  # speed-bubble-run-leap; ordinal 4373
    a.extend(_holds('Right+X:88 None:10'))  # speed-bubble-right-pillar; ordinal 4383
    a.extend(_holds('Right:22 None:12'))  # speed-bubble-peak; ordinal 4397
    a.extend(_holds('Right+A:14 Right:10 None:40'))  # speed-bubble-pillar-top; ordinal 4407
    a.extend(_holds('Left:5 Left+A:88 None:40'))  # speed-bubble-left-return; ordinal 4465
    a.extend(_holds('None:15 Up+A:28 Right+A+X:36 None:45 Left+A+X:87 None:20 Left+A:25 None:35'))  # speed-bubble-climb-recover; ordinal 4473
    a.extend(_holds('None:7 Right:5 Right+A:4 Left+A:18 Right:2 Right+A:24'))  # speed-bubble-left-wall-upper; ordinal 4493
    a.extend(_holds('Left:6 Right:2 Right+A:22 Left+A:28 X:1 None:35'))  # speed-bubble-left-wall-double; ordinal 4501
    a.extend(_holds('Right:10 None:5 Down:1 None:10 Down:1 None:10 X:1 None:59 X:1 None:59 X:1 None:59 X:1 None:59 X:1 None:59'))  # speed-bubble-bomb-lift; ordinal 4581
    a.extend(_holds('X:1 None:61 X:1 None:61 X:1 None:61 X:1 None:61 X:1 None:61'))  # speed-bubble-bomb-lift62; ordinal 4598
    a.extend(_holds('Right:13 None:60'))  # speed-bubble-bomb-column; ordinal 4758
    a.extend(_holds('X:1 None:51') * 62)  # speed-bubble-clear-ceiling; ordinal 4766
    a.extend(_holds('X:1 None:51') * 29)  # speed-bubble-top-arrival; ordinal 4774
    a.extend(_holds('Left:60 Up:1 None:10 Up:1 None:35'))  # speed-bubble-top-land; ordinal 4793
    a.extend(_holds('Left:10 None:8 Right:1 None:10 B+Right:18 B+Right+A:52'))  # speed-bubble-super-cross18; ordinal 5072
    a.extend(_holds('B+Right+A:13 B+Right:1 Left:2 Left+A:15 Right+A:28 Right:10 Select:1 Down:4 Down+X:1 Right:12 None:40'))  # speed-bubble-super-central; ordinal 5082
    a.extend(_holds('Y:1 Right:22 None:16'))  # speed-bubble-gap-footing; ordinal 5115
    a.extend(_holds('B+Right:11 B+Right+A:60 Right:20 None:10'))  # speed-bubble-short-gap; ordinal 5123
    a.extend(_holds('Right+A:10 Right:25 None:20 Select:1 None:1 Select:1 None:5 Right+X:140 None:15'))  # speed-farm-room-enter; ordinal 5131
    a.extend(_holds('Right:1 Right+A+X:30 Up:1 Up+X:1 Up:17 Right:1 Up+Right+A:35 Up+X:1 Up:30'))  # speed-farm-shaft-hop; ordinal 5215
    a.extend(_holds('Up+A+X:20 Up+A:1 Up+A+X:20 Up+A:1 Up+A+X:20 Up:20'))  # speed-farm-shaft-fast; ordinal 5233
    a.extend(_holds('Left+A:10 Up+A+X:20 Up+A:1 Up+A+X:20 Up+A:1 Up+A+X:20 Up:20'))  # speed-farm-shaft-rise; ordinal 5243
    a.extend(_holds('Left:11 Up+X:1 Up+A+X:48 Right+A:13 Up:20'))  # speed-farm-shaft-ledge; ordinal 5271
    a.extend(_holds('Right:7 Up+X:1 Up+A+X:42 Right+A:20 Up:35'))  # speed-farm-upper-jump; ordinal 5282
    a.extend(_holds('Left:4 Up:2'))  # speed-farm-upper-align; ordinal 5298
    a.extend(_holds('Left:2 Up:8 Up+A+X:38 Right+A:26 Up:25'))  # speed-farm-upper-launch; ordinal 5307
    a.extend(_holds('Right:10 Right+X:140 None:10'))  # speed-hall-entry; ordinal 5316
    a.extend(_holds('None:35 B+Right:300'))  # speed-hall-run-a; ordinal 5325
    a.extend(_holds('B+Right:330'))  # speed-hall-run-b; ordinal 5334
    a.extend(_holds('B+Right:15 Select:1') + _holds('Right+X:1 Right:14') * 5 + _holds('Right:155 Y:1 None:10'))  # speed-item-room-entry; ordinal 5348
    a.extend(_holds('Right:1 Right+A:8 Right+A+X:50 None:100'))  # speed-orb-jump; ordinal 5359
    a.extend(_holds('A:1 None:90'))  # speed-message-dismiss; ordinal 5368
    a.extend(_holds('A:1 None:120'))  # speed-message-close; ordinal 5384
    a.extend(_holds('None:80 A:1 None:120'))  # speed-booster-148406; ordinal 5407
    assert len(a) == 148406
    return a


def extend_ice(a):
    """Authored Speed148406 -> Ice161613, without input/NMI timing changes.

    Archived 2026-10-19 sessions01a153bb-3fca-7c60-acb7-ef6509fc3f25
    and01a15498-a511-7142-a1de-18b74cb2ac99; ordinals identify original commands.
    """
    # First session: selected return route and farm refill.
    assert len(a) == 148406
    a = list(a)
    a.extend(_holds('Left:8 Left+X:190 None:15'))  # ice-return-room-exit; ordinal 5510
    a.extend(_holds('Left+A:20 Left+X:190 None:15'))  # ice-return-hall; ordinal 5519
    a.extend(_holds('B+Left:300'))  # ice-hall-run-a; ordinal 5531
    a.extend(_holds('B+Left+X:210 None:15'))  # ice-farm-return; ordinal 5540
    a.extend(_holds('None:25 Left:10 Left+A+X:50 Up:35'))  # ice-farm-perch; ordinal 5549
    a.extend(_holds('Down:1 None:10 Down:1 None:10') + _holds('X:1 None:19') * 20)  # ice-farm-bombs; ordinal 5558
    a.extend(_holds('Right+X:20 None:8') + _holds('X:1 None:19') * 30)  # ice-farm-center-bombs; ordinal 5586
    a.extend(_holds('X:1 None:19') * 30 + _holds('None:60'))  # ice-farm-full-refill; ordinal 5596
    a.extend(_holds('Up:1 None:10 Up:1 None:10 Right:1 Right+A+X:60 None:20'))  # ice-farm-descend-align; ordinal 5610
    a.extend(_holds('Right:13 Down:1 None:10 Down:1 None:10 X:1 None:100'))  # ice-farm-shaft-bomb; ordinal 5621
    a.extend(_holds('Down:1 None:10 Down:1 None:10 Left:11 None:10 X:1 None:120'))  # ice-farm-morph-drop; ordinal 5630
    a.extend(_holds('Right:5 None:10 X:1 None:140'))  # ice-farm-shaft-enter; ordinal 5639
    a.extend(_holds('Right:4 None:75'))  # ice-farm-middle-drop; ordinal 5649
    a.extend(_holds('Left:7 None:10 X:1 None:140'))  # ice-farm-lower-drop; ordinal 5658
    a.extend(_holds('None:10 Up:1 None:10 Up:1 None:10 Left:1 Left+A+X:35 Up:1 Up+X:1 None:30'))  # ice-farm-bottom-hop; ordinal 5667
    a.extend(_holds('Left:1 Left+A+X:52 Left:1 Left+X:180 None:15'))  # ice-bubble-arrival; ordinal 5692
    a.extend(_holds('B+Left+X:180'))  # ice-bubble-west-descent; ordinal 5714
    a.extend(_holds('Left+A:100 None:10'))  # ice-bubble-central-hop; ordinal 5723
    a.extend(_holds('None:110 Left:80 None:20'))  # ice-bubble-lower-landing; ordinal 5732
    a.extend(_holds('Left+A:100 Left:50 Left+X:160 None:15'))  # ice-green-bubbles-return; ordinal 5741
    a.extend(_holds('Right:15 None:45 Left:50 None:10'))  # ice-bubble-west-foot; ordinal 5750
    a.extend(_holds('Right:25 None:50 Left:75 None:10'))  # ice-bubble-low-door; ordinal 5759
    a.extend(_holds('Left+X:170 None:15'))  # ice-green-tunnel-entry; ordinal 5770
    a.extend(_holds('Left+X:20 B+Left+A:60 B+Left:20'))  # ice-green-tunnel-hop-a; ordinal 5785
    a.extend(_holds('B+Left+A+X:1 B+Left+A:14') * 7 + _holds('B+Left:10'))  # ice-green-tunnel-hop-b; ordinal 5799
    a.extend(_holds('B+Left+A+X:35 Up:1 None:50'))  # ice-green-tunnel-perch; ordinal 5808
    a.extend(_holds('B+Left:3 B+Left+A+X:55 Up:1 None:20'))  # ice-green-tunnel-hop-c; ordinal 5817
    a.extend(_holds('B+Left:3 B+Left+A+X:43 Up:1 None:30'))  # ice-green-tunnel-hop-d; ordinal 5826
    # Second session: selected Ice approach, pickup and dismissed message.
    assert len(a) == 154091
    a.extend(_holds('B+Left+A+X:110 B+Left+X:12'))  # ice-dragon-west-b; ordinal 49
    a.extend(_holds('B+Left+A+X:60 B+Left+X:2 B+Left+A+X:60 B+Left+X:2'))  # ice-dragon-west-c; ordinal 56
    a.extend(_holds('B+Left+A+X:85 B+Left+X:2 B+Left+A+X:85 B+Left+X:2 B+Left+A+X:85 B+Left+X:2'))  # ice-dragon-west-d; ordinal 63
    a.extend(_holds('B+Left+A+X:80 B+Left+X:2 B+Left+A+X:80 B+Left+X:2 B+Left+A+X:80 B+Left+X:2 B+Left+A+X:80 B+Left+X:2'))  # ice-dragon-west-e; ordinal 72
    a.extend(_holds('B+Right+A:38 A:34 None:4'))  # ice-dragon-west-step-b; ordinal 109
    a.extend(_holds('B+Left+A+X:48 A+X:36 None:3'))  # ice-dragon-west-high-a; ordinal 116
    a.extend(_holds('B+Left+A+X:68 B+Left+X:2 B+Left+A+X:75 B+Left+X:2 B+Left+A+X:75 B+Left+X:2'))  # ice-dragon-west-f; ordinal 123
    a.extend(_holds('B+Left+A+X:27 A+X:50 None:3'))  # ice-dragon-west-step-d; ordinal 144
    a.extend(_holds('B+Left+A+X:68 B+Left+X:90'))  # ice-dragon-west-exit-b; ordinal 153
    a.extend(_holds('B+Left+X:80'))  # ice-cathedral-entry; ordinal 160
    a.extend(_holds('B+Left+A+X:42 A+X:28 None:3'))  # ice-cathedral-climb-a; ordinal 167
    a.extend(_holds('B+Right:20 B+Right+A:12 A:16 B+Left+A:32 A:35 None:3'))  # ice-cathedral-climb-b; ordinal 188
    a.extend(_holds('B+Right:25 None:12'))  # ice-cathedral-rightbase; ordinal 195
    a.extend(_holds('B+Left+A+X:62 A+X:28 None:3'))  # ice-cathedral-climb-c; ordinal 202
    a.extend(_holds('Right:19 None:12'))  # ice-cathedral-midbase; ordinal 209
    a.extend(_holds('A:28 Left+A:24 A:35 None:3'))  # ice-cathedral-climb-d; ordinal 216
    a.extend(_holds('B+Left+A+X:67 A+X:25 None:3'))  # ice-cathedral-high-a; ordinal 223
    a.extend(_holds('B+Left:4 B+Left+A+X:85 B+Left+X:8'))  # ice-cathedral-west-a; ordinal 235
    a.extend(_holds('B+Left+X:35'))  # ice-cathedral-midfall; ordinal 244
    a.extend(_holds('Right:19 None:12 B+Right+A:31 A:48 None:3'))  # ice-cathedral-middle-step; ordinal 258
    a.extend(_holds('B+Right+A+X:40 A+X:45 None:3'))  # ice-cathedral-middle-step-b; ordinal 265
    a.extend(_holds('B+Left+A+X:61 A+X:25 None:3'))  # ice-cathedral-middle-high; ordinal 272
    a.extend(_holds('X:10 B+Left+A+X:38 A+X:50 None:3'))  # ice-cathedral-center-top; ordinal 279
    a.extend(_holds('B+Left:8 B+Left+A+X:72 X:12'))  # ice-cathedral-west-b; ordinal 286
    a.extend(_holds('X:5 B+Left+A+X:39 A+X:52 None:3'))  # ice-cathedral-left-climb; ordinal 293
    a.extend(_holds('B+Left+A+X:50 A+X:43 None:3'))  # ice-cathedral-left-top; ordinal 307
    a.extend(_holds('B+Left:10 B+Left+A+X:65 X:20'))  # ice-cathedral-west-c; ordinal 314
    a.extend(_holds('Up+X:3 Up:1 B+Left+A+X:70 B+Left+X:100'))  # ice-cathedral-west-exit-d; ordinal 344
    a.extend(_holds('B+Left+X:80 B+Left:1 B+Left+X:63 B+Left+X+L:3 B+Left+L:1 B+Select+Left:1 B+Left:1 B+Select+Left:1 B+Left+X:1 B+Left:15 B+Left+X:1 B+Left:15 B+Left+X:1 B+Left:15 B+Y+Left:1 B+Left+X:80'))  # ice-entrance-safer-cross-c; ordinal 615
    assert len(a) == 157527
    # Ordinal 622: safer-cross-c + walljump-c[157525:158325].
    # The following original climb holds ARE that 800-frame donor tail.
    # Original donor crossing351 ended157525; replacement615 ends157527.
    # Preserve the authored two-frame difference; no timing compensation.
    a.extend(_holds('B+Left:4 B+Left+A:55 A:40 None:3'))  # ice-cathedral-entrance-climb-a; ordinal 367
    a.extend(_holds('Up+X:65 Up:1 B+Left+A+X:60 A+X:40 None:3'))  # ice-cathedral-entrance-climb-b; ordinal 374
    a.extend(_holds('B+Right:13'))  # ice-cathedral-entrance-leftledge; ordinal 383
    a.extend(_holds('B+Right+A:20 B+Left+A:18 B+Right:3 B+Right+A:12 Up+A:1 Up+X:10'))  # ice-cathedral-entrance-walljump-b; ordinal 470
    a.extend(_holds('Up:1 Up+X:1 Up:10 Up+X:1 Up:10 Up+X:1 Up:10 Up+X:1 Up:10 Up+X:1 Up:20'))  # ice-cathedral-entrance-openfloor; ordinal 520
    a.extend(_holds('A+X:35 B+Left+A+X:43 X:30'))  # ice-cathedral-entrance-climb-c; ordinal 527
    a.extend(_holds('X:10 B+Left+A+X:65 X:25 B+Right:13'))  # ice-cathedral-entrance-climb-d; ordinal 536
    a.extend(_holds('B+Right+A:40 B+Left:3 B+Left+A:32 B+Left+X:90'))  # ice-cathedral-entrance-walljump-c; ordinal 543
    assert len(a) == 158327
    a.extend(_holds('None:140'))  # ice-business-healthier-entry; ordinal 634
    a.extend(_holds('R:3 X+R:1 R:10 X+R:1 R:10 X+R:1 R:10 X+R:1 R:25'))  # ice-business-dragon-kill; ordinal 648
    a.extend(_holds('B+Left+A+X:33 A+X:40 None:3'))  # ice-business-dragon-drop; ordinal 662
    a.extend(_holds('B+Left+X:45 B+Select+Left:1 B+Left:1 B+Select+Left:1 B+Left+X:1 B+Left:135'))  # ice-business-ice-door; ordinal 671
    a.extend(_holds('B+Y+Left:1 B+Left+X:220'))  # ice-access-run-a; ordinal 678
    a.extend(_holds('B+Y+Left:1 B+Left+A+X:65 B+Left+X:105'))  # ice-access-west-exit; ordinal 685
    a.extend(_holds('B+Left+A+X:35 B+Left+X:2'))  # ice-tripper-hop-b; ordinal 706
    a.extend(_holds('B+Left+X:20'))  # ice-tripper-ledge-a; ordinal 713
    a.extend(_holds('B+Left+X:12 B+Left+A+X:60 B+Left+X:2'))  # ice-tripper-hop-e; ordinal 741
    a.extend(_holds('B+Left+X:5 B+Left+A+X:60 B+Left+X:130'))  # ice-tripper-west-exit; ordinal 748
    a.extend(_holds('B+Left:1 B+Left+X:1 B+Left:140'))  # ice-tripper-west-door; ordinal 755
    a.extend(_holds('B+Left+X:8 B+Left+A+X:35 X:20'))  # ice-pre-shaft-climb-a; ordinal 771
    a.extend(_holds('B+Left:15 B+Right+A+X:43 X:20'))  # ice-pre-shaft-climb-b; ordinal 778
    a.extend(_holds('B+Right:15 B+Left+A+X:43 X:20 B+Left:15 B+Right+A+X:43 X:20'))  # ice-pre-shaft-climb-c; ordinal 794
    a.extend(_holds('B+Right:15 B+Left+A+X:43 X:20 B+Left:15 B+Right+A+X:43 X:20 B+Right:15 B+Left+A+X:43 X:20 B+Left:15 B+Right+A+X:43 X:20'))  # ice-pre-shaft-climb-d; ordinal 808
    a.extend(_holds('B+Right+X:40 X:12'))  # ice-pre-shaft-top-right; ordinal 817
    a.extend(_holds('Down:1 None:8 Down:1 None:10 Left:10 None:10 X:1 None:65'))  # ice-pre-shaft-bomb-floor; ordinal 824
    a.extend(_holds('Down:1 None:20 Down:1 None:20 Left:8 None:10 X:1 None:65'))  # ice-pre-shaft-morph-a; ordinal 831
    a.extend(_holds('None:64'))  # ice-pre-shaft-drop-a; ordinal 838
    a.extend(_holds('Right:30'))  # ice-pre-shaft-crawl-a; ordinal 847
    a.extend(_holds('Left:20'))  # ice-pre-shaft-drop-b; ordinal 854
    a.extend(_holds('None:10 Right:24'))  # ice-pre-shaft-crawl-b; ordinal 861
    a.extend(_holds('B+Right+X:1 B+Right:12 B+Right+X:1 B+Right:12 B+Right+X:1 B+Right:12 B+Right+X:1 B+Right:12'))  # ice-pre-shaft-crawl-c; ordinal 876
    a.extend(_holds('Up:1 None:20 B+Right+X:50 B+Right:1 B+Right+X:1 B+Right:110'))  # ice-room-entry; ordinal 883
    a.extend(_holds('B+Right+A+X:65 B+Right:1 B+Right+X:1 B+Right:130'))  # ice-room-entry-hop; ordinal 890
    a.extend(_holds('B+Right+X:1 B+Right:18 B+Right+A+X:40 None:90'))  # ice-pickup-a; ordinal 897
    a.extend(_holds('A:1 None:90'))  # ice-pickup-dismiss-a; ordinal 905
    a.extend(_holds('None:160 A:1 None:90'))  # ice-beam-dismissed-161613; ordinal 928
    # Ordinal 949 copies this exact route to ice-beam-161613.
    assert len(a) == 161613
    return a


def extend_ice_return(a):
    """Authored 2026-10-24 Ice return, including upper shutter bypass and health drops.

    Controller expressions preserve the selected checked branches exactly.
    This reaches Business Center; it does not qualify Crocomire or its approach.
    """
    assert len(a) == 161733
    # Ice west exit, then jump/morph through the pre-Ice shaft passage.
    a.extend(_holds('Y:1 None:1 Left:8 Left+X:1 Left:160 None:120'))
    a.extend(_holds('Left:60 Left+A:14 Left+Down:1 Left:3 Left+Down:1 Left:110 None:30'))
    a.extend(_holds('None:1 A:16 Down:1 Left:24 None:1'))
    a.extend(_holds('Down:1 None:12 Down:1 None:12 Left:40 None:1'))
    a.extend(_holds('Left:25 None:50 Right:16 None:18 Left:16 None:35'))
    a.extend(_holds('None:15 Up:1 None:12 Up:1 None:12 Right:8 Right+X:1 Right:80 None:120'))
    assert len(a) == 162763
    # Trippers: fresh jump/shoot edges; selected branch ends on the third ledge.
    a.extend(_holds('B+Right:1'))
    a.extend(_holds('B+Right+A+X:1 B+Right+A:11') * 4)
    a.extend(_holds('B+Right:2 B+Right+A+X:1 B+Right+A:23 None:1 B+Right:34'))
    a.extend(_holds('B+Right+A+X:1 B+Right+A:11') * 5)
    a.extend(_holds('B+Right:8'))
    a.extend(_holds('B+Right+A+X:1 B+Right+A:11') * 5)
    assert len(a) == 163001
    a.extend(_holds('B+Right:25'))
    a.extend(_holds('B+Right+A+X:1 B+Right+A:11') * 7)
    a.extend(_holds('None:1'))
    a.extend(_holds('Right+A+X:1 Right+A:11') * 8)
    # Replace the final76 idle frames of the earlier exit trial at first gameplay.
    a.extend(_holds('Right:30 None:34'))
    assert len(a) == 163271
    a.extend(_holds('B+Right+X:1 B+Right:11') * 20)
    a.extend(_holds('None:120'))
    # Closed growing shutters: ordinary bombs open the upper return tunnel.
    # Early Down edges retain A to preserve jump height during airborne morph.
    a.extend(_holds('Left:68 None:1 A:2 A+Down:1 A:2 A+Down:1 A:8 X:1 None:65'))
    a.extend(_holds('Left:12 None:1 Up:1 None:12 Up:1 None:12 A:2 A+Down:1 A:2 A+Down:1 A:14 Right+A:24 Right:30 None:1'))
    a.extend(_holds('Left:16 None:1'))
    a.extend(_holds('Up:1 None:12 Up:1 None:12 A:2 A+Down:1 A:2 A+Down:1 A:20 Right+A:20 Right:15 None:1'))
    a.extend(_holds('X:1 None:65 Right:18 None:1 Right:70 None:1'))
    assert len(a) == 164155
    # Wait for the tunnel Sova to return, bomb it from range, collect its drop.
    a.extend(_holds('Right:60 None:800 None:38 X:1 Left:20 None:70 Right:28 None:1'))
    a.extend(_holds('Right:165 None:35'))
    a.extend(_holds('None:6 Up:1 None:12 Up:1 None:12 Right:1 Right+X:1 Right:60 None:130'))
    assert len(a) == 165597
    # Business Center: ranged freezing/kills and body contact with health drops.
    a.extend(_holds('Right:10 None:8 R:3'))
    a.extend(_holds('R+X:1 R:11') * 20)
    a.extend(_holds('Right:8 Right+A:8 Right:24 Left:18 None:1 Down:1'))
    a.extend(_holds('Down+X:1 Down:11') * 4)
    a.extend(_holds('None:1 Up:1'))
    a.extend(_holds('Up+X:1 Up:11') * 4)
    a.extend(_holds('None:1 Right:2 None:1 A:12 A+Down:1'))
    a.extend(_holds('Down+X:1 Down:11') * 4)
    a.extend(_holds('None:1'))
    assert len(a) == 166082
    return a


def extend_reserve(a):
    """Norfair Reserve branch from the authored Bubble Mountain upper-west landing.

    Preserve the selected controller sequence, including the intermediate missile
    message. The final reserve message completes before the neutral follow-up.
    """
    assert len(a) == 148406
    a = list(a[:145574])
    a.extend(_holds('Select:1 None:1 Select:1 None:5 Left+X:140 None:15'))  # reserve-ac83-entry
    a.extend(_holds('Y:1 X:1 None:10 X:1 None:10 X:1 None:10 X:1 None:10 X:1 None:10'))  # reserve-geruta-clear
    a.extend(_holds('Left+A:43 Up+A:1 None:15'))  # reserve-first-pillar
    a.extend(_holds('Left:6 Left+A:52 Up+A:1 None:12'))  # reserve-second-pillar-near
    a.extend(_holds('Left:4 None:8'))  # reserve-second-pillar-settle
    a.extend(_holds('None:180 A:1 None:60'))  # reserve-missile-message
    a.extend(_holds('None:110 A:1 None:40'))  # reserve-missile-dismiss
    a.extend(_holds('R:2 X+R:1 None:20'))  # reserve-aperture-shot
    a.extend(_holds('Left:6 Left+A:10 A+X:1 Down+A:1 A:1 Down+A:1 Left+A:35'))  # reserve-aperture-close-morph
    a.extend(_holds('Left:25'))  # reserve-aperture-through
    a.extend(_holds('Left:60 Up:1 None:10 Up:1 Left+X:45 None:120'))  # reserve-ac5a-entry
    a.extend(_holds('Left:45 None:120'))  # reserve-ac5a-cross
    a.extend(_holds('Left:8 Left+A:50 Up+A:1 None:15'))  # reserve-room-first-stone
    a.extend(_holds('Left+A:50 Up+A:1 None:15'))  # reserve-room-second-stone
    a.extend(_holds('Left:6 Left+A:28 Down+A:1 A:1 A+X:1 A:10'))  # reserve-third-early-shot
    a.extend(_holds('Left+A:12 A+L:2 A+X+L:1 Left+A:4 Up+A:1 None:10'))  # reserve-third-land-aimed
    a.extend(_holds('Left:18 None:8'))  # reserve-third-recover
    a.extend(_holds('Left+A:50 Up+A:1 None:15'))  # reserve-room-fourth-stone
    a.extend(_holds('Left+A:50 Up+A:1 None:15 Left+A:50 Up+A:1 None:15'))  # reserve-tank-ledge
    a.extend(_holds('L:2 X+L:1 None:12 Left:18 None:400 A:1 None:180'))  # reserve-collected
    assert len(a) == 147887
    return a


def extend_power_bomb_entry(a):
    """Continue the authored Ice-return route to the first Power Bomb room.

    Room entry is navigation coverage; acquisition remains a separate check.
    """
    assert len(a) == 166082
    a = list(a)
    a.extend(_holds('A:20 Right+A:20 None:45'))  # business-ascent-platform1
    a.extend(_holds('A:12 Left+A:35 None:45'))  # business-ascent-leftledge
    a.extend(_holds('Right+A:35 None:50'))  # business-ascent-middle-direct
    a.extend(_holds('Left+A:28 None:50'))  # business-ascent-upperleft-direct
    a.extend(_holds('Left:10 None:4 A:24'))  # business-elevator-clearance-rise
    a.extend(_holds('Right+A:38 None:50'))  # business-elevator-clearance-land
    a.extend(_holds('Left:18 None:4'))  # business-elevator-center
    a.extend(_holds('None:12 Up:1 None:40'))  # business-elevator-settled-trigger
    a.extend(_holds('None:910'))  # business-elevator-arrived
    a.extend(_holds('Left:12 Left+X:65 None:100'))  # pb-west-cf80
    a.extend(_holds('Left+X:90 None:110'))  # pb-west-glass-tube
    a.extend(_holds('Left+X:90 None:110'))  # pb-west-cf54
    a.extend(_holds('Left+X:90 None:110'))  # pb-west-a408-entry
    a.extend(_holds('B+Right:22 B+Left:20'))  # pb-a408-runup
    a.extend(_holds('B+Left:5 B+Left+A:50'))  # pb-a408-running-hop
    a.extend(_holds('Left+X:4 None:16'))  # pb-a408-center-landing
    a.extend(_holds('B+Left+X:23'))  # pb-a408-center-runup
    a.extend(_holds('B+Left+A:30'))  # pb-a408-west-early-jump
    a.extend(_holds('Left+A:6 Down+A:1 A:1 A+X:1 None:20'))  # pb-a408-maw-ranged-down
    a.extend(_holds('Left:3 Left+A:30 None:12'))  # pb-a408-west-ledge-final
    a.extend(_holds('Left+X:30 None:110'))  # pb-west-a3dd-entry
    a.extend(_holds('None:8 B+Left+X:24 B+Left+A:12 B+Left+A+X:1 B+Left+A:32'))  # pb-a3dd-center-hop
    a.extend(_holds('None:1 Left+A:5 Left+A+X:1 Left+A:35'))  # pb-a3dd-stone-center
    a.extend(_holds('Left+A:24 None:25 B+Left+X:18 B+Left+A:50'))  # pb-a3dd-west-hop
    a.extend(_holds('Left+X:65 None:110'))  # pb-redtower-return
    a.extend(_holds('Left+A+X:25 None:110'))  # pb-redtower-door-hop
    a.extend(_holds('Left+X:100 None:110'))  # pb-energy-station-entry
    a.extend(_holds('None:90 Left:72 None:350 A:1 None:40'))  # pb-energy-refilled
    a.extend(_holds('Right:12 Right+A:50 Right+X:50 None:150'))  # pb-energy-exit
    a.extend(_holds('Up:2 Up+X:1 None:10'))  # pb-tower-ripper1-freeze
    a.extend(_holds('Left:13 None:4 A:20 Right+A:8'))  # pb-tower-ripper1-align-air
    a.extend(_holds('Up+A:1 Right+A:6 Up+A:1 None:45'))  # pb-tower-ripper1-brake-land
    a.extend(_holds('None:190 Up:2 Up+X:1 None:10'))  # pb-tower-ripper2-freeze
    a.extend(_holds('Right:8 Right+A:8 Up+A:1 A:16'))  # pb-tower-ripper2-clear-rise
    a.extend(_holds('Left+A:14 Up+A:1 None:45'))  # pb-tower-ripper2-land
    a.extend(_holds('None:24 Up:2 Up+X:1 None:10'))  # pb-tower-ripper3-freeze
    a.extend(_holds('Right:14 Right+A:6 Up+A:1 A:18'))  # pb-tower-ripper3-rise
    a.extend(_holds('Left+A:14 Up+A:1 None:45'))  # pb-tower-ripper3-turn-land
    a.extend(_holds('Right:1 None:95 R:2 X+R:1 None:25'))  # pb-tower-ripper4-gap-freeze
    a.extend(_holds('Right+A:74 Up+A:1 None:18'))  # pb-tower-ripper4-gap-land
    a.extend(_holds('A:36 Left+A:26 Up+A:1 None:25'))  # pb-tower-stone-gap
    a.extend(_holds('Left:6 Left+A:10 Up+A:1 A:25 Right+A:18 Up+A:1 None:25'))  # pb-tower-second-stone-gap
    a.extend(_holds('None:6 Right:18 None:8 Up:2 Up+X:1 None:20'))  # pb-tower-shot-shaft
    a.extend(_holds('Up:2 Up+X:1 None:25 A:40 Right+A:10 Up+A:1 None:25'))  # pb-tower-shaft-upper-ledge
    a.extend(_holds('Left:2 None:10 B+Left:1 B+Left+A:28 Up+A:1 None:15'))  # pb-tower-left-shaft-run-hop
    a.extend(_holds('A:30 Left+A:24 Up+A:1 None:30'))  # pb-tower-mid-shaft-land
    a.extend(_holds('Left:4 A:26 Right+A:40 Up+A:1 None:25'))  # pb-tower-right-middle-ledge
    a.extend(_holds('Right:2 Right+X:20 None:20'))  # pb-tower-middle-recover
    a.extend(_holds('A:26 Right+A:52 Up+A:1 None:20'))  # pb-tower-upper-right-ledge
    a.extend(_holds('Right:10 None:4 A:27 Left+A:40 Up+A:1 None:31'))  # pb-tower-upper-left-clearance
    a.extend(_holds('Left:12 None:8'))  # pb-tower-high-shelf-align
    a.extend(_holds('None:4 A:40 Right+A:34 Up+A:1 None:20'))  # pb-tower-high-right-land
    a.extend(_holds('A:40 Left+A:34 Up+A:1 None:20'))  # pb-tower-high-left-land
    a.extend(_holds('Left:12 None:12 A:40 Right+A:34 Up+A:1 None:20'))  # pb-tower-high-right-second
    a.extend(_holds('A:40 Left+A:34 Up+A:1 None:20'))  # pb-tower-top-left-shelf
    a.extend(_holds('None:126 Up:2 Up+X:1 None:12'))  # pb-tower-upper-ripper1-freeze
    a.extend(_holds('Right:12 Right+A:8 Up+A:1 A:20 Left+A:24 Up+A:1 None:50'))  # pb-tower-upper-ripper1-land
    a.extend(_holds('Up:2 Up+X:1 None:12'))  # pb-tower-upper-ripper2-freeze
    a.extend(_holds('Right:12 Right+A:8 Up+A:1 A:20 Left+A:24 Up+A:1 None:50'))  # pb-tower-upper-ripper2-land
    a.extend(_holds('None:57 Up:2 Up+X:1 None:12'))  # pb-tower-upper-ripper3-freeze
    a.extend(_holds('Right:12 Right+A:8 Up+A:1 A:20 Left+A:22 Up+A:1 None:52'))  # pb-tower-upper-ripper3-land
    a.extend(_holds('Right:1 None:41 R:2 X+R:1 None:12'))  # pb-tower-upper-ripper4-diagonal-freeze
    a.extend(_holds('A:16 A+R:2 A+X+R:1 A:1 None:85'))  # pb-tower-top-floor-air-shot
    a.extend(_holds('Right:6 Right+A:24 Up+A:1 A:20 Right+A:24 Up+A:1 None:45'))  # pb-tower-top-opening-jump
    a.extend(_holds('Right+X:50 None:110'))  # pb-east-a2f7-entry
    a.extend(_holds('None:45 X:1 None:15 X:1 None:15 X:1 None:15'))  # pb-a2f7-entry-waver-clear
    a.extend(_holds('Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-edge-short-hop
    a.extend(_holds('Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-second-short-hop
    a.extend(_holds('Right+X:8 None:12 Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-third-short-hop
    a.extend(_holds('Right+X:8 None:12 Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-fourth-short-hop
    a.extend(_holds('Right+X:12 None:12 Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-fifth-short-hop
    a.extend(_holds('Right+X:13 None:12 Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-sixth-short-hop
    a.extend(_holds('Right+X:8 None:12 Right:4 Right+A:7 Right:25 Up:1 None:8'))  # pb-a2f7-seventh-short-hop
    a.extend(_holds('Right:4 Right+A:10 Right:24 Up:1 None:10'))  # pb-a2f7-raised-stone-hop
    a.extend(_holds('Right:4 Right+A:8 Right:36 Up:1 None:10 Right+X:75 None:150'))  # pb-east-a322-entry
    a.extend(_holds('Right:15 None:12 A:4 Down+A:2 Down+A+X:1 Down:12 Down+X:1 None:45'))  # pb-a322-upper-shot-floors
    a.extend(_holds('A:4 Down+A:2 Down+A+X:1 None:60'))  # pb-a322-second-shot-floor
    a.extend(_holds('Right:12 None:45'))  # pb-a322-lower-floor-drop
    a.extend(_holds('Left:16 None:12 A:4 Down+A:2 Down+A+X:1 None:65'))  # pb-a322-final-shot-floor
    a.extend(_holds('Right:16 None:100'))  # pb-a322-bottom-door-ledge
    a.extend(_holds('Left:8 Select+Left:1 Left:1 Select+Left:1 Left:1 Left+X:1 Left:50 None:160'))  # pb-west-a3ae-green-door
    a.extend(_holds('Left:20 None:170'))  # pb-a3ae-enter
    assert len(a) == 174992
    return a


def extend_power_bomb_pickup(a):
    """Collect the first Power Bomb after the preserved A3AE entry route."""
    assert len(a) == 174992
    a = list(a)
    a.extend(_holds('Y:1 Left:8 None:12 Left:4 Left+A:44 Up+A:1 None:12'))  # pb-a3ae-first-stone
    a.extend(_holds('X:1 None:8 A:10 Left+A:14 Up+A:1 None:25'))  # pb-a3ae-first-boyon-platform
    a.extend(_holds('Left+A:10 Down+A:2 Down+A+X:1 Left+A:10 Up+A:1 None:28'))  # pb-a3ae-second-stone-braked
    a.extend(_holds('Left:4 Left+A:7 Left:25 Up:1 None:8'))  # pb-a3ae-third-stone
    a.extend(_holds('Left:4 Left+A:12 Left:20 Up:1 None:20'))  # pb-a3ae-fourth-stone
    a.extend(_holds('Left:4 Left+A:7 Left:25 Up:1 None:8'))  # pb-a3ae-fifth-stone
    a.extend(_holds('Left:4 Left+A:7 Left:25 Up:1 None:20 X:1 None:15'))  # pb-a3ae-orb-approach
    a.extend(_holds('Left:4 Left+A:44 Up+A:1 None:420 A:1 None:180'))  # pb-a3ae-power-bomb-collected
    assert len(a) == 176055
    return a


def extend_wave_gate(a):
    """Observed Bubble return -> ADAD gate re-close, without timing offsets."""
    a = list(a[:152344])
    assert len(a) == 152344
    a.extend(_holds('Right+X:130 None:20'))  # Middle-east Bubble door.
    a.extend(_holds('Right+A+X:20 Right+X:150 None:20'))
    a.extend(_holds('Right+X:80 None:40'))  # AD5E shaft.
    a.extend(_holds('Left+X:72 None:30'))
    a.extend(_holds('None:25 Right+X:40 Right+A+X:35 Select:1 None:1 Select:1 Right+X:1 Right:25 Y:1 Right+X:150 None:20'))
    a.extend(_holds('B+Right:4 B+Right+A+X:80 None:5'))  # ADAD moving platforms.
    a.extend(_holds('None:10 Right+A+X:56 A+X:35 None:10'))
    a.extend(_holds('B+Right:8 B+Right+A+X:28'))
    a.extend(_holds('Right+A:22 Left:4 Right:4 Left:4 Right:4'))
    a.extend(_holds('Up+X:1 Up:15 Up+X:1 Up:15 Up+X:1 Up:60'))  # Open the blue gate.
    a.extend(_holds('Right+A+X:55 None:20'))  # Crossing triggers re-close.
    assert len(a) == 153648
    return a


def extend_wave_pickup(a):
    """Observed ADAD gate exit -> fresh Wave award and completed message."""
    assert len(a) == 153648
    a = list(a)
    a.extend(_holds('Right+X:45 None:20 None:420 A:1 None:180'))  # Missile ledge.
    a.extend(_holds('B+Right:31 B+Right+A:70 None:10'))  # Upper runway edge.
    a.extend(_holds('Right+A:16 Left:2 Left+A:1 Right+A:20'))  # Spike ledge.
    a.extend(_holds('Left:25 B+Right:6 B+Right+A:28 Left:3 Left+A:1 Right+A:25'))
    a.extend(_holds('Right+A:17 Left:2 Left+A:14 Right+A:35 None:10'))  # Clear overhang.
    a.extend(_holds('Right:10 Select:1 None:1 Select:1 Right+X:1 Right:25 Y:1 Right+X:150 None:20'))
    a.extend(_holds('Right+A+X:55 None:100'))  # ADDE orb approach; ends154995.
    a.extend(_holds('Right+A:1 Right+A+X:30 None:100'))  # Fresh Wave award.
    a.extend(_holds('None:300 A:1 None:180'))  # Message dismissal and follow-up.
    assert len(a) == 155607
    return a


def extend_crocomire_shot_regression(a):
    """Observed Wave departure, refill, Crocomire fight and diagonal shot."""
    assert len(a) == 155607
    a = list(a)
    a.extend(_holds('Left+X:180 None:15 Left+A+X:35 Left+X:160 None:15 B+Left+X:200'))
    a.extend(_holds('None:25 Down:1 None:10 Down:1 None:10 Left:120'))
    a.extend(_holds('None:10 Up:1 None:10 Up:1 None:10 Left+A+X:70'))
    a.extend(_holds('Left+X:150 None:15 Right:20 Up+A:5 Left+A:3 Left+A+X:1'))
    a.extend(_holds('A:10 Left+X:150 None:15 B+Left:4 B+Left+A:45 Left+X:50'))
    a.extend(_holds('None:10 Up+A:25 Right+A:35 None:20 Right+X:38 None:5'))
    a.extend(_holds('Right:23 None:10 Left:16 Up+A:25 Left+A:30 None:20'))
    a.extend(_holds('Left:35 None:10 Up+A:25 Right+A:35 None:20 Left:45'))
    a.extend(_holds('None:10 Up+A+X:25 Right+A+X:35 None:20 Right+X:38 None:10'))
    a.extend(_holds('Right:22 None:10 Up+A:25 Left+A+X:35 None:20 Left:3'))
    a.extend(_holds('Left+X:100 None:30 Left+X:140 None:100 Left+A:65 Left:55'))
    a.extend(_holds('None:60 Left+A:70 Left:60 None:80 Left+A:45 None:110'))
    a.extend(_holds('Up+A:8 Down+A:5 Down+A+X:1 None:100 Up+A:8 Down+A:5'))
    a.extend(_holds('Down+A+X:1 None:100 Right:5 None:10 Down:5 None:5'))
    a.extend(_holds('Down:5 None:110 Right:8 None:90 X:1 None:130'))
    a.extend(_holds('X:1 Right:100 None:30 Left:10 X:1 Left:110'))
    a.extend(_holds('None:40 Left:15 Left+X:1 Left:125 None:30 X:1'))
    a.extend(_holds('Left:130 None:20 Up:10 None:15 Left:38 None:15'))
    a.extend(_holds('Up:10 None:15 Up:5 None:10 Up+A:15 Left+A:45'))
    a.extend(_holds('Left+X:100 None:30 Right:27 None:14 Left:3 Left+X:40'))
    a.extend(_holds('None:15 Left+X:130 None:70 Left+X:75 None:40 Left:13'))
    a.extend(_holds('None:10 Up+A:8 Down+A:5 Down+A+X:1 None:85 Left:65'))
    a.extend(_holds('None:10 Up+A:5 Left+A:3 Left+A+X:1 Left+X:70 None:20'))
    a.extend(_holds('Right:35 None:10 Left:3 None:5 Up+A:7 Left+A:2'))
    a.extend(_holds('Left+A+X:1 Left:85 None:20 Left+X:140 None:60 Left:21'))
    a.extend(_holds('None:100 Right:5 None:110 Left:13 None:10 Up+A:8'))
    a.extend(_holds('Down+A:5 Down+A+X:1 None:165 Left:11 None:8 Up+A:8'))
    a.extend(_holds('Down+A:5 None:15 Down:4 Down+X:1 None:180 Right:3'))
    a.extend(_holds('Right+X:100 None:70 Right:35 None:240 Right:22 None:220'))
    a.extend(_holds('Left:3 Left+A:15 Left+A+X:1 Left:125 None:60 Left+A:8'))
    a.extend(_holds('Left+A+X:1 Left:130 None:65 B+Left+X:180 None:70 B+Left+A:65'))
    a.extend(_holds('B+Left+X:50 B+Left+A:65 B+Left+X:50 B+Left+A:65 B+Left+X:100 None:75'))
    a.extend(_holds('Left:30 None:45 Select:1 None:2 Select:1 None:5'))
    a.extend(_holds('Left:14 None:8 Up+A:5 Down+A:3 Down+A+X:1 None:165'))
    a.extend(_holds('Y:1 Right:25 X:80 None:15 Right:25 X:70'))
    a.extend(_holds('None:20 X:70 None:20 X:70 A+X:10 A:1'))
    a.extend(_holds('None:70 Right:15 X:65 A+X:10 A:1 None:65'))
    a.extend(_holds('X:65 A+X:10 A:1 None:65 Right:10 X:65'))
    a.extend(_holds('A+X:10 A:1 None:65 Right:20 X:65 A+X:10'))
    a.extend(_holds('A:1 None:65 Left+X:30 Right+X:3 X:130 A+X:10'))
    a.extend(_holds('A:1 None:35 X:65 A+X:10 A:1 None:30'))
    a.extend(_holds('Right+X:12 X:53 A+X:10 A:1 None:35 Right+X:25'))
    a.extend(_holds('X:60 Right+X:12 X:40 A+X:10 A:1 X:115'))
    a.extend(_holds('A+X:10 A:1 None:35 Select:1 None:1 Select:1'))
    a.extend(_holds('A:10 A+X:1 None:12 X:1 None:25 A:15'))
    a.extend(_holds('A+X:1 None:25 X:1 None:35 A:13 A+X:1'))
    a.extend(_holds('None:30 Y:1 X:65 A+X:14 A:1 None:30'))
    a.extend(_holds('Right+X:25 X:40 A+X:14 A:1 None:30 X:70'))
    a.extend(_holds('A+X:14 A:1 None:30 Right+X:12 X:98 A+X:14'))
    a.extend(_holds('A:1 None:30 X:65 A+X:14 A:1 None:30'))
    a.extend(_holds('Right:20 Select:1 None:200 A:14 A+X:1 None:10'))
    a.extend(_holds('X:1 None:30 X:1 None:25 A:14 A+X:1'))
    a.extend(_holds('None:10 X:1 None:30 Y:1 Right+X:20 X:45'))
    a.extend(_holds('A+X:14 A:1 Select:1 None:9 X:1 None:30'))
    a.extend(_holds('Y:1 Right+X:15 X:110 Left+X:15 Right+X:3 X:30'))
    a.extend(_holds('Left+X:20 Right+X:3 X:25 X+R:44 R:1 Select+R:1'))
    a.extend(_holds('R:10 X+R:1 R:10 X+R:1 R:20'))
    assert len(a) == 167295
    return a


def extend_crocomire_completion(a):
    # Continue the qualified fight, then both melts and the wall/skeleton sequence.
    a = a[:167285]
    a.extend(_holds('X+R:1 R:10 X+R:1 R:10 X+R:1 R:35'))
    a.extend(_holds('Y:1 Right+X:15 X+R:101 R:1 Select+R:1 R:29'))
    a.extend(_holds('X+R:1 R:30 Y:1 Right+X:23 X+R:158 R:1'))
    a.extend(_holds('X+R:85 Right+X:14 X+R:50 R:1 X+R:80 Right+X:22'))
    a.extend(_holds('X+R:15 R:1 X+R:60 Right+X:14 X+R:33 R:1'))
    a.extend(_holds('X+R:60 Right+X:20 X+R:28 R:1 X+R:60 Right+X:20'))
    a.extend(_holds('X+R:19 R:1 X+R:80 R:1 X+R:80 Right+X:22'))
    a.extend(_holds('X+R:3 R:1 X+R:60 Right+X:20 X+R:32 R:1'))
    a.extend(_holds('X+R:60 Right+X:20 X+R:40 R:1 X+R:70 None:1480'))
    a.extend(_holds('B+Left:120 None:30 Left:18 None:1265'))
    assert len(a) == 171694
    return a


def route_frames(source, name):
    """Reproduce authored controller holds/splices; do not adjust emulator timing.

    A jumps, X fires and B runs in these original coverage routes. Splices
    count input frames from boot and exclude the endpoint. Original source
    commands are identified beside their authored expressions. The CLI validates
    the source movie and canonical output hashes before writing a route.
    """
    a = source
    for extend in (_early_route, _missile_route, extend_missile, extend_parlor,
                   extend_bombs, extend_hill, extend_green, extend_super):
        for milestone, a in extend(a):
            if milestone == name:
                return a
    if name == 'save-death-continue':
        # Retain the saved-slot route through observed contact input 72760.
        # Authored idle exposure, then fresh A presses for Continue and both maps.
        return a[:72761] + _holds('None:1800 A:1 None:300 A:1 None:300 A:1 None:900')
    if name in ('crocomire-shot', 'crocomire'):
        route = extend_crocomire_shot_regression(extend_wave_pickup(extend_wave_gate(extend_ice(extend_redtower(a)))))
        return extend_crocomire_completion(route) if name == 'crocomire' else route
    if name in ('wave-gate', 'wave-beam'):
        route = extend_wave_gate(extend_ice(extend_redtower(a)))
        return extend_wave_pickup(route) if name == 'wave-beam' else route
    if name == 'norfair-reserve':
        return extend_reserve(extend_redtower(a))
    if name in ('power-bomb-room-entry', 'power-bomb'):
        entry = extend_power_bomb_entry(extend_ice_return(extend_ice(extend_redtower(a)) + _hold('', 120)))
        return extend_power_bomb_pickup(entry) if name == 'power-bomb' else entry
    if name == 'ice-return-business':
        return extend_ice_return(extend_ice(extend_redtower(a)) + _hold('', 120))
    if name in ('ice-beam', 'ice-beam-settled'):
        frames = extend_ice(extend_redtower(a))
        # Authored neutral tail for 120 follow-up frames after Ice's message.
        return frames + _hold('', 120) if name == 'ice-beam-settled' else frames
    if name == 'speed-booster-settled':
        # Authored follow-up: release all buttons after the late pickup message.
        return extend_redtower(a) + _hold('', 120)
    return extend_redtower(a)[:ROUTES[name][0]]


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('movie', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--rom', type=Path, required=True)
    parser.add_argument('--route', choices=tuple(ROUTES), help='apply an authored coverage route')
    parser.add_argument('--lag-log', type=Path, help='lsnes CSV with input and boundary companions')
    parser.add_argument('--frames', type=int, help='number of observed wait steps to import')
    args = parser.parse_args()
    if ((args.lag_log and (args.route or args.frames is None))
            or (args.frames is not None and not args.lag_log)):
        parser.error('--lag-log requires --frames and cannot combine with --route')
    if args.route and hashlib.sha256(args.movie.read_bytes()).hexdigest() != ROUTE_SOURCE_SHA256:
        parser.error('--route requires the original any% movie from movies/manifest.toml')
    frames = load(args.movie, args.rom)
    if args.lag_log:
        frames = aligned_wait_prefix(frames, args.lag_log, args.frames)
    if args.route:
        frames = route_frames(frames, args.route)
        data = ''.join(f'{word:04X}\r\n' for word in frames).encode('ascii')
        count, expected = ROUTES[args.route]
        if len(frames) != count or hashlib.sha256(data).hexdigest() != expected:
            raise ValueError('authored route output length or SHA-256 mismatch')
        args.output.write_bytes(data)
    else:
        args.output.write_text(''.join(f'{word:04X}\n' for word in frames), encoding='ascii')
    print(f'Imported {len(frames)} controller frames')
