"""Read-only verification of Rust paired envelope digests for evidence import."""
import hashlib
import json


def verify_pair(pair,validation,focus):
    def sha(data):return hashlib.sha256(data).hexdigest()
    for side,tag in (('reference',b'RCP1'),('subject',b'SCP1')):
        raw=bytes(pair[side])
        if raw[:4]!=tag or raw[4:6]!=b'\x01\x00' or sha(raw)!=pair[side+'_sha256']:
            raise ValueError('Paired raw state format/digest differs')
    # serde_json's default map serializer sorts object keys. The wire pair digest
    # binds metadata, side digests and the actual verified cold history.
    fields={name:pair[name] for name in ('schema_version','identities','input_index','pad','history_sha256','reference_sha256','subject_sha256')}
    if sha(json.dumps(fields,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())!=pair['pair_sha256']:
        raise ValueError('Paired metadata digest differs')
    if pair['pair_sha256']!=validation['pair_sha256'] or pair['history_sha256']!=focus['cache_history_sha256'] or validation['identities']!=pair['identities']:
        raise ValueError('Paired validation/history identity differs')
