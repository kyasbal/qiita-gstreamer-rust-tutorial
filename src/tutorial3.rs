extern crate gstreamer as gst;
use gst::prelude::*;

fn main(){
    gst::init().unwrap();

    // Instanciate elements in pipeline
    let source = gst::ElementFactory::make("uridecodebin", Some("source")).expect("Could not instanciate uridecodebin");
    let convert = gst::ElementFactory::make("audioconvert", Some("convert")).expect("Could not instanciate audioconvert");
    let sink = gst::ElementFactory::make("autoaudiosink", Some("sink")).expect("Could not instanciate audiosink");

    // Instanciate pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Add all elements inside of the pipeline
    pipeline.add_many(&[&source,&convert,&sink]).unwrap();
    convert.link(&sink).expect("element could not be linked");
    // It is impossible to link with source and convert here.

    // Set uri property in uridecodebin
    let uri = "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
    source.set_property("uri",&uri).expect("Couldn't set uri property on uridecodebin");

    // Initiate weak pointer to be used in different thread
    let pipeline_weak = pipeline.downgrade();
    let convert_weak = convert.downgrade();
    
    // Add event listener
    source.connect_pad_added(move |_,src_pad|{
        // Getting actual reference from weak reference if it was not discarded
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline)=>pipeline,
            None=>return,
        };
        let convert = match convert_weak.upgrade() {
            Some(convert)=>convert,
            None=>return,
        };
        println!("Received new pad {} from {}",src_pad.get_name(),pipeline.get_name());

        // Obtain the sink_pad from audioconvert element
        let sink_pad = convert.get_static_pad("sink").expect("Failed to get static sink pad from convert");
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoreing");
            return;
        }
        
        // Retrive capability of the elements
        let new_pad_caps = src_pad.get_current_caps().expect("Failed to get caps of new pad");
        let new_pad_struct  = new_pad_caps.get_structure(0).expect("Failed to get first structure of caps");
        let new_pad_type = new_pad_struct.get_name();
        
        // Check this pad is for audio, otherwise, it should be for video
        let is_audio = new_pad_type.starts_with("audio/x-raw");
        if !is_audio{
            println!("It has type {} which is not a raw audio.Ignoreing",new_pad_type);
            return;
        }

        // Link the src pad to sink pad
        let res = src_pad.link(&sink_pad);
        if res.is_err() {
            println!("Type is {} but link failed",new_pad_type);
        }else{
            println!("Link succeeded type {}",new_pad_type)
        }
    });

    // Make pipeline state Playing
    pipeline.set_state(gst::State::Playing).expect("Failed to set the pipeline to the playing state");

    // Obtain the bus and loop while monitor the messages
    let bus = pipeline.get_bus().unwrap();
    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE){
        use gst::MessageView;
        match msg.view(){
            gst::MessageView::Error(err)=>{
                eprintln!("Error received from element {:?} {}",err.get_src().map(|s| s.get_path_string()),err.get_error());
                eprintln!("Debugging information {:?}",err.get_debug());
                break;
            }
            gst::MessageView::StateChanged(state_changed)=>{ // If pipeline state was changed
                if state_changed.get_src().map(|s| s == pipeline).unwrap_or(false){
                    println!("Pipeline state was changed from {:?}| to {:?}",state_changed.get_old(),state_changed.get_current());
                }
            }
            gst::MessageView::Eos(..)=>break,
            _=>()
        }
    }

    //Cleaning up
    pipeline.set_state(gst::State::Null).expect("Failed to set the pipeline state to null");
}