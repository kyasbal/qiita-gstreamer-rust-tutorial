#[cfg(feature = "tutorial5")]
mod tutorial5{
    extern crate gstreamer as gst;

    use gst::prelude::*;
    use gtk::*;
    use gdk::prelude::*;

    pub fn run(){
        initialize_gtk_gstreaner();

        let uri = "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
        let playbin = gst::ElementFactory::make("playbin",None).unwrap();
        playbin.set_property("uri", &uri).unwrap();

        playbin.connect("video-tags-changed",false,|args| {
            let pipeline = args[0].get::<gst::Element>().expect("Failed to get value in video-tags-changed argument").unwrap();
            post_app_message(&pipeline);
            None
        }).expect("Failed to connect to video-tag-changed");

        playbin.connect("audio-tags-changed",false,|args|{
            let pipeline = args[0].get::<gst::Element>().expect("Failed to get value in audio-tags-changed argument").unwrap();
            post_app_message(&pipeline);
            None
        }).expect("Failed to connect to audio-tags-changed");

        playbin.connect("text-tags-changed",false,|args|{
            let pipeline = args[0].get::<gst::Element>().expect("Failed to get value in text-tags-changed argument").unwrap();
            post_app_message(&pipeline);
            None
        }).expect("Failed to connect to text-tags-changed");

        create_ui(&playbin);
    }

    fn post_app_message(pipeline: &gst::Element){
        
    }

    fn create_ui(playbin: & gst::Element){
        let main_window = Window::new(WindowType::Toplevel);
        main_window.connect_delete_event(|_,_| {
            gtk::main_quit();
            Inhibit(false)
        });

        main_window.show_all();
        gtk::main();
    }

    fn initialize_gtk_gstreaner(){
        if let Err(err) = gtk::init(){
            eprintln!("Failed to initialize gtk");
        }
        if let Err(err) = gst::init() {
            eprintln!("Failed to initialize gst");
        }
    }
}

#[cfg(feature = "tutorial5")]
fn main(){
    tutorial5::run();
}

#[cfg(not(feature = "tutorial5"))]
fn main(){
    println!("Please compile with --features tutorial5[-x11][-quartz]");
}