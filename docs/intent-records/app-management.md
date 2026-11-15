$id-8316538020433252
title: Architecture-independent application version management
date: 2026/11/15
source: @ottojung
kind: requirement

Provide pleasant developer-facing CLI tools for deployed applications: enumerate applications and the historical versions of an application, inspect specific releases, and restore application code to any previously deployed release. The tooling must not assume a particular application's data or software architecture.

$id-1773745508913857
title: Constant-work rollback without data migrations
date: 2026/11/15
source: @ottojung
kind: requirement

Code rollback to any retained release must use O(1) time and O(1) extra memory with respect to the number and size of prior releases. Rollback is not a rebuild, does not copy old code and must not run migrations. Database or application-data migration support is a distinct, future feature.
