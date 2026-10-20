# VPS builds

The repository's `heaven` Linux runner handles backend jobs from trusted pushes
or manual dispatches and the Android signed-release pipeline. Pull-request
tests and security scans continue using GitHub-hosted runners.

Backend builds start a disposable PostgreSQL cluster bound to `127.0.0.1:25432`
under the job's temporary directory. This cluster is independent of the VPS's
application database. Tests retain the separate application and read-only roles
required for row-level-security coverage. The cluster is stopped after the job.

The Android workflow supports manual dispatch to build and upload a signed APK
without publishing a release. Tag-triggered builds retain the existing change
check and GitHub Release publication. Signing keys and signing properties are
kept in the service's private `/run` directory and removed by an always-run step.

All VPS registrations share 4 CPU cores, 10 GiB RAM, one build slot, and a 30 GB
volume for runner files, workspaces, caches, and temporary files. The machine's
job-start hook accepts only trusted owner pushes, schedules, and manual runs.
Toolchains and build caches persist across runs. The runner user has no sudo access.
