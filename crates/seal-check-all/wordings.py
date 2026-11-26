# Lists every refusal, failed check and stop wording in the seal crate, and every host mistake the C API reports, for PROGRESS.md's "Draft wordings".
# Run from the repository: python3 -I crates/seal-check-all/wordings.py OUT.md
import re, glob, sys
FILES = sorted(glob.glob('crates/seal/src/**/*.rs', recursive=True)) + sorted(glob.glob('crates/seal-c/src/*.rs'))
CALLS = {
    'refuse_why(': ('refused', 1, 2),
    'refuse(': ('refused', 1, None),
    'Message::refused(': ('refused', 1, None),
    'check_failed(': ('check failed', 0, 2),
    'failed(': ('check failed', 0, 2),
    'not_designed(': ('not designed yet', 1, None),
    'Message::not_designed_yet(': ('not designed yet', 1, None),
    'not_built(': ('not built yet', 1, None),
    'Message::not_built_yet(': ('not built yet', 1, None),
    'door_refusal(': ('refused (at a door)', 1, 2),
    'mistake(': ('host mistake (C API)', 0, None),
}
def args_at(src, i):
    # i points just after '('; returns list of argument texts and end index
    depth, out, cur, j = 1, [], '', i
    while j < len(src):
        c = src[j]
        if c == '"':
            k = j + 1
            while src[k] != '"':
                k += 2 if src[k] == '\\' else 1
            cur += src[j:k+1]; j = k + 1; continue
        if c == "'" and src[j:j+3].endswith("'") and len(src[j:j+3])==3:
            cur += src[j:j+3]; j += 3; continue
        if c in '([{': depth += 1
        if c in ')]}':
            depth -= 1
            if depth == 0:
                out.append(cur.strip()); return out, j
        if c == ',' and depth == 1:
            out.append(cur.strip()); cur = ''; j += 1; continue
        cur += c; j += 1
    return out, j
def text_of(arg):
    if arg is None: return None
    arg = arg.strip().lstrip('&')
    m = re.match(r'^(?:format!\()?\s*"((?:[^"\\]|\\.)*)"', arg, re.S)
    if not m:
        return None
    t = m.group(1)
    t = re.sub(r'\\\s*\n\s*', '', t).replace('\\"', '"').replace('\\\\', '\\')
    t = t.replace('{{', '\x01').replace('}}', '\x02').replace('\x01', '{').replace('\x02', '}')
    return t
seen = set(); out = []
for path in FILES:
    src = open(path).read()
    src = src.split('#[cfg(test)]')[0]
    items = []
    for name, (kind, what_index, why_index) in CALLS.items():
        for m in re.finditer(r'(?<![\w:])' + re.escape(name), src):
            args, end = args_at(src, m.end())
            what = text_of(args[what_index]) if len(args) > what_index else None
            if not what: continue
            why = text_of(args[why_index]) if why_index is not None and len(args) > why_index else None
            tail = src[end:end+300]
            wm = re.match(r'\)\s*\.with_why\(\s*(&?(?:format!\()?"(?:[^"\\]|\\.)*")', tail, re.S)
            if wm and not why: why = text_of(wm.group(1))
            fix = '.with_fix(' in tail[:200]
            if kind == 'not designed yet': what = what + ' isn\'t decided yet'
            if kind == 'not built yet': what = what + ' built yet'
            key = (kind, what, why)
            if key in seen: continue
            seen.add(key)
            items.append((m.start(), kind, what, why, fix))
    if items:
        out.append(f'\n**`{path}`**\n')
        for _, kind, what, why, fix in sorted(items):
            out.append(f'- {kind}: {what}' + (' (shown with a `fix:` line)' if fix else ''))
            if why: out.append(f'  - why: {why}')
print(len(seen))
open(sys.argv[1], 'w').write('\n'.join(out) + '\n')
