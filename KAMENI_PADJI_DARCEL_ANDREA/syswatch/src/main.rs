use chrono::Local;
use std::fmt::{self, Display, Formatter};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::System;

#[derive(Debug, Clone)]
struct CpuInfo {
    usage: f32,
}

#[derive(Debug, Clone)]
struct MemInfo {
    total: u64,
    used: u64,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: i32,
    name: String,
    cpu: f32,
    memory: u64,
}

#[derive(Debug, Clone)]
struct SystemSnapshot {
    cpu: CpuInfo,
    memory: MemInfo,
    processes: Vec<ProcessInfo>,
}

impl Display for CpuInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Usage CPU global : {:.2}%", self.usage)
    }
}

impl Display for MemInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mémoire : {}/{} Mo utilisés",
            self.used / 1024,
            self.total / 1024
        )
    }
}

impl Display for ProcessInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>6} │ {:<25} │ {:>6.2}% │ {:>8} Ko",
            self.pid,
            self.name,
            self.cpu,
            self.memory
        )
    }
}

impl Display for SystemSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== SysWatch Snapshot ===")?;
        writeln!(f, "{}\n{}\n", self.cpu, self.memory)?;
        writeln!(f, "Top 5 processus CPU :")?;
        writeln!(f, "PID    │ Nom                      │ CPU    │ Mémoire")?;
        writeln!(f, "------+--------------------------+--------+----------")?;
        for process in &self.processes {
            writeln!(f, "{}", process)?;
        }
        Ok(())
    }
}

fn collect_snapshot() -> io::Result<SystemSnapshot> {
    let mut system = System::new_all();
    system.refresh_all();

    let cpu = CpuInfo {
        usage: system.global_cpu_info().cpu_usage(),
    };

    let memory = MemInfo {
        total: system.total_memory(),
        used: system.used_memory(),
    };

    let mut processes: Vec<_> = system
        .processes()
        .values()
        .map(|process| ProcessInfo {
            pid: process.pid().as_u32() as i32,
            name: process.name().to_string(),
            cpu: process.cpu_usage(),
            memory: process.memory(),
        })
        .collect();

    processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(5);

    Ok(SystemSnapshot {
        cpu,
        memory,
        processes,
    })
}

fn format_response(snapshot: &SystemSnapshot, command: &str) -> String {
    match command.trim().to_lowercase().as_str() {
        "cpu" => format!("{}\n", snapshot.cpu),
        "mem" => format!("{}\n", snapshot.memory),
        "ps" => {
            let mut lines = vec![
                "PID    │ Nom                      │ CPU    │ Mémoire".to_string(),
                "------+--------------------------+--------+----------".to_string(),
            ];
            for proc in &snapshot.processes {
                lines.push(proc.to_string());
            }
            lines.join("\n") + "\n"
        }
        "all" => format!("{}\n", snapshot),
        "help" => "Commandes disponibles : cpu, mem, ps, all, help, quit\n".to_string(),
        "quit" => "quit\n".to_string(),
        _ => "Commande inconnue. Tape help pour la liste.\n".to_string(),
    }
}

fn log_message(entry: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("syswatch.log")?;
    writeln!(file, "{}", entry)
}

fn handle_client(mut stream: TcpStream, shared_snapshot: Arc<Mutex<SystemSnapshot>>) -> io::Result<()> {
    let peer = stream.peer_addr()?;
    let connect_time = Local::now();
    log_message(&format!(
        "[{}] Connexion depuis {}",
        connect_time.format("%Y-%m-%d %H:%M:%S"),
        peer
    ))?;

    writeln!(stream, "Bienvenue sur SysWatch. Tape help pour commencer.")?;
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let command = line?;
        log_message(&format!(
            "[{}] {} -> {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            peer,
            command
        ))?;

        let snapshot = match shared_snapshot.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => break,
        };

        let response = format_response(&snapshot, &command);
        if command.trim().eq_ignore_ascii_case("quit") {
            writeln!(stream, "Au revoir !")?;
            break;
        }
        writeln!(stream, "{}", response)?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let initial_snapshot = collect_snapshot()?;
    let shared_snapshot = Arc::new(Mutex::new(initial_snapshot));
    let updater_snapshot = Arc::clone(&shared_snapshot);

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        if let Ok(new_snapshot) = collect_snapshot() {
            if let Ok(mut guard) = updater_snapshot.lock() {
                *guard = new_snapshot;
            }
        }
    });

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("SysWatch TCP server démarré sur 127.0.0.1:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let snapshot_clone = Arc::clone(&shared_snapshot);
                thread::spawn(move || {
                    if let Err(err) = handle_client(stream, snapshot_clone) {
                        eprintln!("Erreur client : {}", err);
                    }
                });
            }
            Err(err) => eprintln!("Erreur accept : {}", err),
        }
    }

    Ok(())
}
