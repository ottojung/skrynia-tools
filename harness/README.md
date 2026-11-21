# Theorem Lab real-P2 browser acceptance harness

This directory holds the disposable acceptance fixture for the native Theorem
Lab feedback client. It drives real headless Chromium over nginx HTTPS through
a framing-only trusted edge, the compiled secure kernel, clean dumb Program0,
the ordinary native P1 service and the actual feedback P2 guest from this
repository's `examples/theorem_lab_feedback_p2` source.

Run:

```
python3 harness/real_feedback_p2_chromium.py --keep
```

`--keep` preserves the disposable work directory for log inspection; without it
the directory is removed after the run. The script pins every artifact by
SHA-256 and refuses to run if any binary differs.

Pinned artifacts (all disposable-test only):

| artifact | SHA-256 |
| --- | --- |
| secure kernel | `8da114ff4c6c835621eca71a2840c4262c04b9ac1d37705b058dc2f3f9168d0f` |
| clean Program0 | `d57ab27aacc01f6d76b272aed8621ce9af0f3000ed1de2309ec533cbf181a5fa` |
| ordinary P1 | `83a65a2f655ec466f94e57daf25163de60ecf8793bfc6e1be2e0608075b1d80a` |
| feedback P2 | `5a26657858f515d29deaff012fc0109ea2d03a428d19895255239e8b780ad2bc` |
| browser client (`feedback.js`) | `9e680491d51afdaa1d627c62c2aa558d6d917d273172241466113401c35c393c` |

The browser client file is the unmodified `source/theorem-lab/feedback.js` from
skrynia-apps PR306, which stacks on PR305. The edge validates Host, Origin and
session and injects the private Program0 bearer only; it forwards the browser's
binary request bytes unchanged and performs no application-data work. Reads
are a scoped public `OP_APP_GET`; writes go through the real P2's own bounded
CAS merge. No privileged app write capability, root token, production DNS, TLS
certificate or real secret is used.
