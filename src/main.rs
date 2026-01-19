use std::net::TcpListener;
use std::io::prelude::*; // Importa Read y Write
use std::fs;
use std::time::Duration;
use std::thread;
use servidor_web::ThreadPool;

fn handle_connection (mut stream: std::net::TcpStream){
    let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let peticion = String::from_utf8_lossy(&buffer[..]);

       // Código Idiomático
            let (status_line, filename) = if peticion.starts_with("GET / HTTP/1.1") {
                ("HTTP/1.1 200 OK", "hello.html")
            } else if peticion.starts_with("GET /sleep HTTP/1.1") {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "hello.html")
            } else {
                ("HTTP/1.1 404 NOT FOUND", "404.html")
            };
        // ¡Mucho más limpio! Sin variables mutables ni declaraciones sueltas.
           
        // 2. Preparamos la respuesta
        
        // Leemos el archivo HTML a un String
        let contents = fs::read_to_string(filename).unwrap();
        let length = contents.len();

        // 3. Formateamos el mensaje HTTP completo
        // \r\n es el salto de línea estándar en HTTP
        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );

        // 4. Enviamos los bytes por el cable TCP
        stream.write_all(response.as_bytes()).unwrap();
        
        println!("Respuesta enviada.");
    
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Servidor listo en 127.0.0.1:7878");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let  stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream)
        });

    }
}