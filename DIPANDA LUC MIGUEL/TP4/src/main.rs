

#[derive(Debug, PartialEq)]
enum Etat {
    Pret,
    EnExecution { cpu_id: u8 },
    Bloque(String), 
    Termine(i32),  
}

#[derive(Debug)]
struct Processus {
    pid: u32,
    nom: String,
    etat: Etat,
    parent_pid: Option<u32>, 
}

struct Gestionnaire {
    liste: Vec<Processus>,
    dernier_pid: u32,
}

impl Gestionnaire {
    fn nouveau() -> Self {
        Gestionnaire {
            liste: Vec::new(),
            dernier_pid: 0,
        }
    }

    
    fn spawn(&mut self, nom: &str, parent: Option<u32>) -> u32 {
        self.dernier_pid += 1;
        let nouveau_p = Processus {
            pid: self.dernier_pid,
            nom: nom.to_string(),
            etat: Etat::Pret,
            parent_pid: parent,
        };
        self.liste.push(nouveau_p);
        self.dernier_pid
    }

    fn transitionner(&mut self, pid: u32, nouvel_etat: Etat) -> Result<(), String> {
        for p in &mut self.liste {
            if p.pid == pid {
                p.etat = nouvel_etat;
                return Ok(());
            }
        }
        Err(format!("Erreur : Le processus avec le PID {} n'existe pas.", pid))
    }

    fn afficher_tout(&self) {
        println!("\n--- Liste des Processus ---");
        for p in &self.liste {
            println!("[PID {}] Name: {} | Etat: {:?}", p.pid, p.nom, p.etat);
        }
    }
}

fn main() {
    let mut systeme = Gestionnaire::nouveau();

   
    let pid_init = systeme.spawn("init", None);

    let pid_bash = systeme.spawn("bash", Some(pid_init));

   
    let _ = systeme.transitionner(pid_init, Etat::EnExecution { cpu_id: 0 });
    let _ = systeme.transitionner(pid_bash, Etat::Bloque(String::from("Attente clavier")));

    systeme.afficher_tout();

   
    match systeme.transitionner(99, Etat::Pret) {
        Ok(_) => println!("Succès !"),
        Err(e) => println!("\nNotification système : {}", e),
    }
}