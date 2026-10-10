"""Versioned complete readiness domain and mandatory evidence contract."""
CONTRACT_VERSION = 2
DOMAIN = {'domain': 'Japan/USA any%', 'frames': 18000}
MANDATORY_CHECKS = ('preflight', 'check', 'lint', 'instruction', 'timing',
                    'fallback-only-parity', 'generated-parity', 'changed-entry-coverage',
                    'synchronization', 'checkpoint-validation', 'feedback-demonstrations',
                    'instruction-diagnosis')
CANDIDATE_CHECKS = ('generated-parity', 'changed-entry-coverage', 'instruction-diagnosis')
REUSABLE_CHECKS = ('fallback-only-parity', 'synchronization', 'check', 'lint')


def validate_gate_scope(name, report, contract_version=None):
    """Reject a self-described success that omitted its mandatory scope."""
    if report.get('status') != 'pass' or report.get('complete') is not True:
        raise ValueError('Mandatory gate is failed or incomplete: ' + name)
    if contract_version not in (None, 1, 2):
        raise ValueError('Unknown readiness evidence contract version')
    extensible = name in ('instruction', 'timing') and report.get('schema_version') == 2
    if name in ('instruction', 'timing') and contract_version == 2 and not extensible:
        raise ValueError('Fresh pattern evidence requires extensible declaration contract')
    if extensible:
        from pattern_contract import validate_report
        validate_report(report, name)
    if name in ('fallback-only-parity', 'generated-parity', 'synchronization'):
        if report.get('expected_frames') != 18000 or report.get('completed_frames') != 18000:
            raise ValueError('Mandatory gate frame scope incomplete: ' + name)
    if name in ('fallback-only-parity', 'generated-parity') and report.get('matched_frames') != 18000:
        raise ValueError('Mandatory parity matched scope incomplete')
    if name == 'generated-parity' and (report.get('generated_steps', 0) <= 0 or report.get('compiled_table_entries', 0) <= 0):
        raise ValueError('Generated gate lacks actual compiled execution')
    if name == 'fallback-only-parity' and report.get('generated_steps') != 0:
        raise ValueError('Fallback gate executed compiled code')
    if name == 'instruction' and not extensible and (report.get('expected') != 70011 or report.get('run') != 70011
                                  or report.get('failed', 0) != 0 or report.get('skipped', 0) != 0):
        raise ValueError('Mandatory instruction corpus scope incomplete')
    if name == 'instruction-diagnosis' and (report.get('expected_instructions', 0) <= 0 or
                                            report.get('completed_instructions') != report.get('expected_instructions')):
        raise ValueError('Focused instruction scope incomplete')
    if name == 'checkpoint-validation' and (report.get('completed_frames') != 1 or report.get('observed_rollovers') != 2):
        raise ValueError('Checkpoint cold validation scope incomplete')

    if name == 'instruction' and not extensible:
        if report.get('passed') != 70011 or set(report.get('opcodes', {})) != {'a9','a2','c2','e2','8d','ea','ca'}:
            raise ValueError('Instruction completed case/opcode set differs')
        for opcode in report['opcodes'].values():
            if opcode.get('expected',0) < 10000 or opcode.get('run') != opcode['expected'] or opcode.get('passed') != opcode['expected'] or opcode.get('failed') != 0 or opcode.get('skipped') != 0:
                raise ValueError('Instruction opcode coverage incomplete')
        if sum(opcode['expected'] for opcode in report['opcodes'].values()) != 70011:
            raise ValueError('Instruction expected per-opcode count differs')
    if name == 'timing' and not extensible:
        expected = {'crates/snes/tests/hdma_oracle.csv','crates/snes/tests/irq_mask_timing.rs','crates/snes/tests/interrupt_timing.rs','crates/snes/tests/dma_timing.rs'}
        if report.get('returncode') != 0 or set(report.get('oracles',{})) != expected or not report.get('sources') or not report.get('conditions',{}).get('applicable'):
            raise ValueError('Independent timing scope/oracles incomplete')
    if name == 'synchronization':
        if report.get('checkpoint_pc') != '828948' or report.get('start_frame') != 272 or report.get('end_frame') != 18000 or report.get('candidate_frame_offset') != 0 or report.get('events',0) <= 0 or report.get('candidate_completed_frames') != 18000:
            raise ValueError('Strict independent synchronization scope differs')
    if name == 'changed-entry-coverage':
        executed = report.get('executed',[])
        if (report.get('generated_instructions',0) <= 0 or report.get('uncovered_keys') != [] or not executed or report.get('validated_distinct_keys') != len(executed)
                or not report.get('candidate_inventory_sha256') or not report.get('profile_sha256')):
            raise ValueError('Changed-entry actual execution evidence incomplete')
        for key in report.get('required_keys',[]):
            if not any(row.get('address') == key.get('address') and row.get('mx') == key.get('mx') and row.get('count',0)>0 for row in executed):
                raise ValueError('Required changed key did not execute')
    if name == 'checkpoint-validation':
        for side in ('reference','subject'):
            detail=report.get(side,{})
            if detail.get('observed_rollovers') != 2 or detail.get('instructions',0) <= 0 or not detail.get('ordered_trace_sha256') or not detail.get('ending_sha256'):
                raise ValueError('Checkpoint complete cold side validation missing')
    if name == 'feedback-demonstrations':
        required={'correct-execution','wrong-instruction','transient-effect','uncovered-key','empty-compilation','preflight-build-failure','interrupted-truncated','incompatible-stale','paired-cache'}
        cases=report.get('cases',{})
        if set(cases) != required or report.get('expected_cases') != len(required) or report.get('completed_cases') != len(required) or any(case.get('status') != 'pass' or case.get('complete') is not True for case in cases.values()):
            raise ValueError('Feedback demonstrations incomplete')
