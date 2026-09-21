# Publier forkstify

Deux canaux, dans cet ordre : une **release GitHub** qui porte le binaire,
puis un **paquet AUR** qui le pose sur les machines Arch.

## 1. La release

Le workflow `.github/workflows/release.yml` se déclenche sur un tag `v*`.
Il vérifie que le tag, `Cargo.toml` et `manifest.json` disent la même
version, lance les tests, compile dans `rust:1-slim` — la même image que
le `Dockerfile` de la racine, donc ce que la CI publie est ce que
`bin/build` produit — puis attache à la release :

- `forkstify-<version>-x86_64-linux.tar.gz` (le binaire allégé + `LICENSE`)
- `SHA256SUMS`

```bash
# la version doit être la même aux trois endroits
$EDITOR Cargo.toml manifest.json      # version = "0.2.0"
git commit -am "Version 0.2.0" && git push
git tag v0.2.0 && git push origin v0.2.0
```

**Le plancher de glibc** est celui de Debian 12 (2.36). Le binaire tourne
sur Arch et sur tout ce qui est plus jeune ; ailleurs, on compile depuis
les sources.

## 2. Le paquet AUR

`forkstify-bin/` est la copie de travail du dépôt AUR. Une fois la release
publiée, l'empreinte se remplit toute seule :

```bash
cd packaging/aur/forkstify-bin
$EDITOR PKGBUILD                      # pkgver=0.2.0, pkgrel=1
updpkgsums                            # remplit sha256sums depuis l'archive
makepkg --printsrcinfo > .SRCINFO
makepkg -si                           # essai local avant de publier
```

Puis on pousse vers l'AUR, qui est un dépôt git par paquet :

```bash
git clone ssh://aur@aur.archlinux.org/forkstify-bin.git /tmp/aur-forkstify
cp PKGBUILD .SRCINFO /tmp/aur-forkstify/
cd /tmp/aur-forkstify && git add -A
git commit -m "forkstify-bin 0.2.0" && git push
```

La première publication demande un compte AUR avec une clé SSH déclarée,
et le dépôt se crée au premier `git push`.

**`sha256sums=('SKIP')`** est un provisoire : il laisse `makepkg` marcher
avant qu'une release existe. Il faut que `updpkgsums` soit passé avant
toute publication — l'AUR n'accepte pas `SKIP` pour une archive
téléchargée.

## Ce que le paquet installe, et ce qu'il n'installe pas

Il pose `/usr/bin/forkstify`. Il **ne pose pas** le widget de barre : le
dépôt est le plugin Omarchy ([0021](../../docs/decisions/0021-le-depot-est-le-plugin-omarchy.md)),
et il s'ajoute avec `omarchy plugin add`. Les deux se complètent bien : le
widget teste `command -v forkstify`, donc sur une machine où le paquet est
installé il propose « Launch » tout de suite, sans passer par « Install ».

## Le troisième canal, pour mémoire

Un `forkstify` (sans `-bin`) qui compile depuis les sources rendrait
service à qui se méfie des binaires, au prix d'un `makedepends` sur rust,
`alsa-lib` et `gcc`. À ouvrir quand `-bin` aura tourné quelques versions.
