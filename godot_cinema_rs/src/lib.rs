use gdnative::*;
use opencv::core;
use opencv::videoio::VideoCapture;
use opencv::videoio::CAP_ANY;

/// The HelloWorld "class"
pub struct Screen {
    pub vid: Option<VideoCapture>,
    pub img: Option<Image>,
}

unsafe impl Send for Screen {}

impl NativeClass for Screen {
    type Base = Spatial;
    type UserData = user_data::MutexData<Screen>;

    fn class_name() -> &'static str {
        "Screen"
    }

    fn init(_owner: Self::Base) -> Self {
        unsafe { Self::_init(_owner) }
    }
}

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl Screen {
    /// The "constructor" of the class.
    fn _init(_owner: Spatial) -> Self {
        Screen {
            vid: None,
            img: None,
        }
    }
    // In order to make a method known to Godot, the #[export] attribute has to be used.
    // In Godot script-classes do not actually inherit the parent class.
    // Instead they are"attached" to the parent object, called the "owner".
    // The owner is passed to every single exposed method.
    #[export]
    fn _ready(&mut self, _owner: Spatial) {
        let mut vid = VideoCapture::new_from_file_with_backend("sample.mp4", CAP_ANY).unwrap();
        let opened = VideoCapture::is_opened(&vid).unwrap();
        if opened {
            self.vid = Some(vid);
            godot_print!("video is open");
        } else {
            self.vid = None;
            godot_print!("video is closed");
        }
        // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.
    }

    #[export]
    fn _process(&mut self, owner: Spatial, delta: f64) {
        if let Some(vid) = &mut self.vid {
            let mut frame = core::Mat::default().unwrap();
            vid.read(&mut frame).unwrap();

            let size = frame.size().unwrap();
            let width = size.width;
            let height = size.height;

            let byte_length = width * height * 3;
            let mut byte_array = gdnative::ByteArray::new();
            byte_array.resize(byte_length);

            // is causing the crash
            unsafe {
                std::ptr::copy(
                    frame.data_mut().unwrap() as *mut _,
                    byte_array.write().as_mut_ptr() as *mut _,
                    byte_length as usize,
                );
            }
            let mut img = Image::new();
            img.create_from_data(width as i64, height as i64, false, 4, byte_array);

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
