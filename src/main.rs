use std::env;
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct ResultadoEscaneo {
    puerto: u16,
    abierto: bool,
    banner: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Error: Faltan argumentos.");
        println!("Uso: cargo run -- <IP_OBJETIVO> <PUERTOS_SEPARADOS_POR_COMAS>");
        println!("Ejemplo: cargo run -- 127.0.0.1 22,80,443,8080");
        return;
    }

    let ip_objetivo = &args[1];

    let puertos: Vec<u16> = args[2]
        .split(',')
        .filter_map(|p| p.trim().parse::<u16>().ok())
        .collect();

    if puertos.is_empty() {
        println!("Error: No se especificaron puertos válidos.");
        return;
    }

    println!("Iniciando escaneo dinámico en: {}", ip_objetivo);
    println!("Puertos a analizar: {:?}", puertos);
    println!("--------------------------------------------------");

    let (tx, rx) = mpsc::channel();
    let mut total_hilos = 0;

    for puerto in puertos {
        let tx_hilo = tx.clone();
        let ip = ip_objetivo.clone();
        total_hilos += 1;

        thread::spawn(move || {
            let direccion = format!("{}:{}", ip, puerto);
            let timeout = Duration::from_millis(800);

            match TcpStream::connect_timeout(&direccion.parse().unwrap(), timeout) {
                Ok(mut stream) => {
                    let mut buffer = [0; 64];
                    stream
                        .set_read_timeout(Some(Duration::from_millis(500)))
                        .unwrap();

                    let banner = match stream.read(&mut buffer) {
                        Ok(bytes_leidos) if bytes_leidos > 0 => {
                            String::from_utf8_lossy(&buffer[..bytes_leidos])
                                .trim()
                                .replace("\n", " ")
                                .replace("\r", "")
                        }
                        _ => "No expone banner (Requiere petición activa)".to_string(),
                    };

                    tx_hilo
                        .send(ResultadoEscaneo {
                            puerto,
                            abierto: true,
                            banner,
                        })
                        .unwrap();
                }
                Err(_) => {
                    tx_hilo
                        .send(ResultadoEscaneo {
                            puerto,
                            abierto: false,
                            banner: String::new(),
                        })
                        .unwrap();
                }
            }
        });
    }

    drop(tx);

    for resultado in rx {
        if resultado.abierto {
            println!(
                "Puerto {:<5} ... [ ABIERTO ] -> Banner: \"{}\"",
                resultado.puerto, resultado.banner
            );
        } else {
            println!("Puerto {:<5} ... [ CERRADO ]", resultado.puerto);
        }
    }

    println!("---------------------------------------------------");
    println!("Escaneo finalizado. Se analizaron {} puertos.", total_hilos);
}
