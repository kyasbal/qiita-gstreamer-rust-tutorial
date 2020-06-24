// extern crate gstreamer as gst;
// use gst::prelude::*;

// fn main(){
//     // Initialize pipeline
//     gst::init().unwrap();

//     // Instanciating source and sink elements
//     let source = gst::ElementFactory::make("videotestsrc", Some("source")).expect("Could not create a source element");
//     let sink = gst::ElementFactory::make("autovideosink",Some("sink")).expect("Could not create a sink element");

//     // Instanciate pipeline
//     let pipeline = gst::Pipeline::new(Some("test-manual-pipeline"));

//     // Connecting each elements
//     pipeline.add_many(&[&source,&sink]).unwrap();
//     source.link(&sink).expect("Elements could not be linked");

//     // Set a property
//     source.set_property_from_str("pattern", "smpte");
//     source.set_property(property_name, value)

//     // Start playing pipeline
//     pipeline.set_state(gst::State::Playing).expect("Unable to set the pipeline to Playing state");

//     // Watch pipeline until Eos or getting an error
//     let bus = pipeline.get_bus().unwrap();
//     for msg in bus.iter_timed(gst::ClockTime::from_mseconds(3000)){
//         match msg.view() {
//             gst::MessageView::Error(err)=>{
//                 eprintln!(
//                     "Error received from element {:?}: {}",
//                     err.get_src().map(|s| s.get_path_string()),
//                     err.get_error()
//                 );
//                 eprintln!("Debugging information: {:?}", err.get_debug());
//                 break;
//             }
//             gst::MessageView::Eos(..)=>break,
//             _=>(),
//         }
//     }
//     source.set_property_from_str("pattern", "snow");
//     for msg in bus.iter_timed(gst::ClockTime::from_mseconds(3000)){
//         match msg.view() {
//             gst::MessageView::Error(err)=>{
//                 eprintln!(
//                     "Error received from element {:?}: {}",
//                     err.get_src().map(|s| s.get_path_string()),
//                     err.get_error()
//                 );
//                 eprintln!("Debugging information: {:?}", err.get_debug());
//                 break;
//             }
//             gst::MessageView::Eos(..)=>break,
//             _=>(),
//         }
//     }

//     // Cleaning up
//     pipeline.set_state(gst::State::Null).expect("Unable to set the pipeline to `Null` state");
// }