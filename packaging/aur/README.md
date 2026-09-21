# Publishing forkstify

Two channels, in this order: a **GitHub release** that carries the binary,
then an **AUR package** that puts it on Arch machines.

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

Then you push to the AUR, which is one git repository per package:

```bash
git clone ssh://aur@aur.archlinux.org/forkstify-bin.git /tmp/aur-forkstify
cp PKGBUILD .SRCINFO /tmp/aur-forkstify/
cd /tmp/aur-forkstify && git add -A
git commit -m "forkstify-bin 0.2.0" && git push
```

The first publication needs an AUR account with an SSH key declared, and the
repository is created on the first `git push`.

**`sha256sums=('SKIP')`** is temporary: it lets `makepkg` work before a
release exists. `updpkgsums` must have been run before any publication — the
AUR does not accept `SKIP` for a downloaded archive.

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
