"""Non-vacuous security predicates for the real-stack Theorem P2 fixture.

The predicates are pure and std-only so CI can check that the fixture's
security assertions actually demand denial/non-leak without needing the
compiled kernel, Program 0, P1 or P2 artifacts. A predicate returns True only
for the *safe* outcome; any successful forge or leak makes it False.
"""
import json


def forge_denied(d):
    """A forged capability must not read back a forged key as a successful op."""
    return d.get('kind') != 'ok'


def sibling_no_leak(d):
    """A sibling must receive no successful response carrying victim state."""
    if d.get('kind') != 'ok':
        return True
    try:
        state = json.loads(d.get('body', b''))
    except Exception:
        return False
    return not any(
        entry.get('id') == 'secret' for entry in state.get('feedback', [])
    )
