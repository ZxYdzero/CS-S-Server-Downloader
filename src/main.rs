#![allow(dead_code)]
use std::fs::{self};
use std::io::{copy, self};
use std::path::Path;

use iced::executor;
use iced::widget::{button, column, container, progress_bar, text, Column};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    Theme,
};
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

mod download;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Debug)]
struct Example {
    downloads: Vec<Download>,
    last_id: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Download(usize),
    DownloadProgressed((usize, download::Progress)),

}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Example, Command<Message>) {
        (
            Example {
                downloads: vec![Download::new(0)],
                last_id: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("下载器")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Download(index) => {
                if let Some(download) = self.downloads.get_mut(index) {
                    download.start();
                }
            }
            Message::DownloadProgressed((id, progress)) => {
                if let Some(download) =
                    self.downloads.iter_mut().find(|download| download.id == id)
                {
                    download.progress(progress);
                }
            }

        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.downloads.iter().map(Download::subscription))
    }

    fn view(&self) -> Element<Message> {
        let downloads = Column::with_children(
            self.downloads.iter().map(Download::view).collect(),
        )
        .spacing(20)
        .align_items(Alignment::End);

        container(downloads)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
    }
}

#[derive(Debug)]
struct Download {
    id: usize,
    state: State,
}

#[derive(Debug)]
enum State {
    Idle,
    Downloading { progress: f32 },
    Finished,
    Errored,
}

impl Download {
    pub fn new(id: usize) -> Self {
        Download {
            id,
            state: State::Idle,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            State::Idle { .. }
            | State::Finished { .. }
            | State::Errored { .. } => {
                self.state = State::Downloading { progress: 0.0 };
            }
            State::Downloading { .. } => {}
        }
    }

    pub fn progress(&mut self, new_progress: download::Progress) {
        if let State::Downloading { progress } = &mut self.state {
            match new_progress {
                download::Progress::Started => {
                    *progress = 0.0;
                }
                download::Progress::Advanced(percentage) => {
                    *progress = percentage;
                }
                download::Progress::Finished => {
                    self.state = State::Finished;
                    let mut hecystring = find();
                    hecystring.push_str("\\cstrike\\download");
                    println!("正在解压");

                    extract(Path::new("./tmp/tmp.zip"), Path::new("./tmp"));
                    println!("解压完成");
                    println!("正在导入");
                    let pr = hecystring.clone();
                    println!("导入位置：{}", &pr);
                    copy_dir_recursively("./tmp/csgo-server-map-qq_26978213-master-patch-98686/", &pr.clone()).expect("无法导入地图文件");
        
                    println!("导入完成");
                    println!("删除tmp");
                    let error = remove();
                    error.unwrap();
                    println!("删除完成");
                }
                download::Progress::Errored => {
                    self.state = State::Errored;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Downloading { .. } => {
                download::file(self.id, "https://gitcode.net/qq_26978213/csgo-server-map/-/archive/qq_26978213-master-patch-98686/csgo-server-map-qq_26978213-master-patch-98686.zip")
                    .map(Message::DownloadProgressed)
            }
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let current_progress = match &self.state {
            State::Idle { .. } => 0.0,
            State::Downloading { progress } => *progress,
            State::Finished { .. } => 100.0,
            State::Errored { .. } => 0.0,
        };

        let progress_bar = progress_bar(0.0..=100.0, current_progress);

        let control: Element<_> = match &self.state {
            State::Idle => button("Start The Download!")
                .on_press(Message::Download(self.id))
                .into(),
            State::Finished => {
                column!["Waiting For Unzip"]
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .into()
            }
            State::Downloading { .. } => {
                text(format!("Downloading... {current_progress:.2}%")).into()
            }
            State::Errored => column![
                "ERROR :(",
                button("Retry").on_press(Message::Download(self.id)),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .into(),
        };

        Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .push(progress_bar)
            .push(control)
            .into()
    }
}

//查询注册表的函数 //主要
fn find() -> String {

    let hkcu: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);
    let place: RegKey = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 240").unwrap();
    let path: String = place.get_value("InstallLocation").unwrap();
    path
}

fn extract(test: &Path, target: &Path) {
    let zipfile = std::fs::File::open(&test).unwrap();
    let mut zip = zip::ZipArchive::new(zipfile).unwrap();

    if !target.exists() {
        let _ = fs::create_dir_all(target).map_err(|e| {
            println!("{}", e);
        });
    }
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        println!("Filename: {} {:?}", file.name(), file.sanitized_name());

        if file.is_dir() {
            println!("file utf8 path {:?}", file.name_raw());
            let target = target.join(Path::new(&file.name().replace("\\", "")));
            fs::create_dir_all(target).unwrap();
        } else {
            let file_path = target.join(Path::new(file.name()));
            let mut target_file = if !file_path.exists() {
                println!("file path {}", file_path.to_str().unwrap());
                fs::File::create(file_path).unwrap()
            } else {
                fs::File::open(file_path).unwrap()
            };
            let _ = copy(&mut file, &mut target_file);

        }
    }
}

fn copy_dir_recursively(src: &str, dst: &str) -> io::Result<()> {

    // 遍历源目录下的所有目录项
    println!("源目录位于{}", src);
    println!("目标目录位于{}", dst);
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        fs::create_dir_all(&dst).unwrap();


        // 如果是文件，则直接复制到目标目录
        if entry.metadata()?.is_file() {
            let dst_path = format!("{}\\{}", dst, file_name);
            fs::copy(&path, &dst_path)?;
        }
        // 如果是目录，则递归调用复制函数进行复制
        else {

            let dst_path = format!("{}\\{}", dst, file_name);
            copy_dir_recursively(&path.to_string_lossy(), &dst_path)?;
        }
    }

    Ok(())
}

fn remove() -> io::Result<()>{
    fs::remove_dir_all("./tmp")?;
    Ok(())
}
