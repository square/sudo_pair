sudo_pair
=========

[![Build Status](https://travis-ci.org/square/sudo_pair.svg?branch=master)](https://travis-ci.org/square/sudo_pair)

`sudo_pair` is a [plugin for sudo][sudo_plugin_man] that requires another
human to approve and monitor privileged sudo sessions.

<p align="center">
  <img width="982" alt="a demonstrated sudo_pair session" src="https://raw.githubusercontent.com/square/sudo_pair/master/demo.gif?token=AAAQ8nqdmjg9ZdBK3dGwl5plM_3IagRVks5a9dxmwA%3D%3D">
</p>

## About

`sudo` is used by engineers daily to run commands as privileged users.
But on some sensitive systems, you really want to ensure that no
individual can act entirely autonomously. At Square, this includes
applications that manage our internal access-control systems, store
accounting ledgers, or even move around real money. This plugin allows
us to ensure that no user can act entirely on their own authority within
these systems.

This plugin and its components are still in prerelease, as we want to
get feedback from the open-source community before officially releasing
1.0.

## Installation

### WARNING: Misconfiguring sudo can lock you out of your machine. Test this in a throwaway environment.

For now, `sudo_pair` must be compiled from source. It is a standard
Rust project, and the following should suffice to build it on any recent
version of Rust:

```sh
git clone https://github.com/square/sudo_pair.git
cd sudo_pair
cargo build --release
```

Once built, the plugin itself will need to be installed in a place where
`sudo` can find it. Generally this is under `/usr/libexec/sudo` (on
macOS hosts it's `/usr/local/libexec/sudo`). An appropriate approval
script must be installed into the `PATH`. A directory must be created
for `sudo_pair` to manage the sockets it uses for communication between
plugin and client. And finally, `sudo` must be configured to load and
use the plugin.

```sh
# WARNING: these files may not be set up in a way that is suitable
# for your system. Proceed only on a throwaway host.

# install the plugin shared library
install -o root -g root -m 0644 ./target/release/libsudopair.dylib /usr/libexec/sudo

# create a socket directory
install -o root -g root -m 0644 -d /var/run/sudo_pair

# install the approval script; as currently configured, it denies access
# to users approving their own sudo session and may lock you out
install -o root -g root -m 0755 ./sample/bin/sudo_approve /usr/bin/sudo_approve

# your `/etc/sudo.conf` may already have entries necessary for sudo to
# function correctly; if this is the case, the two files will need to be
# merged
install -o root -g root -m 0644 ./sample/etc/sudo.conf /etc/sudo.conf

# if these prompts don't work for you, they're configurable via a simple
# templating language explained later in the README
install -o root -g root -m 0644 ./sample/etc/sudo.prompt.user /etc/sudo.prompt.user
install -o root -g root -m 0644 ./sample/etc/sudo.prompt.pair /etc/sudo.prompt.pair
```

## Configuration

The plugin can be provided several options to modify its behavior. These
options are provided to the plugin by adding them to the end of the
`Plugin` line in `/etc/sudo.conf`.

Example:

```
Plugin sudo_pair sudo_pair.so socket_dir=/var/tmp/sudo_pair gids_exempted=42,109
```

The full list of options are as follows:

* `binary_path` (default: `/usr/bin/sudo_approve`)

  This is the location of the approval binary. The approval command itself needs to run under the privileges of the destination user or group, and this is done so using sudo, so it must be exempted from requiring its own pair approval.

* `user_prompt_path` (default: `/etc/sudo_pair.prompt.user`)

  This is the location of the prompt template to display to the user invoking sudo; if no template is found at this location, an extremely minimal default will be printed. See the [Prompts](#prompts) section for more details.

* `pair_prompt_path` (default: `/etc/sudo_pair.prompt.pair`)

  This is the location of the prompt template to display to the user being asked to approve the sudo session; if no template is found at this location, an extremely minimal default will be printed. See the [Prompts](#prompts) section for more details.

* `socket_dir` (default: `/var/run/sudo_pair`)

  This is the path where this plugin will store sockets for sessions that are pending approval. This directory must be owned by root and only writable by root, or the plugin will abort.

* `gids_enforced` (default: `0`)

  This is a comma-separated list of gids that sudo_pair will gate access to. If a user is `sudo`ing to a user that is a member of one of these groups, they will be required to have a pair approve their session.

* `gids_exempted` (default: none)

  This is a comma-separated list of gids whose users will be exempted from the requirements of sudo_pair. Note that this is not the opposite of the `gids_enforced` flag. Whereas `gids_enforced` gates access *to* groups, `gids_exempted` exempts users sudoing *from* groups. For instance, this setting can be used to ensure that oncall sysadmins can respond to outages without needing to find a pair.

  Note that root is *always* exempt.

## Prompts

This plugin allows you to configure the prompts that are displayed to
both users being asked to find a pair and users being asked to approve
another user's `sudo` session. If prompts aren't
[configured](#configuration) (or can't be found on the filesystem),
extremely minimal ones are provided as a default.

The contents of the prompt files are raw bytes that should be printed to
the user's terminal. This allows fun things like terminal processing of
ANSI escape codes for coloration, resizing terminals, and setting window
titles, all of which are (ab)used in the sample prompts provided.

These prompts also [implement](src/template.rs) a simple `%`-escaped
templating language. Any known directive preceded by a `%` character is
replaced by an expansion, and anything else is treated as a literal
(e.g., `%%` is a literal `%`, and `%a` is a literal `a`).

Available expansions:

* `%b`: the name of the appoval _b_inary
* `%B`: the full path to the approval _B_inary
* `%C`: the full _C_ommand `sudo` was invoked as (recreated as best-effort)
* `%d`: the cw_d_ of the command being run under `sudo`
* `%h`: the _h_ostname of the machine `sudo` is being executed on
* `%H`: the _H_eight of the invoking user's terminal, in rows
* `%g`: the real _g_id of the user invoking `sudo`
* `%p`: the _p_id of this `sudo` process
* `%u`: the real _u_id of the user invoking `sudo`
* `%U`: the _U_sername of the user running `sudo`
* `%W`: the _W_idth of the invoking user's terminal, in columns

## Approval Scripts

The [provided approval script](sample/bin/sudo_approve) is just a small
(but complete) example. As much functionality as possible has been moved
into the plugin, with one (important, temporary) exception: currently,
the script must verify that the user approving a `sudo` session is not
the user who is requesting the session.

Other than that, the only thing required of the "protocol" is to:

  * connect to a socket (as either the user or group being `sudo`ed to)
  * wire up the socket's input and output to the user's STDIN and STDOUT
  * send a `y` to approve, or anything else to decline
  * close the socket to terminate the session

As it turns out, you can pretty much just do this with `socat`:

```sh
socat STDIO /path/to/socket
```

The script incldued with this project isn't much more than this. It
performs a few extra niceties (implicitly `sudo`s if necessary, turns
off terminal echo, disables Ctrl-C, and kills the session on Ctrl-D),
but not much more. Ctrl-C was disabled so a user who's forgotten that
this is terminal is being used to monitor another user's session doesn't
instinctively kill it with Ctrl-C.

## Limitations

Sessions under `sudo_pair` can't be used in the middle of a pipe. I'll
consider lifting these restrictions, but doing so is inherently
problematic.

Allowing piped data to standard input, as far as I can tell, likel
results in a complete bypass of the security model here. Commands can
often accept input on `stdin`, and there's no reasonable way to show
this information to the pair.

On the other hand, if `sudo` output is piped to `stdout`, we could
simply log it like we log TTY output. This works, except we print the
prompt itself on `stdout`. We could print the prompt to `stderr`
instead. In retrospect, maybe we should do this. Redirecting to `stderr`
would still be problematic, but at least we get some ability to insert
`sudo` commands at the front of pipes. I'll consider this.

## Security Model

This plugin allows users to `sudo -u ${user}` to become a user or
`sudo -g ${group}` to gain an additional group.

When a user does this, a socket is created that is owned and only
writable by `${user}` (or `${group}`). In order to connect to that
socket, the approver must be able to write to files as that `${user}`
(or `${group}`). In other words, they need to be [on the other side of
the airtight hatchway][airtight-hatchway]. In practical terms, this
means the approver needs to also be able to `sudo` to that user or
group.

To facilitate this, the plugin exempts the approval script. And the
sample approval script automatically detects the user or group you need
to become and runs `sudo -u ${user}` (or `sudo -g ${group}`) implicitly.

As a concrete example, these are the sockets opened for `sudo -u root`,
`sudo -u nobody`, and `sudo -g sys`:

```
drwxr-xr-x   3 root    wheel     96 May  8 09:17 .
s-w-------   1 root    wheel      0 May  8 09:16 1882.29664.sock    # sudo -u root
s-w-------   1 nobody  wheel      0 May  8 09:17 1882.29921.sock    # sudo -u nobody
s----w----   1 root    sys        0 May  8 09:18 1882.29994.sock    # sudo -g sys
```

As a result, the only people who can approve a `sudo` session to a user
or group must *also* be able to `sudo` as that user or group.

Due to limitations of the POSIX filesystem permission model, a user may
sudo to a new user (and gain its groups) or sudo to a new group
(preserving their current user), but not both simultaneously.

## Project Layout

This project is composed of three Rust crates:

* [`sudo_plugin-sys`](sudo_plugin-sys): raw Rust FFI bindings to the [`sudo_plugin(8)`][sudo_plugin_man] interface
* [`sudo_plugin`](sudo_plugin): a set of Rust structs and macros to simplify writing plugins
* [`sudo_pair`](sudo_pair): the implementation of this plugin

## Dependencies

Given the security-sensitive nature of this project, it is an explicit
goal to have a minimal set of dependencies. Currently, those are:

* [rust-lang/libc][libc]
* [rust-lang-nursery/rust-bindgen][bindgen]
* [rust-lang-nursery/error-chain][error-chain]

## Contributions

Contributions are welcome! This project should hopefully be small
(~500loc for the plugin itself, ~1kloc for the wrappers around writing
plugins) and well-documented enough for others to participate without
difficulty.

Pick a [TODO](sudo_pair/src/lib.rs) and get started!

## Bugs

Please report non-security issues on the GitHub tracker. Security issues
are covered by Square's [bug bounty program](BUG-BOUNTY.md).

## License

`sudo_pair` is  distributed under the terms of both the Apache License
(Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) for details.

[sudo_plugin_man]: https://www.sudo.ws/man/1.8.22/sudo_plugin.man.html
[libc]: https://github.com/rust-lang/libc
[bindgen]: https://github.com/rust-lang-nursery/rust-bindgen
[error-chain]: https://github.com/rust-lang-nursery/error-chain
