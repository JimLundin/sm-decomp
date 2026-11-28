def outcome(status: str, case: str = "audio/example") -> str:
    difference = (
        'cut = 3\nfield = "audio.reply0"\n' if status == "difference" else ""
    )
    return (
        f'format = 1\n[[cases]]\nid = "{case}"\nobservation_sha256 = "original"\n'
        f'[cases.outcome]\nstatus = "{status}"\n{difference}'
        "[cases.counts]\ncuts = 3\nrestorations = 2\n"
    )
