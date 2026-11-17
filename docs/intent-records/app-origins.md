$id-1990749427327003
title: Stable application origins and relative links
date: 2026/11/17
source: @ottojung
kind: requirement

Applications should have stable, application-specific browser origins
like kawun.a.vau.place. Their release files should be independently
versioned and all links within one application should work as relative
links without requiring developers to specify a public URL scheme.
A stable entrypoint should choose the currently active release, and
rolling back application code should remain constant-time and
constant-extra-memory. Browser navigation should preserve the incoming
pathname, query string and fragment. Application deployment must
not impose a specific approach to application data.
