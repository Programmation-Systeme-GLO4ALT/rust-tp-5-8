#[derive(Debug, Clone, PartialEq)]
enum EtatProcessus {
    Pret,
    EnExecution { cpu_id: u8 },
    Bloque { raison: String },
    Termine { code_retour: i32 },
    Zombie,
}

#[derive(Debug, Clone)]
enum Priorite {
    TresFaible,
    Faible,
    Normale,
    Haute,
    TresHaute,
    TempsReel(u8),
}

#[derive(Debug)]
struct Processus {
    pid: u32,
    nom: String,
    etat: EtatProcessus,
    priorite: Priorite,
    memoire_ko: u64,
    pid_parent: Option<u32>,
}

#[derive(Debug)]
struct GestionnaireProcessus {
    processus: Vec<Processus>,
    prochain_pid: u32,
}

impl GestionnaireProcessus {
    fn nouveau() -> Self {
        Self {
            processus: Vec::new(),
            prochain_pid: 1,
        }
    }

    fn creer_processus(
        &mut self,
        nom: String,
        priorite: Priorite,
        memoire_ko: u64,
        pid_parent: Option<u32>,
    ) -> u32 {
        let pid = self.prochain_pid;
        self.prochain_pid += 1;

        let processus = Processus {
            pid,
            nom,
            etat: EtatProcessus::Pret,
            priorite,
            memoire_ko,
            pid_parent,
        };

        self.processus.push(processus);
        pid
    }

    fn trouver(&self, pid: u32) -> Option<&Processus> {
        self.processus.iter().find(|processus| processus.pid == pid)
    }

    fn changer_etat(
        &mut self,
        pid: u32,
        nouvel_etat: EtatProcessus,
    ) -> Result<(), String> {
        match self.processus.iter_mut().find(|processus| processus.pid == pid) {
            Some(processus) => {
                processus.etat = nouvel_etat;
                Ok(())
            }
            None => Err(format!("PID introuvable : {}", pid)),
        }
    }

    fn memoire_totale_utilisee(&self) -> u64 {
        self.processus.iter().map(|processus| processus.memoire_ko).sum()
    }

    fn processus_par_etat(&self, etat: &EtatProcessus) -> Vec<&Processus> {
        self.processus
            .iter()
            .filter(|processus| &processus.etat == etat)
            .collect()
    }

    fn tuer_processus(&mut self, pid: u32) -> Result<i32, String> {
        match self.processus.iter_mut().find(|processus| processus.pid == pid) {
            Some(processus) => {
                let code_retour = 0;
                processus.etat = EtatProcessus::Termine { code_retour };
                Ok(code_retour)
            }
            None => Err(format!("Impossible de tuer le PID {} : introuvable", pid)),
        }
    }

    fn afficher_resume(&self) {
        println!("=== Resume des processus ===");
        println!("Nombre total de processus : {}", self.processus.len());
        println!("Memoire totale (Ko)       : {}", self.memoire_totale_utilisee());

        for processus in &self.processus {
            println!(
                "PID={} | nom={} | etat={} | priorite={} | memoire={} Ko | parent={}",
                processus.pid,
                processus.nom,
                decrire_etat(&processus.etat),
                decrire_priorite(&processus.priorite),
                processus.memoire_ko,
                processus
                    .pid_parent
                    .map(|pid| pid.to_string())
                    .unwrap_or_else(|| String::from("Aucun"))
            );
        }
    }
}

fn decrire_etat(etat: &EtatProcessus) -> String {
    match etat {
        EtatProcessus::Pret => String::from("Pret"),
        EtatProcessus::EnExecution { cpu_id } => format!("EnExecution(cpu={})", cpu_id),
        EtatProcessus::Bloque { raison } => format!("Bloque({})", raison),
        EtatProcessus::Termine { code_retour } => format!("Termine(code={})", code_retour),
        EtatProcessus::Zombie => String::from("Zombie"),
    }
}

fn decrire_priorite(priorite: &Priorite) -> String {
    match priorite {
        Priorite::TresFaible => String::from("TresFaible"),
        Priorite::Faible => String::from("Faible"),
        Priorite::Normale => String::from("Normale"),
        Priorite::Haute => String::from("Haute"),
        Priorite::TresHaute => String::from("TresHaute"),
        Priorite::TempsReel(niveau) => format!("TempsReel({})", niveau),
    }
}

fn main() {
    let mut gp = GestionnaireProcessus::nouveau();

    let init = gp.creer_processus(String::from("init"), Priorite::Haute, 1024, None);

    let bash = gp.creer_processus(String::from("bash"), Priorite::Normale, 4096, Some(init));

    let nginx = gp.creer_processus(
        String::from("nginx"),
        Priorite::TempsReel(20),
        8192,
        Some(init),
    );

    gp.changer_etat(bash, EtatProcessus::EnExecution { cpu_id: 0 })
        .unwrap();
    gp.changer_etat(
        nginx,
        EtatProcessus::Bloque {
            raison: String::from("Attente E/S disque"),
        },
    )
    .unwrap();

    gp.afficher_resume();

    println!();
    match gp.tuer_processus(bash) {
        Ok(code) => println!("bash termine avec code {}", code),
        Err(e) => eprintln!("Erreur : {}", e),
    }

    println!();
    let processus_prets = gp.processus_par_etat(&EtatProcessus::Pret);
    println!("Processus en etat Pret : {}", processus_prets.len());

    if let Some(proc_init) = gp.trouver(init) {
        println!("Processus init retrouve : {:?}", proc_init);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_processus() {
        let mut gp = GestionnaireProcessus::nouveau();
        let pid = gp.creer_processus(String::from("init"), Priorite::Haute, 1024, None);

        assert_eq!(pid, 1);
        assert!(gp.trouver(pid).is_some());
    }

    #[test]
    fn changement_etat() {
        let mut gp = GestionnaireProcessus::nouveau();
        let pid = gp.creer_processus(String::from("bash"), Priorite::Normale, 2048, None);

        gp.changer_etat(pid, EtatProcessus::EnExecution { cpu_id: 1 })
            .unwrap();

        let proc = gp.trouver(pid).unwrap();
        assert_eq!(proc.etat, EtatProcessus::EnExecution { cpu_id: 1 });
    }

    #[test]
    fn memoire_totale() {
        let mut gp = GestionnaireProcessus::nouveau();
        gp.creer_processus(String::from("p1"), Priorite::Faible, 100, None);
        gp.creer_processus(String::from("p2"), Priorite::Normale, 300, None);

        assert_eq!(gp.memoire_totale_utilisee(), 400);
    }

    #[test]
    fn tuer_un_processus() {
        let mut gp = GestionnaireProcessus::nouveau();
        let pid = gp.creer_processus(String::from("bash"), Priorite::Normale, 512, None);

        let code = gp.tuer_processus(pid).unwrap();
        assert_eq!(code, 0);

        let proc = gp.trouver(pid).unwrap();
        assert_eq!(proc.etat, EtatProcessus::Termine { code_retour: 0 });
    }
}
