import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import movie_inputs as movie


class MovieCli(unittest.TestCase):
    def test_import_keeps_crlf_hex_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'inputs'
            with patch.object(movie, 'load', return_value=[0, 0x100, 0xffff]), contextlib.redirect_stdout(io.StringIO()):
                movie.main(['movie', str(output), '--rom', 'rom'])
            self.assertEqual(output.read_bytes(), b'0000\r\n0100\r\nFFFF\r\n')


if __name__ == '__main__':
    unittest.main()
