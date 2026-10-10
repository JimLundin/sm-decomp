"""Source-owned timed patterns and versioned, independently checkable scope."""
import hashlib
import json
from pathlib import Path

DECLARATION = 'crates/recomp/src/timed_patterns.json'
FIXTURES = 'tools/pattern_fixtures.json'


def identity(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def load_declaration(root, override=None, prospective=None):
    root = Path(root)
    raw = (root / DECLARATION).read_bytes()
    if override is not None and Path(override).read_bytes() != raw:
        raise ValueError('Pattern declaration differs from production source declaration')
    declaration = json.loads(raw)
    return project_declaration(declaration,prospective)


def project_declaration(declaration,prospective=None):
    if prospective:
        candidates=[row for row in declaration.get('prospective_patterns',[]) if row['opcode']==prospective]
        if len(candidates)!=1:raise ValueError('Unknown prospective pattern; independent implementation decision required')
        declaration=dict(declaration,patterns=candidates)
        for field in ('timing_tests','timing_oracles','hardware_conditions'):declaration[field]=candidates[0][field]
    validate_declaration(declaration)
    return declaration


def validate_declaration(declaration):
    if declaration.get('schema_version') != 2 or not declaration.get('patterns'):
        raise ValueError('Pattern declaration version/scope missing')
    if declaration.get('execution_modes') != ['native'] or set(declaration.get('decimal_states', [])) != {False, True}:
        raise ValueError('Pattern declaration execution modes unsupported or incomplete')
    for field in ('timing_tests', 'timing_oracles', 'timing_sources', 'hardware_conditions', 'exclusions'):
        if not declaration.get(field):
            raise ValueError('Pattern declaration missing ' + field)
    seen = set()
    for pattern in declaration['patterns']:
        opcode = pattern.get('opcode', '')
        if len(opcode) != 2 or opcode in seen or not 0 <= int(opcode, 16) <= 255:
            raise ValueError('Invalid or duplicate declared opcode')
        seen.add(opcode)
        for field in ('mnemonic', 'mode', 'addressing', 'corpus', 'required_conditions', 'independent_fixtures', 'timing_branches', 'inapplicable'):
            if not pattern.get(field):
                raise ValueError('Pattern declaration missing ' + opcode + '/' + field)
        if not pattern.get('mx') or not set(pattern['mx']) <= {0, 1, 2, 3} or len(set(pattern['mx'])) != len(pattern['mx']):
            raise ValueError('Invalid declared M/X widths')
        if pattern['corpus'] != opcode + '.n.json':
            raise ValueError('Declared corpus must cover complete available native opcode file')
        mandatory = {'native-mx-' + str(mx) for mx in pattern['mx']} | {'binary', 'decimal', 'program-bank-wrap'}
        if not mandatory <= set(pattern['required_conditions']):
            raise ValueError('Pattern declaration omits applicable mode/boundary conditions')
        if 'm8' in pattern['timing_branches'] and 'm16' not in pattern['timing_branches'] and any(mx < 2 for mx in pattern['mx']):
            raise ValueError('Pattern declaration omits M16 timing branch')
        if 'x8' in pattern['timing_branches'] and 'x16' not in pattern['timing_branches'] and any(mx % 2 == 0 for mx in pattern['mx']):
            raise ValueError('Pattern declaration omits X16 timing branch')


def pattern_conditions(case, pattern):
    """Input properties only; CPU/memory/timing expectations remain independent."""
    state = case['initial']
    mx = ((state['p'] >> 4) & 3)
    # runtime mx key is M in bit 1, X in bit 0.
    conditions = {'native-mx-' + str(mx), 'decimal' if state['p'] & 8 else 'binary'}
    length = len([row for row in case['cycles'] if row[2].startswith(('dp-', '-p-')) and row[1] is not None])
    if state['pc'] + max(length, 1) > 0xffff:
        conditions.add('program-bank-wrap')
    conditions.add('m8' if mx & 2 else 'm16')
    conditions.add('x8' if mx & 1 else 'x16')
    conditions.add('fixed')
    conditions.add('mx' + str(mx) + ('-decimal' if state['p'] & 8 else '-binary'))
    conditions.update(case.get('coverage', []))
    return sorted(conditions & (set(pattern['required_conditions']) | set(pattern['timing_branches'])))


def validate_report(report, kind='instruction'):
    declaration = report.get('declaration', {})
    validate_declaration(declaration)
    if identity(declaration) != report.get('declaration_sha256') or not report.get('source_sha256'):
        raise ValueError('Pattern declaration/source identity incomplete')
    if report.get('scope') not in ('complete production declaration', 'complete prospective declaration'):
        raise ValueError('Development pattern scope cannot establish acceptance')
    patterns = {row['opcode']: row for row in declaration['patterns']}
    if kind == 'timing':
        if (report.get('returncode') != 0 or set(report.get('oracles', {})) != set(declaration['timing_oracles'])
                or report.get('sources') != declaration['timing_sources']
                or report.get('tests') != declaration['timing_tests']
                or report.get('conditions', {}).get('applicable') != declaration['hardware_conditions']):
            raise ValueError('Declared independent timing scope incomplete')
        return
    if report.get('corpus_verified') is not True or report.get('corpus_archive_sha256') != report.get('pinned_dependency', {}).get('sha256'):
        raise ValueError('Pattern complete pinned corpus verification missing')
    if set(report.get('opcodes', {})) != set(patterns):
        raise ValueError('Pattern declared opcode coverage incomplete')
    for field in ('expected', 'run', 'passed'):
        if report.get(field, 0) <= 0 or report[field] != report['expected']:
            raise ValueError('Pattern complete case count missing')
    if report.get('failed') != 0 or report.get('skipped') != 0:
        raise ValueError('Pattern supported case failed or skipped')
    corpus = {row['opcode']: row for row in report.get('corpus', [])}
    if set(corpus) != set(patterns):
        raise ValueError('Pattern complete corpus identities missing')
    for opcode, pattern in patterns.items():
        count = report['opcodes'][opcode]
        expected = corpus[opcode]['cases'] + len(pattern['independent_fixtures'])
        if expected <= 0 or count.get('expected') != expected or any(count.get(key) != expected for key in ('run', 'passed')) or count.get('failed') != 0 or count.get('skipped') != 0 or not corpus[opcode].get('sha256'):
            raise ValueError('Pattern complete opcode corpus/fixtures missing')
        coverage = report.get('coverage', {}).get(opcode, {})
        for condition in pattern['required_conditions'] + pattern['timing_branches']:
            row = coverage.get(condition, {})
            if row.get('expected', 0) <= 0 or row.get('passed') != row['expected']:
                raise ValueError('Pattern declared condition/branch missing: ' + opcode + '/' + condition)
        if count.get('independent_fixtures') != pattern['independent_fixtures']:
            raise ValueError('Pattern independent coverage substitution incomplete')
    if sum(row['expected'] for row in report['opcodes'].values()) != report['expected']:
        raise ValueError('Pattern aggregate count differs')
