use crate::error::ReadError;
use crate::readable::Readable;

/// ── Dispatch STATIQUE ──────────────────────────────────────────────────────
/// Le type concret est résolu à la compilation (monomorphisation).
/// Zéro surcoût à l'exécution, mais une seule implémentation par appel.
pub fn process_static<R: Readable>(source: &mut R) -> Result<(), ReadError> {
    let name = source.source_name().to_owned();
    let content = source.read_content()?;

    print_report(&name, &content);
    Ok(())
}

/// ── Dispatch DYNAMIQUE ─────────────────────────────────────────────────────
/// Travaille sur une collection hétérogène : chaque élément peut être
/// n'importe quelle implémentation de `Readable` emballée dans un `Box`.
pub fn process_all(sources: &mut Vec<Box<dyn Readable>>) {
    for source in sources.iter_mut() {
        let name = source.source_name().to_owned();

        match source.read_content() {
            Ok(content) => print_report(&name, &content),
            Err(e) => eprintln!("  [✗] {name} → {e}"),
        }
    }
}

// ── Helpers privés ─────────────────────────────────────────────────────────

fn print_report(name: &str, content: &str) {
    let lines = content.lines().count();
    let words = content.split_whitespace().count();
    let bytes = content.len();

    println!("┌─ Source : {name}");
    println!("│  {lines} ligne(s)  ·  {words} mot(s)  ·  {bytes} octet(s)");
    println!("│");
    for line in content.lines().take(5) {
        println!("│  {line}");
    }
    if lines > 5 {
        println!("│  … ({} lignes supplémentaires)", lines - 5);
    }
    println!("└─────────────────────────────────");
}
