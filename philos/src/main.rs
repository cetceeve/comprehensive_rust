use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Fork;

struct Philosopher {
    name: String,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    thoughts: mpsc::SyncSender<String>,
}

impl Philosopher {
    fn think(&self) {
        self.thoughts
            .send(format!("Eureka! {} has a new idea!", &self.name))
            .unwrap();
    }

    fn eat(&self) {
        // Pick up forks...
        let _left_guard = self.left_fork.lock().unwrap();
        let _right_goard = self.right_fork.lock().unwrap();
        println!("{} is eating...", &self.name);
        thread::sleep(Duration::from_millis(10));
    }
}

static PHILOSOPHERS: &[&str] = &["Socrates", "Plato", "Aristotle", "Thales", "Pythagoras"];

fn main() {
    // Create forks
    // let s_left = Arc::new(Mutex::new(Fork));
    // let p_left = Fork;
    // let a_left = Fork;
    // let t_left = Fork;
    // let py_left = Fork;
    let forks = vec![
        Arc::new(Mutex::new(Fork)),
        Arc::new(Mutex::new(Fork)),
        Arc::new(Mutex::new(Fork)),
        Arc::new(Mutex::new(Fork)),
        Arc::new(Mutex::new(Fork)),
    ];

    // Create philosophers
    let (sender, receiver) = mpsc::sync_channel(10);

    let mut philosophers = Vec::new();
    for (index, philo) in PHILOSOPHERS.iter().enumerate() {
        // To avoid a deadlock, we have to break the symmetry.
        // Therefore we swith the left/right fork for one philosopher
        if index == 0 {
            philosophers.push(Philosopher {
                name: philo.to_string(),
                left_fork: forks[index].clone(),
                right_fork: forks[PHILOSOPHERS.len() - 1].clone(),
                thoughts: sender.clone(),
            });
        } else {
            philosophers.push(Philosopher {
                name: philo.to_string(),
                left_fork: forks[index - 1].clone(),
                right_fork: forks[index].clone(),
                thoughts: sender.clone(),
            });
        }
    }
    // Make each of them think and eat 100 times
    for philo in philosophers {
        thread::spawn(move || {
            for _ in 0..100 {
                philo.think();
                philo.eat();
            }
        });
    }

    // Output their thoughts
    loop {
        match receiver.recv() {
            Ok(msg) => println!("{msg}"),
            Err(_) => print!("Something went wrong!"),
        }
    }
}
