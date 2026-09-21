# Publishing forkstify

Two channels, in this order: a **GitHub release** that carries the binary,
then an **AUR package** that puts it on Arch machines.

**State on 21/09/2026.** The release channel is live: `v0.1.0` is published,
with `forkstify-0.1.0-x86_64-linux.tar.gz` (17.4 MB) and `SHA256SUMS`. The
AUR channel waits: **account registration is temporarily closed** while the
AUR deals with a wave of automated sign-ups. Nothing is blocked by it — see
*Publishing without an AUR account* below.

## 1. The release

The `.github/workflows/release.yml` workflow fires on a `v*` tag. It checks
that the tag, `Cargo.toml` and `manifest.json` all say the same version,
runs the tests, compiles in `rust:1-slim` — the same image as the root
`Dockerfile`, so what CI publishes is what `bin/build` produces — then
attaches to the release:

- `forkstify-<version>-x86_64-linux.tar.gz` (the stripped binary +
  `LICENSE`)
- `SHA256SUMS`

```bash
# the version must be the same in all three places
$EDITOR Cargo.toml manifest.json      # version = "0.2.0"
git commit -am "Version 0.2.0" && git push
git tag v0.2.0 && git push origin v0.2.0
```

**The glibc floor** is Debian 12's (2.36). The binary runs on Arch and on
anything newer; elsewhere, you build from source.

## 2. The AUR package

`forkstify-bin/` is the working copy of the AUR repository. Once the release
is published, the digest fills itself in:

```bash
cd packaging/aur/forkstify-bin
$EDITOR PKGBUILD                      # pkgver=0.2.0, pkgrel=1
updpkgsums                            # fills sha256sums from the archive
makepkg --printsrcinfo > .SRCINFO
makepkg -si                           # a local try before publishing
```

The package for `v0.1.0` is ready and was built for real on 21/09/2026:
one clean `forkstify-bin-0.1.0-1-x86_64.pkg.tar.zst` of 16 MB, holding
`/usr/bin/forkstify` and its licence, and nothing else.

Then you push to the AUR, which is one git repository per package:

```bash
git clone ssh://aur@aur.archlinux.org/forkstify-bin.git /tmp/aur-forkstify
cp PKGBUILD .SRCINFO /tmp/aur-forkstify/
cd /tmp/aur-forkstify && git add -A
git commit -m "forkstify-bin 0.2.0" && git push
```

The first publication needs an AUR account with an SSH key declared, and the
repository is created on the first `git push`.

`sha256sums` is filled in and must stay so: the AUR does not accept `SKIP`
for a downloaded archive. `options=('!strip' '!debug')` is there because the
binary arrives already stripped from the release — without it, `makepkg`
carves out an empty debug package.

## Publishing without an AUR account

AUR registration reopening is out of our hands, and there is no manual
queue: the announcement comes on the Arch news feed and the `aur-general`
list. **Do not script retries against the sign-up page.** Meanwhile, two
roads are open and neither needs an account.

**The release, which is the main road anyway.** Omarchy users add the
plugin and `omarchy/install.sh` fetches the published binary, checks its
digest, and never needs Docker. This is what most people will do; the AUR
only serves Arch users who are not on Omarchy.

**`makepkg` straight from a clone.** Any Arch user can install the very
package the AUR would serve, in one command, from this repository:

```bash
git clone https://github.com/aropixel/forkstify.git
cd forkstify/packaging/aur/forkstify-bin && makepkg -si
```

Worth a line in the `README.md` so people find it.

**A personal pacman repository**, if `pacman -S forkstify` matters before
the AUR reopens: build the package in CI, run `repo-add` on it, and host the
`.pkg.tar.zst` and the `.db` on the release or on GitHub Pages; users add it
to `pacman.conf`. It is real work, and it duplicates what the AUR will do
for free — worth it only if the wait drags on.

## What the package installs, and what it does not

It puts `/usr/bin/forkstify` in place. It does **not** put the bar widget
there: the repository is the Omarchy plugin
([0021](../../docs/decisions/0021-the-repository-is-the-omarchy-plugin.md)),
and it is added with `omarchy plugin add`. The two complement each other
well: the widget tests `command -v forkstify`, so on a machine where the
package is installed it offers "Launch" right away, with no "Install" step.

## The third channel, for the record

A `forkstify` (with no `-bin`) building from source would serve whoever
distrusts binaries, at the cost of a `makedepends` on rust, `alsa-lib` and
`gcc`. To be opened once `-bin` has run for a few versions.
