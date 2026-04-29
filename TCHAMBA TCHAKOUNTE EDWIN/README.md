# TP Rust — Module 2 (TP 5 à 8)

**Auteur :** TCHAMBA TCHAKOUNTE EDWIN
**Cours :** Programmation Système — GLO4ALT

## Description des TP

### TP5 — Parseur de fichier de configuration

Lecture d'un fichier `clé=valeur` avec gestion d'erreurs typée.

**Concepts abordés :**
- `Result<T, E>` et propagation d'erreurs avec l'opérateur `?`
- Définition d'un enum d'erreurs custom avec la crate `thiserror`
- Conversion automatique d'erreurs via `#[from]`
- Lecture bufferisée (`BufReader`, `lines()`)
- Pattern matching (`match`, `matches!`)
- Parsing typé (`str::parse::<T>()`)

**Cas d'erreur gérés :**
1. Fichier introuvable (`Io`)
2. Ligne sans `=` (`LigneMalFormee`)
3. Champ obligatoire absent (`ChampManquant`)
4. Valeur non parseable (`ValeurInvalide`)

### TP6 — Abstraction I/O via traits

Abstraction de sources et destinations de données via des traits, avec
démonstration des deux formes de polymorphisme.

**Concepts abordés :**
- Définition et implémentation de traits (`Source`, `Destination`)
- Méthodes par défaut surchargeables
- **Polymorphisme statique** via génériques (`fn f<S: Source>(...)`)
  → monomorphisation à la compilation, zéro coût à l'exécution
- **Polymorphisme dynamique** via objets-traits (`&mut dyn Trait`)
  → vtable, indirection à l'exécution, mais flexible
- Collection hétérogène de types via `Vec<Box<dyn Trait>>`

## Exécution

```bash
# Compiler les deux TPs
make build-all

# Lancer le TP5 (parseur de config.txt)
make run-tp5

# Lancer le TP6 (démos d'abstraction I/O)
make run-tp6

# Tests unitaires
make test-all

# Nettoyage
make clean-all
```

Ou directement avec cargo :

```bash
cd tpX-nom_du_tp
cargo run
cargo test
```

## Structure

```
TCHAMBA TCHAKOUNTE EDWIN/
├── Makefile
├── README.md
├── tp5-parseur_fichier/
│   ├── Cargo.toml
│   ├── config.txt           # fichier d'entrée d'exemple
│   └── src/main.rs
└── tp6-abstraction_generique/
    ├── Cargo.toml
    └── src/main.rs
```

## Dépendances externes

- **TP5** : `thiserror = "1.0"` (génération d'erreurs idiomatiques)
- **TP6** : aucune (uniquement `std`)
