# Abstraction I/O avec trait Readable en Rust

##  Description

Ce projet illustre l’utilisation d’un trait en Rust pour abstraire la lecture de données depuis différentes sources.

Le trait `Readable` permet de définir une interface commune pour lire du contenu, indépendamment de la source.

##  Objectifs

* Comprendre les traits en Rust
* Implémenter plusieurs sources de données
* Utiliser le polymorphisme statique et dynamique

##  Fonctionnement

Le programme définit un trait `Readable` avec une méthode :

fn read(&mut self) -> Result<String, std::io::Error>

Trois implémentations sont fournies :

* `FileReader` : lit depuis un fichier
* `MemoryReader` : lit depuis une chaîne en mémoire
* `StdinReader` : lit depuis l'entrée standard (clavier)

##  Deux approches utilisées

### 1. Générique (statique)

fn read_static<T: Readable>(source: T)

* Le type est connu à la compilation
* Plus rapide

### 2. Dynamique

Vec<Box<dyn Readable>>

* Permet de manipuler plusieurs types différents
* Utilise le polymorphisme dynamique

##  Exemple de sortie

(statique)
nom=Alice

(dynamique)
nom=Alice

(dynamique)
Bonjour depuis la mémoire

##  Installation

git clone <url_du_repo>
cd parser_kv

##  Exécution

cargo run

##  Remarque

* La lecture depuis `stdin` peut bloquer le programme si utilisée sans précaution
* Aucune utilisation de `unwrap()` dans le code

##  Concepts utilisés

* Trait
* Implémentation de trait
* Génériques
* Polymorphisme dynamique (`Box<dyn Trait>`)
* Gestion des entrées/sorties (I/O)


