# Security policy

temet-work runs coding agents and commands on your machine. Its daemon uses the GitHub
CLI, `gh`, with the sign-in `gh` already has, to read, comment on and close issues and to
merge pull requests. For a project whose code host is GitLab it uses `glab` the same way,
to read merge requests and merge them. It starts Claude Code sessions with
`--permission-mode auto` in herdr-core terminals, and runs each project's `deploy` and
`verify` commands from your config with `sh`. A security problem is any way for someone
other than you to make it do one of these, or to read a token it passes on.

## Reporting a vulnerability

Report it privately through GitHub: on [temet-ai/temet-work](https://github.com/temet-ai/temet-work),
open the **Security** tab and choose **Report a vulnerability**. Please do not open a public
issue for it.

Say what you found, how to reproduce it, and which version (`temet-work parts` prints it)
and platform you used. We aim to reply within a week. Fixes ship in the next release rather
than as patches to older ones.

## Supported versions

Only the latest release is supported.

## What temet-work does with your credentials

temet-work has no setting for a credential. GitHub access goes through `gh` and its own
sign-in, and GitLab access through `glab` and its own. When `GITHUB_TOKEN` or `GH_TOKEN`
is set in the daemon's environment, the daemon passes it to each agent it starts, with
`HTTPS_PROXY`, `NO_PROXY` and `NODE_EXTRA_CA_CERTS`, over herdr-core's local socket.
temet-work writes none of these to disk.

Each agent the daemon starts gets a random 256-bit secret in `TEMET_WORK_AGENT_TOKEN`, which
tells the daemon which agent sent a command (`docs/commands.md` says what it checks and
what it does not). The daemon makes the secret when it starts the agent and passes it, in
the agent's environment, over herdr-core's local socket; herdr-core passes it to the other
machine for an agent there, and that agent sends it back with each command over SSH. The
event log keeps the secret's SHA-256 digest, and the secret is in no log or reply.

The event log (`docs/privacy.md`) records the text `gh` or `glab` returns when it refuses
a merge and the output of each verify command, so a command that prints a secret puts it
in the log. temet-work sets no file permissions itself: the config file, the event log and
the daemon's socket get the permissions your umask gives them.
