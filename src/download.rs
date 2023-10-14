use iced::subscription;
use std::{hash::Hash, fs::{File, self}, io::Write};


pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    url: T,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(id, State::Ready(url.to_string()), move |state| {
        download(id, state)
    })
}

#[derive(Debug, Hash, Clone)]
pub struct Download<I> {
    id: I,
    url: String,
    
}

async fn download<I: Copy>(id: I, state: State) -> ((I, Progress), State) {
    
    match state {
        State::Ready(url) => {

            let response = reqwest::get(&url).await;
            fs::create_dir("./tmp").unwrap();

            let dest = File::create(".\\tmp\\tmp.zip").unwrap();

            match response {
                Ok(response) => {
                    if let Some(total) = response.content_length() {
                        let length = response.content_length().unwrap();
                        println!("下载长度在Reqwest为{}", length);
                        (
                            (id, Progress::Started),
                            State::Downloading {
                                dest,
                                response,
                                total,
                                downloaded: 0,
                            },
                        )
                    } else {
                        ((id, Progress::Errored), State::Finished)
                    }
                }
                Err(_) => ((id, Progress::Errored), State::Finished),
            }
        }
        State::Downloading {
            mut dest,
            mut response,
            total,
            downloaded,
        } => match response.chunk().await {
            Ok(Some(chunk)) => {
                let downloaded = downloaded + chunk.len() as u64;
                let percentage = (downloaded as f32 / total as f32) * 100.0;
                dest.write_all(&chunk).unwrap();
                (
                    (id, Progress::Advanced(percentage)),
                    State::Downloading {
                        dest,
                        response,
                        total,
                        downloaded,
                    },
                )
            }
            Ok(None) => ((id, Progress::Finished), State::Finished),
            Err(_) => ((id, Progress::Errored), State::Finished),
        },
        State::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.

            iced::futures::future::pending().await
        }
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}

pub enum State {
    Ready(String),
    Downloading {
        dest: File,
        response: reqwest::Response,
        total: u64,
        downloaded: u64,
    },
    Finished,
}
