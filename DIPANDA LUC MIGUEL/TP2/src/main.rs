
fn main() {
   
    let mut ma_tache = (String::from("Apprendre l'Ownership"), 5, false);

    println!("État initial : {:?}", ma_tache);

    
    ma_tache = marquer_terminee(ma_tache);

    println!("État final : {:?}", ma_tache);
}


fn marquer_terminee(tache: (String, u8, bool)) -> (String, u8, bool) {
    let (nom, priorite, _) = tache; // On décompose le tuple
    (nom, priorite, true)           // On en crée un nouveau marqué comme 'true'
}