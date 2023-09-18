# cal7tor • *cal*endar P*7* extrac*tor*

> !! Fork de [cal8tor](https://git.mylloon.fr/Anri/cal8tor) !!
>
> !! [En cours de dev](https://git.mylloon.fr/Anri/cal7tor/pulls/1) -> ne fonctionne pas !!

Extracteur d'emploi du temps pour les masters d'informatique de Paris Cité (Diderot)

[![dependency status](https://deps.rs/repo/gitea/git.mylloon.fr/Anri/cal7tor/status.svg)](https://deps.rs/repo/gitea/git.mylloon.fr/Anri/cal7tor)

## Installation

### Manuellement

Cf. [Compilation et installation](#compilation-et-installation).

## Lancer

Pour afficher la page d'aide

```
$ cal7tor --help
```

## Voir le calendrier dans le terminal

Pour les LP par exemple, lance :

```bash
$ cal7tor lp
```

> Le rendu peut parfois être difficile à lire, n'hésites pas à utiliser l'option
> `-c` (ou `--cl`) pour ajuster la longueur des cellules du planning.

## Exporter le calendrier au format `.ics`

Pour les LP par exemple, lance :

```bash
$ cal8tor LP --export calendar.ics
```

> Le fichier comprend le fuseau horaire pour `Europe/Paris` et est
> conforme à [cet outil de validation](https://icalendar.org/validator.html).

## Compilation et installation

Vous aurez besoin de Rust pour compiler le programme.

<details><summary>Vous avez aussi besoin d'<code>OpenSSL</code>.</summary>

- Ubuntu: `sudo apt install libssl-dev`
- Fedora: `dnf install openssl-devel`
</details>

1. Clone le dépôt et s'y rendre

```bash
$ git clone https://git.mylloon.fr/Anri/cal7tor.git && cd cal7tor
```

2. Compiler et installer l'application

```bash
$ cargo install --path .
```

3. Tu peux maintenant supprimer le dossier `cal7tor` !

---

N'hésite pas à faire un PR pour améliorer le projet !
