use std::sync::Arc;
use tokio::{time, select, join};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;

struct Fork;

struct Philosopher {
    name: String,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    thoughts: Sender<String>,
}

impl Philosopher {
    async fn think(&self) {
        self.thoughts
            .send(format!("Eureka! {} has a new idea!", &self.name)).await.unwrap();
    }

    async fn eat(&self) {
        // Pick up forks...
        let _ = self.left_fork.lock().await;
        // Add a delay before picking the second fork to allow the execution
        // to transfer to another task
        time::sleep(time::Duration::from_millis(1)).await;
        
        let _ = self.right_fork.lock().await;
        println!("{} is eating...", &self.name);
        time::sleep(time::Duration::from_millis(5)).await;
    }
}

static PHILOSOPHERS: &[&str] =
    &["Socrates", "Hypatia", "Plato", "Aristotle", "Pythagoras"];

#[tokio::main]
async fn main() {
    // Create forks
    let forks = 
        vec![Fork, Fork, Fork, Fork, Fork]
        .into_iter().map(|x| Arc::new(Mutex::new(x)))
        .collect::<Vec<Arc<Mutex<Fork>>>>();

    // Create philosophers
    let (sender, mut receiver) = mpsc::channel::<String>(10);

    let mut philosophers = Vec::new();
    for (index, philo) in PHILOSOPHERS.iter().enumerate() {
        if index == 0 {
            philosophers.push(Philosopher {
                name: philo.to_owned().into(),
                left_fork: forks.get(index).unwrap().clone(),
                right_fork: forks.get(PHILOSOPHERS.len() - 1).unwrap().clone(),
                thoughts: sender.clone(),
            });
        } else {
            philosophers.push(Philosopher {
                name: philo.to_owned().into(),
                left_fork: forks.get(index - 1).unwrap().clone(),
                right_fork: forks.get(index).unwrap().clone(),
                thoughts: sender.clone(),
            });
        }
    }

    // for philo in philosophers {
    //     tokio::spawn(async move {
    //         philo.think().await;
    //         philo.eat().await;
    //     });
   // }
    for phil in philosophers {
        tokio::spawn(async move {
            for _ in 0..100 {
                phil.think().await;
                phil.eat().await;
            }
        });
    }

    // Output their thoughts
    while let Some(thought) = receiver.recv().await {
        println!("Here is a thought: {thought}");
    }
}