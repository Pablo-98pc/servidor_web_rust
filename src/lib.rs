use std::thread;
use std::sync::{Arc, Mutex, mpsc};
// Un 'Job' es una función que está en el Heap (Box), 
// que se puede enviar entre hilos (Send) y que no tiene referencias colgantes ('static).
type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Option<Job>>
}

struct Worker {
    id:usize,
    thread: Option<thread::JoinHandle<()>>
}



impl Worker {
    pub fn new (id:usize,reciever : Arc<Mutex<mpsc::Receiver<Option<Job>>>>) -> Worker {
            let thread = thread::spawn(move ||{
                loop{
                    let message = reciever.lock().unwrap().recv().unwrap();
                    match message {
                        Some(job) => {
                            println!("Soy el Worker {} y he recibido el trabajo.",id);
                            job();
                        }
                        None => {
                            println!("Worker {} desconectando...",id);
                            break;
                        }
                    }
                  
                }
            });
        Worker {
            id,
            thread:Some(thread)
        }
    }


}

impl ThreadPool {
    /// Crea una nueva ThreadPool.
    ///
    /// El tamaño es el número de hilos en la piscina.
    ///
    /// # Pánico
    ///
    /// La función hará pánico si el tamaño es 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender,reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));
        let mut workers : Vec<Worker> = Vec::with_capacity(size);
        for n in 0.. size {
            let worker = Worker::new(n,Arc::clone(&reciever));
            workers.push(worker);
        }
        ThreadPool {
            workers,
            sender
        }
    }

    /// Ejecuta la función enviada en un hilo disponible.
    ///
    /// F es el tipo de la closure (función anónima).
    /// Debe cumplir tres requisitos (Traits):
    /// 1. FnOnce: Se ejecuta una vez.
    /// 2. Send: Se puede enviar de un hilo a otro.
    /// 3. 'static: Vive lo suficiente (no tiene referencias prestadas colgando).
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f); // Empaquetamos la función
        self.sender.send(Some(job)).unwrap(); // La enviamos por el tubo
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Enviando señal de terminación a todos los workers.");

        // BUCLE 1: Mandar a dormir (Enviar None)
        // Enviamos un 'None' por cada worker que tenemos.
        for _ in &self.workers {
            self.sender.send(None).unwrap();
        }

        println!("Cerrando todos los workers.");

        // BUCLE 2: Esperar a que terminen (Join)
        for worker in &mut self.workers {
            println!("Cerrando worker {}", worker.id);

            // .take() saca el valor del Option y deja un None en su lugar.
            // Nos permite llevarnos el hilo aunque solo tengamos &mut worker.
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}