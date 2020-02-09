use gdnative::*;

use std::fs::File;
use std::io::Read;
use std::process::{Child, Command};
use std::sync::mpsc;

use opencv::core;
use opencv::videoio::VideoCapture;
use opencv::videoio::CAP_ANY;

pub fn get_youtube_video_url(youtube_url: &str) -> String {
    let stdout = Command::new("youtube-dl")
        .args(vec!["--format", "248", "--get-url", youtube_url])
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(stdout).unwrap()
}

pub fn get_youtube_audio_url(youtube_url: &str) -> String {
    let stdout = Command::new("youtube-dl")
        .args(vec!["--format", "249", "--get-url", youtube_url])
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(stdout).unwrap()
}

pub fn save_youtube_audio_to_wav(audio_url: &str) -> String {
    let f_name = "audio.wav";

    godot_print!("ffmpeg started downloading: {}", audio_url);

    let stdout = Command::new("ffmpeg")
        .args(vec!["-y", "-i", &audio_url.trim(), f_name])
        .output()
        .unwrap()
        .stderr;

    let output = String::from_utf8(stdout).unwrap();
    godot_print!("ffmpeg output: {}", output);

    godot_print!("ffmpeg works");

    f_name.into()
}

fn get_youtube_video(video_url: &str) -> Option<VideoCapture> {
    let vid = VideoCapture::new_from_file_with_backend(&video_url, CAP_ANY).unwrap();
    let opened = VideoCapture::is_opened(&vid).unwrap();

    if opened {
        Some(vid)
    } else {
        None
    }
}

fn frame_to_img(frame: &mut core::Mat) -> Image {
    let size = frame.size().unwrap();
    let width = size.width;
    let height = size.height;

    let byte_length = width * height * 3;
    let mut byte_array = gdnative::ByteArray::new();
    byte_array.resize(byte_length);

    unsafe {
        std::ptr::copy(
            frame.data_mut().unwrap() as *mut _,
            byte_array.write().as_mut_ptr() as *mut _,
            byte_length as usize,
        );
    }
    let mut img = Image::new();
    img.create_from_data(width as i64, height as i64, false, 4, byte_array);
    img
}

pub struct YoutubeInfo {
    pub video_url: String,
    pub audio_url: String,
    pub audio_file_name: String,
}

pub struct Screen {
    pub vid: Option<VideoCapture>,
    pub rx: Option<mpsc::Receiver<YoutubeInfo>>,
    pub frame: core::Mat,
    pub img_texture: ImageTexture,
    pub audio_player: Option<AudioStreamPlayer3D>,
}

unsafe impl Send for Screen {}

impl NativeClass for Screen {
    type Base = Spatial;
    type UserData = user_data::MutexData<Screen>;

    fn class_name() -> &'static str {
        "Screen"
    }

    fn init(_owner: Self::Base) -> Self {
        Self::_init(_owner)
    }
}

#[methods]
impl Screen {
    fn _init(owner: Spatial) -> Self {
        Self {
            vid: None,
            frame: core::Mat::default().unwrap(),
            rx: None,
            audio_player: None,
            // needs to be initialized here otherwise it will cause major memory leak
            img_texture: ImageTexture::new(),
        }
    }

    fn add_youtube_video(&mut self, youtube_url: String) {
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        std::thread::spawn(move || {
            let video_url = get_youtube_video_url(youtube_url.as_str());
            let audio_url = get_youtube_audio_url(youtube_url.as_str());
            let audio_file_name = save_youtube_audio_to_wav(&audio_url);

            let youtube_info = YoutubeInfo {
                video_url,
                audio_url,
                audio_file_name,
            };
            tx.send(youtube_info).unwrap();
        });
    }

    #[export]
    fn _on_youtube_url_entry_text_entered(&mut self, owner: Spatial, youtube_url: String) {
        self.add_youtube_video(youtube_url);
    }

    #[export]
    fn _ready(&mut self, owner: Spatial) {
        self.audio_player = unsafe {
            owner
                .get_node(NodePath::from_str("Audio"))
                .unwrap()
                .cast::<AudioStreamPlayer3D>()
        };
        let video = "https://www.youtube.com/watch?v=OfIQW6s1-ew";

        //self.add_youtube_video(video.into());
    }

    #[export]
    fn _process(&mut self, owner: Spatial, delta: f64) {
        if let Some(rx) = &self.rx {
            if let Ok(youtube_info) = rx.try_recv() {
                let mut f = File::open(&youtube_info.audio_file_name).unwrap();
                let mut bytes: Vec<u8> = Vec::new();
                f.read_to_end(&mut bytes).unwrap();

                let byte_length = bytes.len();

                let mut byte_array = gdnative::ByteArray::new();
                byte_array.resize(byte_length as i32);

                unsafe {
                    std::ptr::copy(
                        bytes.as_mut_ptr() as *mut _,
                        byte_array.write().as_mut_ptr() as *mut _,
                        byte_length,
                    );
                }

                let mut audio_sample = AudioStreamSample::new();
                audio_sample.set_data(byte_array);

                // sets audio to 16bit
                audio_sample.set_format(1);

                // very funny audio results without this
                audio_sample.set_mix_rate(48000);
                audio_sample.set_stereo(true);

                unsafe {
                    if let Some(audio_player) = &mut self.audio_player {
                        audio_player.set_stream(audio_sample.cast::<AudioStream>());
                        audio_player.play(0.0);
                    }
                }

                self.vid = get_youtube_video(&youtube_info.video_url);
            }
        }

        if let Some(vid) = &mut self.vid {
            vid.read(&mut self.frame).unwrap();
            let img = frame_to_img(&mut self.frame);
            self.apply_img_to_texture(owner, img);
        }
    }

    fn apply_img_to_texture(&mut self, owner: Spatial, img: Image) {
        self.img_texture.create_from_image(Some(img), 7);
        let path = NodePath::from_str("Screen/CollisionShape/MeshInstance");

        let mesh = unsafe {
            owner
                .get_node(path)
                .unwrap()
                .cast::<MeshInstance>()
                .unwrap()
        };

        let option_mat = unsafe { mesh.get_surface_material(0) };
        if let Some(mat) = option_mat {
            let mut spatial_mat = mat.cast::<SpatialMaterial>().unwrap();
            let texture = self.img_texture.to_texture();
            spatial_mat.set_texture(0, Some(texture));
        }
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<Screen>();
}

// macros that create the entry-points of the dynamic library.
godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
