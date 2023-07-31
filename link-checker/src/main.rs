use reqwest::{blocking::Client, Url};
use scraper::{Html, Selector};
use thiserror::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Error, Debug)]
enum Error {
    #[error("request error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("bad http response: {0}")]
    BadResponse(String),
}

#[derive(Debug)]
struct CrawlCommand {
    url: Url,
    extract_links: bool,
}

#[derive(Debug)]
struct CheckedLink {
    url: Url,
    is_valid: bool,
}

#[derive(Debug)]
struct LinkCheckerWorker {
    link_channel: mpsc::Receiver<Url>,
    result_channel: mpsc::Sender<CheckedLink>,
    client: Client
}

impl LinkCheckerWorker {
    fn new(link_channel: mpsc::Receiver<Url>, result_channel: mpsc::Sender<CheckedLink>) -> Self {
        Self { link_channel, result_channel, client: Client::new()}
    }

    fn check_link(&self, url: Url) -> Result<Url, Error>{
        //println!("Checking {:#}", url);
        let response = self.client.get(url.clone()).send()?;
        if !response.status().is_success() {
            return Err(Error::BadResponse(response.status().to_string()));
        }
        Ok(url)
    }

    fn run(&self) {
        loop {
            match self.link_channel.recv() {
                Ok(url) => {
                    let checked_link = match self.check_link(url.clone()) {
                        Ok(url) => CheckedLink { url, is_valid: true },
                        Err(_) => CheckedLink{url, is_valid: false},
                    };
                    self.result_channel.send(checked_link).expect("Error sending results!")
                },
                Err(_) => break,
            }
        }
    }
}

fn visit_page(client: &Client, command: &CrawlCommand, worker_queues: Vec<mpsc::Sender<Url>>) -> Result<Vec<Url>, Error> {
    println!("Checking {:#}", command.url);
    let response = client.get(command.url.clone()).send()?;
    if !response.status().is_success() {
        return Err(Error::BadResponse(response.status().to_string()));
    }

    let mut link_urls = Vec::new();
    if !command.extract_links {
        return Ok(link_urls);
    }

    let base_url = response.url().to_owned();
    let body_text = response.text()?;
    let document = Html::parse_document(&body_text);

    let selector = Selector::parse("a").unwrap();
    let href_values = document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"));
    for (index, href) in href_values.enumerate() {
        match base_url.join(href) {
            Ok(link_url) => {
                link_urls.push(link_url.clone());
                // use all workers by cycling through them
                worker_queues[index % worker_queues.len()].send(link_url).expect("Error sending link to worker!");
            }
            Err(err) => {
                println!("On {base_url:#}: ignored unparsable {href:?}: {err}");
            }
        }
    }
    Ok(link_urls)
}

fn create_input_channels(num: i8) -> (Vec<mpsc::Sender<Url>>, Vec<mpsc::Receiver<Url>>) {
    let mut senders: Vec<mpsc::Sender<Url>> = Vec::new();
    let mut receivers: Vec<mpsc::Receiver<Url>> = Vec::new();
    for _ in 0..num {
        let (sender, receiver) = mpsc::channel::<Url>();
        senders.push(sender);
        receivers.push(receiver);
    }
    return (senders, receivers);
}

fn main() {
    let client = Client::new();
    let start_url = Url::parse("https://www.google.org").unwrap();
    let crawl_command = CrawlCommand{ url: start_url, extract_links: true };

    // result channel
    let (send_results, receive_results) = mpsc::channel::<CheckedLink>();
    
    // create and start workers
    const NUM_WORKERS: i8 = 5;
    let (worker_input_senders, worker_input_receivers) = create_input_channels(NUM_WORKERS);
    
    let mut handlers: Vec<_> = Vec::new();
    for input_queue in worker_input_receivers {
        let worker = LinkCheckerWorker::new(input_queue, send_results.clone());
        let handler = thread::spawn(move || { worker.run() });
        handlers.push(handler);
    }

    // crawl page
    let links = match visit_page(&client, &crawl_command, worker_input_senders.clone()) {
        Ok(links) => links,
        Err(err) => {
            println!("Could not extract links: {err:#}");
            Vec::new()
        },
    };
    drop(links);

    // stop worker by closing the input channel
    let _ = worker_input_senders.iter().map(|sender| { drop(sender) } );
    let _ = handlers.into_iter().map(|handle| handle.join().expect("Error joining threads"));
    
    drop(send_results);
    loop {
        match receive_results.recv() {
            Ok(link) => {
                println!("Link: {} : {}", link.url.to_string(), if link.is_valid { "valid "} else { "invalid" } );
            },
            Err(_) => break,
        }
    }
    println!("Finished.")
}