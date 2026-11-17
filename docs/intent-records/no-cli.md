$id-2918869708597732
title: Remove the Skrynia Tools command-line interface entirely
date: 2026/11/17
source: @ottojung
kind: requirement

Skrynia Tools must not expose any CLI functionality. Remove its
CLI executable, command parser, installation package, tests and
documentation. Administration must take place through the Skrynia
hosted web interface. The one-time Bash installation bootstrap and
noninteractive GitHub Actions for application publishing remain
distinct from a user-facing CLI.
