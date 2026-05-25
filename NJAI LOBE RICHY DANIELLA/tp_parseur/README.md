# Parseur clé=valeur en Rust

##  Description

Ce projet est un parseur simple écrit en Rust qui lit un fichier texte contenant des paires clé=valeur.

Chaque ligne du fichier doit respecter le format :
clé=valeur


## Gestion des erreurs

* Fichier introuvable
* Ligne mal formatée (sans `=`)
* Gestion propre des erreurs avec `thiserror`
* Aucun usage de `unwrap()`


### Exécution

cargo run

####  Résultat attendu

Clé: ecole | Valeur: Polytechnique
Clé: niveau | Valeur: 4eme annee
Clé: ville | Valeur: Douala

##  Concepts utilisés

1° HashMap
2° Lecture de fichier
3°  Utilisation de `thiserror`



