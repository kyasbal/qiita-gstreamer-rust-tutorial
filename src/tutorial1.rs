// extern crate gstreamer as gst;
// use gst::prelude::*;


// fn main(){
//         // Initialize GStreamer
//         gst::init().unwrap();

//         // Build the pipeline
//         let uri =
//             "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
//         let pipeline = gst::parse_launch(&format!("playbin uri={}", uri)).unwrap();
    
//         // Start playing
//         pipeline
//             .set_state(gst::State::Playing)
//             .expect("Unable to set the pipeline to the `Playing` state");
    
//         // Wait until error or EOS
//         let bus = pipeline.get_bus().unwrap();
//         let msg_types = [gst::MessageType::Error,gst::MessageType::Eos];
//         bus.timed_pop_filtered(gst::ClockTime::from_mseconds(1000), &msg_types);
    
//         // Shutdown pipeline
//         pipeline
//             .set_state(gst::State::Null)
//             .expect("Unable to set the pipeline to the `Null` state");
    
// }