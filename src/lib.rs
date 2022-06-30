#[cfg(test)]
mod tests {
    use crate::{main, AgvSystem, Request};
    fn get_num(msg: &str) -> i32
    {
        println!("{}", msg);
        let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
        input.trim().parse().expect("Please input a number")
    }

    use std::io;
    use std::thread;
    use std::sync::mpsc;
    use std::time::Duration;
    #[test]
    fn manual() {

        let (agv1_sender, agv1_receiver) = mpsc::channel();
        let (agv2_sender, agv2_receiver) = mpsc::channel();
        let agv1 = AgvSystem {request_input: agv1_receiver};
        let agv2 = AgvSystem {request_input: agv2_receiver};

        println!("===== AGV System Controller =====");
        thread::spawn(|| {main(agv1, agv2)});

        let mut exit = false;
        while !exit {
            let number = get_num("select agv system 1-2, 0 to exit, 3 for both");
            match number {
                0 => exit = true,
                1 => {
                    let number = get_num("select action, 1 to request access, 2 to release access");
                    match number {
                        1 => agv1_sender.send(Request {mode: true, id: get_num("enter ID") as u32}).expect("Could not send input"),
                        2 => agv1_sender.send(Request {mode: false, id: get_num("enter ID") as u32}).expect("Could not send input"),
                        _ => {}
                    }
                },
                2 => {
                    let number = get_num("select action, 1 to request access, 2 to release access");
                    match number {
                        1 => agv2_sender.send(Request {mode: true, id: get_num("enter ID") as u32}).expect("Could not send input"),
                        2 => agv2_sender.send(Request {mode: false, id: get_num("enter ID") as u32}).expect("Could not send input"),
                        _ => {}
                    }
                },
                3 => {
                    let number = get_num("select action, 1 to request access, 2 to release access");
                    match number {
                        1 => {
                            let num = get_num("enter ID") as u32;
                            agv1_sender.send(Request {mode: true, id: num}).expect("Could not send input");
                            agv2_sender.send(Request {mode: true, id: num}).expect("Could not send input");
                        },
                        2 => {
                            let num = get_num("enter ID") as u32;
                            agv1_sender.send(Request {mode: false, id: num}).expect("Could not send input");
                            agv2_sender.send(Request {mode: false, id: num}).expect("Could not send input");
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
            thread::sleep(Duration::from_millis(100)) // Wait for threads to finish
        }
    }
}

use std::thread;
use std::sync::{Arc, Mutex, mpsc};

struct Request {
    mode: bool, // true = request, false = release
    id: u32
}

// Simulation of a AGV system
struct AgvSystem {
    request_input: mpsc::Receiver<Request>,
}

impl AgvSystem {
    fn request (&self, sender: mpsc::Sender<Request>) { // Listen for input from user
        loop {
            sender.send(
                self.request_input
                    .recv()
                    .expect("Error when listening for input")
            ).expect("Error while sending request");
        }
    }
}


fn main(agv1: AgvSystem, agv2: AgvSystem) {
    // Create a shared pointer to a mutex that contains the list of requests
    let requests = Arc::new(Mutex::new(vec![]));

    // Create the central listener thread
    let central_requests = Arc::clone(&requests); // Create a new pointer to the mutex
    let (request_notifier, request_notifyee) = mpsc::channel(); // Create the request notification channel
    thread::spawn(move || { // move in order to access outside variables
        let (s1, receiver) = mpsc::channel(); // Create request listener channel
        let s2 = s1.clone(); // Create an additional sender

        thread::spawn(move || {
            agv1.request(s1); // Request input
        });
        thread::spawn(move || {
            agv2.request(s2);
        });

        
        loop {
            let request = receiver.recv().expect("Listener thread finished!"); // Listen on the request channel
            let mut requests = central_requests.lock().expect("Failed to lock central_requests"); // Lock the mutex so no other thread modifies it
            requests.push(request); // Push the request
            request_notifier.send(()).expect("Error when sending notification"); // Notify the request handler that there are new requests
        }
    });

    let mut occupied: Vec<u32> = vec![]; // List with occupied IDs

    // Request handler
    loop {
        request_notifyee.recv().expect("central thread finished!"); // Wait for requests

        let mut requests = requests.lock().expect("Error when locking requests"); // Lock the mutex so the central listener doesn't modify it 

        // Go over requests and handle them
        for request in requests.iter() {
            if request.mode {
                if !occupied.contains(&request.id) {
                    occupied.push(request.id);
                    println!("Request {} accepted!", request.id)
                }
                else {
                    println!("Request {} denied!", request.id)
                }
            }
            else {
                if occupied.contains(&request.id) {
                    let index = occupied.binary_search(&request.id).expect("Binary search failed");
                    occupied.remove(index);
                    println!("Release {} completed!", request.id)
                }
                else {
                    println!("ID was already empty!");
                }
            }
        }
        requests.clear(); // Empty the list of requests when they have been handled
    }
}