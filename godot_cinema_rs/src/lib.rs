use gdnative::*;
use opencv::core;
use opencv::videoio::VideoCapture;
use opencv::videoio::CAP_ANY;

use std::process::{Child, Command};
use std::sync::mpsc;

pub fn get_youtube_video_url(youtube_url: &str) -> String {
    let stdout = Command::new("youtube-dl")
        .args(vec!["--format", "160", "--get-url", youtube_url])
        .output()
        .unwrap()
        .stdout;
    let url = String::from_utf8(stdout).unwrap();
    url
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

fn frame_to_img(frame: &mut opencv::core::Mat) -> Image {
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

pub struct Screen {
    pub vid: Option<VideoCapture>,
    pub img: Option<Image>,
    pub timer: Option<Timer>,
    pub rx: Option<mpsc::Receiver<VideoCapture>>,
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
    fn _init(_owner: Spatial) -> Self {
        Screen {
            vid: None,
            img: None,
            timer: None,
            rx: None,
        }
    }

    #[export]
    fn _ready(&mut self, owner: Spatial) {
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let youtube_url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";
            let video_url = get_youtube_video_url(youtube_url);
            let vid = get_youtube_video(&video_url).unwrap();
            tx.send(vid).unwrap();
        });
    }

    #[export]
    fn _process(&mut self, owner: Spatial, delta: f64) {
        if let Some(rx) = &self.rx {
            if let Ok(vid) = rx.try_recv() {
                self.vid = Some(vid);
            }
        }
        if let Some(vid) = &mut self.vid {
            let mut frame = core::Mat::default().unwrap();
            vid.read(&mut frame).unwrap();
            let img = frame_to_img(&mut frame);

            let mut txt = ImageTexture::new();
            txt.create_from_image(Some(img), 7);

            let path = NodePath::from_str("Screen/CollisionShape/MeshInstance");
            unsafe {
                let mesh = owner
                    .get_node(path)
                    .unwrap()
                    .cast::<MeshInstance>()
                    .unwrap();
                if let Some(mat) = mesh.get_surface_material(0) {
                    let mut mat = mat.cast::<SpatialMaterial>().unwrap();
                    mat.set_texture(0, txt.cast::<Texture>());
                }
            }
        }
    }
    #[export]
    fn get_img(&mut self, _owner: Spatial) -> Image {
        if let Some(img) = &self.img {
            img.new_ref()
        } else {
            Image::new()
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
