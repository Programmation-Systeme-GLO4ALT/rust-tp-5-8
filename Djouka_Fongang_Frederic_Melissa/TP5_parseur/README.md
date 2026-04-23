# Parseur clé=valeur en Rust

##  Description

Ce projet est un parseur simple écrit en Rust qui lit un fichier texte contenant des paires clé=valeur.

Chaque ligne du fichier doit respecter le format :
clé=valeur

##  Exemple de fichier d'entrée (data.txt)

nom=Alice
age=25
ville=Yaounde

##  Fonctionnement

* Le programme lit le fichier `data.txt`
* Chaque ligne est analysée et séparée avec le caractère `=`
* Les données sont stockées dans une `HashMap`
* Les résultats sont affichés dans la console

##  Gestion des erreurs

* Fichier introuvable
* Ligne mal formatée (sans `=`)
* Gestion propre des erreurs avec `thiserror`
* Aucun usage de `unwrap()`

##  Installation

1. Installer Rust sur votre machine
2. Cloner le projet :
   git clone <url_du_repo>
   cd parser_kv

##  Exécution

cargo run

##  Résultat attendu

nom = Alice
age = 25
ville = Yaounde

##  Concepts utilisés

* HashMap
* Lecture de fichier
* Gestion d'erreurs avec Result
* Utilisation de `thiserror`
* Traitement de chaînes de caractères


