$id-2006686696634046
title: Web-based Skrynia Tools administration published in Skrynia
date: 2026/11/17
source: @ottojung
kind: requirement

The Skrynia Tools administration interface must be a web application, not
a command-line admin interface, and must itself be published as objects in
Skrynia. A one-time Bash bootstrap receives SKRYNIA_URL, a grant for the
applications' data scope, and any other necessary configuration using
environment variables or interactive prompts when absent. It publishes
the interface and prints its URL together with a random access password.
Subsequent administration happens through the web interface.

The web interface must display every retained version of every app and
support atomically changing any application's active code release to
any previous or later retained version. Rollback does not perform data
migrations or copy release files.
