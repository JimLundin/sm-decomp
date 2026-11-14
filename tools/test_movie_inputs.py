"""Constructed capture and CLI contracts; no captured movie data is required."""
import contextlib
import csv
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import movie_inputs as movie


VIDEO = [{'nmi_before': '0', 'nmi_after': '1'}] * 4


def events(*spec):
    rows = []
    for number, (kind, overrides) in enumerate(spec, 1):
        event = dict(event=number, movie_frame=1, nmi=0, state=0, room=0,
                     joypad=0x100, including_lag=0, x=0, y=0, scroll_frame=0)
        event.update(overrides)
        rows.append(dict(kind=kind, **{key: str(value) for key, value in event.items()}))
    return rows


def normal(nmi=0):
    return [('wait_enter', {'nmi': nmi}), ('joypad_begin', {'nmi': nmi}),
            ('joypad_read', {'nmi': nmi}), ('wait_exit', {'nmi': nmi + 1})]


class WorkMapping(unittest.TestCase):
    def test_typed_mapping_matches_compatibility_rows(self):
        rows = events(('joypad_begin', {}), ('joypad_read', {}), *normal(1))
        spans = list(movie.mapped_work_spans(VIDEO, rows))
        self.assertEqual([span.as_row() for span in spans],
                         list(movie.mapped_work_events(VIDEO, rows)))
        self.assertEqual([span.status for span in spans],
                         [movie.WorkStatus.GAP, movie.WorkStatus.BLOCKED])
        self.assertTrue(all(span.status.rejects_prefix and span.word is None
                            and span.verifier_step is None for span in spans))
        span, = movie.mapped_work_spans(VIDEO, events(*normal()))
        self.assertIs(span.status, movie.WorkStatus.SUPPORTED)
        self.assertFalse(span.status.rejects_prefix)
        self.assertEqual((span.word, span.verifier_step), (0x100, 0))

    def test_normal_wait_exact_row(self):
        self.assertEqual(list(movie.mapped_work_events(VIDEO, events(*normal()))), [dict(
            event_begin=1, event_end=4, video_begin=1, video_end=1,
            native_read_begin=0, native_read_end=1, boundary='normal_wait',
            status='supported', verifier_step=0, settled=True, word=0x100)])

    def test_gap_permanently_blocks_later_complete_wait(self):
        rows = events(('joypad_begin', {}), ('joypad_read', {}), *normal(1))
        mapped = list(movie.mapped_work_events(VIDEO, rows))
        self.assertEqual([r['status'] for r in mapped], ['gap', 'blocked'])
        self.assertTrue(all('word' not in r and r['verifier_step'] == '' for r in mapped))
        self.assertEqual([(r['event_begin'], r['event_end'], r['native_read_begin'],
                           r['native_read_end']) for r in mapped], [(1, 2, 0, 1), (3, 6, 1, 2)])
        with self.assertRaises(ValueError):
            movie.aligned_work_prefix(VIDEO, rows, 1)

    def test_no_update_requires_exactly_one_lag_nmi(self):
        for lag_count in (0, 1, 2):
            with self.subTest(lag_count=lag_count):
                rows = events(('no_update_enter', {}),
                              *[('lag_nmi', {})] * lag_count,
                              ('no_update_exit', {'including_lag': 1}))
                if lag_count == 1:
                    self.assertEqual(movie.aligned_work_prefix(VIDEO, rows, 1), [0x100])
                else:
                    with self.assertRaises(ValueError):
                        list(movie.mapped_work_events(VIDEO, rows))

    def test_no_update_rejects_changed_input(self):
        with self.assertRaises(ValueError):
            list(movie.mapped_work_events(VIDEO, events(('no_update_enter', {}),
                 ('lag_nmi', {}), ('no_update_exit', {'including_lag': 1, 'joypad': 0}))))

    def test_door_suspension_is_unsettled_until_scroll_irq(self):
        specs = [('place_enter', {'state': 11}), ('vblank_wait_enter', {}),
                 ('vblank_wait_exit', {}), ('scroll_irq_enter', {'including_lag': 1}),
                 ('scroll_irq_exit', {'scroll_frame': 1}), ('more_exit', {})]
        mapped = list(movie.mapped_work_events(VIDEO, events(*specs)))
        self.assertEqual([r['boundary'] for r in mapped],
                         ['door_vblank', 'door_settlement', 'evidence'])
        self.assertEqual([r['settled'] for r in mapped], [False, True, True])
        self.assertEqual(movie.aligned_work_prefix(VIDEO, events(*specs), 1), [0x100])

    def test_door_eof_empty_range_marks_incomplete(self):
        mapped = list(movie.mapped_work_events(VIDEO, events(
            ('place_enter', {'state': 11}), ('joypad_begin', {}), ('joypad_read', {}))))
        self.assertEqual([r['status'] for r in mapped], ['supported', 'incomplete'])
        self.assertEqual((mapped[-1]['event_begin'], mapped[-1]['event_end']), (4, 3))
        self.assertTrue(mapped[-1]['settled'])

    def test_pending_wait_eof(self):
        row, = movie.mapped_work_events(VIDEO, events(('wait_enter', {})))
        self.assertEqual((row['boundary'], row['status'], row['settled']),
                         ('pending_eof', 'incomplete', False))

    def test_cross_frame_read_requires_adjacent_event(self):
        specs = normal()
        specs[2] = ('joypad_read', {'movie_frame': 2})
        specs[3] = ('wait_exit', {'movie_frame': 2, 'nmi': 1})
        self.assertEqual(movie.aligned_work_prefix(VIDEO, events(*specs), 1), [0x100])
        specs.insert(2, ('irq_enter', {}))
        with self.assertRaises(ValueError):
            list(movie.mapped_work_events(VIDEO, events(*specs)))

    def test_seed_ordinals_and_counter_wrap(self):
        specs = normal(65535)
        specs[-1] = ('wait_exit', {'nmi': 0})
        rows = events(*specs)
        for row in rows:
            row['event'] = str(int(row['event']) + 10)
            row['movie_frame'] = '20'
        previous = dict(nmi=65534, state=0, movie_frame=20)
        row, = movie.mapped_work_events(VIDEO, rows, 10, previous, 20, 7)
        self.assertEqual((row['event_begin'], row['event_end'], row['native_read_begin'],
                          row['native_read_end']), (11, 14, 7, 8))

    def test_prefix_drains_unused_csv_suffix(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'video.csv'
            path.write_text('movie_frame,input_frame,input_callbacks,nmi_before,nmi_after\n1,0,1,0,1\n')
            Path(str(path) + '.polls.csv').write_text(
                'movie_frame,callback,subframe,movie_subframe_index,movie_subframe\n1,1,0,0,F. 0 0|............\n')
            rows = events(*normal())
            for row in rows:
                row['joypad'] = '0'
            text = io.StringIO()
            writer = csv.DictWriter(text, rows[0].keys())
            writer.writeheader()
            writer.writerows(rows)
            text.write('x' * (csv.field_size_limit() + 1) + '\n')
            Path(str(path) + '.events.csv').write_text(text.getvalue())
            with self.assertRaises(csv.Error):
                movie.aligned_wait_prefix([0], path, 1)


class EventMapPublication(unittest.TestCase):
    def capture(self, directory):
        path = Path(directory) / 'capture'
        path.write_bytes(b'constructed video')
        Path(str(path) + '.polls.csv').write_bytes(b'constructed polls')
        Path(str(path) + '.events.csv').write_bytes(b'constructed events')
        files = {name: Path(directory) / name for name in ('movie', 'rom', 'native', 'native_source')}
        import zipfile
        with zipfile.ZipFile(files['movie'], 'w') as archive:
            archive.writestr('input', b'constructed movie member')
        for role in ('rom', 'native', 'native_source'):
            files[role].write_bytes(role.encode('ascii'))
        return path, files, Path(directory) / 'map.csv'

    def test_only_settled_prefix_is_pinned(self):
        import hashlib
        import tomllib
        with tempfile.TemporaryDirectory() as directory:
            path, files, output = self.capture(directory)
            specs = normal() + [('joypad_begin', {'nmi': 1}), ('joypad_read', {'nmi': 1})] + normal(2)
            steps = movie.write_work_event_map(VIDEO, events(*specs), path, output, files)
            with output.with_suffix('.csv.toml').open('rb') as stream:
                manifest = tomllib.load(stream)
            self.assertEqual(steps, 1)
            self.assertEqual(manifest['qualified_steps'], 1)
            self.assertEqual(manifest['original_events'], 10)
            self.assertEqual(manifest['original_native_reads'], 3)
            self.assertEqual(manifest['canonical_input_prefix_sha256'],
                             hashlib.sha256(b'0100\r\n').hexdigest())
            self.assertEqual(manifest['map_sha256'], movie.file_sha256(output))
            self.assertFalse(manifest['complete'])
            self.assertEqual(manifest['first_gap_event_begin'], 5)
            self.assertFalse(output.with_name(output.name + '.pending').exists())

    def test_unsettled_door_word_does_not_extend_qualified_hash(self):
        import hashlib
        import tomllib
        with tempfile.TemporaryDirectory() as directory:
            path, files, output = self.capture(directory)
            specs = normal() + [('place_enter', {'state': 11}),
                                ('vblank_wait_enter', {}), ('vblank_wait_exit', {})]
            self.assertEqual(movie.write_work_event_map(VIDEO, events(*specs), path, output, files), 1)
            with output.with_suffix('.csv.toml').open('rb') as stream:
                manifest = tomllib.load(stream)
            self.assertEqual(manifest['qualified_steps'], 1)
            self.assertEqual(manifest['canonical_input_prefix_sha256'],
                             hashlib.sha256(b'0100\r\n').hexdigest())
            self.assertFalse(manifest['complete'])

    def test_identity_change_prevents_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            path, files, output = self.capture(directory)
            def changed_events():
                yield from events(*normal())
                files['native'].write_bytes(b'changed identity')
            with self.assertRaises(ValueError):
                movie.write_work_event_map(VIDEO, changed_events(), path, output, files)
            self.assertFalse(output.exists())
            self.assertFalse(output.with_suffix('.csv.toml').exists())
            self.assertFalse(output.with_name(output.name + '.pending').exists())


class MovieCli(unittest.TestCase):
    def resolve(self, *flags):
        parser = movie.movie_parser()
        return movie.movie_operation(parser.parse_args(['movie', 'output', '--rom', 'rom', *flags]), parser)

    def test_admitted_operations(self):
        cases = [([], movie.MovieOperation.IMPORT),
                 (['--route', next(iter(movie.ROUTES))], movie.MovieOperation.ROUTE),
                 (['--lag-log', 'capture', '--frames', '1'], movie.MovieOperation.ALIGN),
                 (['--event-map-only', '--lag-log', 'capture', '--native', 'native',
                   '--native-source', 'source'], movie.MovieOperation.EVENT_MAP),
                 (['--snes9x-capture', 'capture', '--snes9x-capture-sha256', 'pin',
                   '--seed', 'seed', '--frames', '1'], movie.MovieOperation.SNES9X),
                 (['--native-phase-receipt', 'receipt', '--native-phase-receipt-sha256', 'pin',
                   '--native-phase-inputs', 'inputs', '--native-phase-stdout', 'stdout',
                   '--native-phase-logger', 'logger', '--native-phase-predecessor', 'previous',
                   '--native', 'native', '--native-source', 'source'], movie.MovieOperation.NATIVE_PHASE)]
        for flags, operation in cases:
            with self.subTest(operation=operation):
                self.assertIs(self.resolve(*flags), operation)

    def test_conflicts_rejected_before_work(self):
        cases = [['--native', 'native'], ['--frames', '1'], ['--seed', 'seed'],
                 ['--native-phase-inputs', 'inputs'], ['--eof-capture', 'capture'],
                 ['--event-map-only', '--lag-log', 'capture', '--native', 'native',
                  '--native-source', 'source', '--frames', '1'],
                 ['--lag-log', 'capture', '--frames', '1', '--route', next(iter(movie.ROUTES))]]
        for flags in cases:
            with self.subTest(flags=flags), contextlib.redirect_stderr(io.StringIO()):
                with patch.object(movie, 'run_movie_operation') as work:
                    with self.assertRaises(SystemExit) as failure:
                        movie.main(['movie', 'output', '--rom', 'rom', *flags])
                    self.assertEqual(failure.exception.code, 2)
                    work.assert_not_called()

    def test_import_keeps_crlf_hex_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'inputs'
            with patch.object(movie, 'load', return_value=[0, 0x100, 0xffff]), contextlib.redirect_stdout(io.StringIO()):
                movie.main(['movie', str(output), '--rom', 'rom'])
            self.assertEqual(output.read_bytes(), b'0000\r\n0100\r\nFFFF\r\n')


if __name__ == '__main__':
    unittest.main()
