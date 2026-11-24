$id-2685837321855886
title: Explicit publish/unpublish publication, never implicit storage exposure
date: 2026/11/23
source: Antonina board issue jz99QdHOXdbtvKRbbitcbC-hail (transcribed owner request)
kind: requirement

No component may assume that private open_disk storage is directly servable by
nginx or another static host. App, version and static-asset exposure must be an
explicit operation: a named `syscall.publish` / `syscall.unpublish` import in
the root authority boundary, with a private operator-disk mailbox carrying the
bounded UTF-8 relative public path under a unique u32 ticket. Program 0 remains
a generic token-checking proxy with no publication policy. Only root programs
may import the syscalls; a dedicated generic publication root program performs
publication, while untrusted children receive no direct host imports and no
publication authority.

Only public regular files may be published. Tokens, private operator data and
private mutable objects must never be published, and no direct nginx alias of
storage may be created. A publication must be revoked before the file it names
is deleted or replaced, so a dangling link cannot re-expose later bytes.
Version switching and rollback must preserve this revocation order. Native and
HTTP providers must share the same four-byte syscall semantics.
