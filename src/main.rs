// This section is only works if --feature tutorial5 was specified on build
#[cfg(feature = "tutorial5")]
mod tutorial5 {
    extern crate gstreamer as gst;
    extern crate gstreamer_video as gst_video;

    // gdk and gtk won't be available if there are no #[cfg(feature = "tutorial5")] on scope they enclosed in.
    use gdk::prelude::*;
    use gst::prelude::*;
    use gst_video::prelude::*;
    use gtk::*;

    use std::os::raw::c_void;
    use std::process;

    use glib::object::ObjectType;

    pub fn run() {
        initialize_gtk_gstreaner(); // Initialize gtk and gstreamer

        // Initialize playbin with single file source
        let uri = "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
        let playbin = gst::ElementFactory::make("playbin", None).unwrap();
        playbin.set_property("uri", &uri).unwrap();

        // Add event handler to be notified when video-tag was changed
        playbin
            .connect("video-tags-changed", false, |args| {
                let pipeline = args[0]
                    .get::<gst::Element>()
                    .expect("Failed to get value in video-tags-changed argument")
                    .unwrap();
                // This will send message to application thread
                post_app_message(&pipeline);
                None
            })
            .expect("Failed to connect to video-tag-changed");

        // Add event handler to be notified when audio-tag was changed
        playbin
            .connect("audio-tags-changed", false, |args| {
                let pipeline = args[0]
                    .get::<gst::Element>()
                    .expect("Failed to get value in audio-tags-changed argument")
                    .unwrap();
                // This will send message to application thread
                post_app_message(&pipeline);
                None
            })
            .expect("Failed to connect to audio-tags-changed");

        // Add event handler to be notified when audio-tag was changed
        playbin
            .connect("text-tags-changed", false, |args| {
                let pipeline = args[0]
                    .get::<gst::Element>()
                    .expect("Failed to get value in text-tags-changed argument")
                    .unwrap();
                // This will send message to application thread
                post_app_message(&pipeline);
                None
            })
            .expect("Failed to connect to text-tags-changed");

        // Construct the ui
        create_ui(&playbin);

        // Instruct the bus to emit signals for each received message, and connect to the interesting signals
        let bus = playbin.get_bus().unwrap();
        bus.add_signal_watch();

        let pipeline_weak = playbin.downgrade();
        bus.connect_message(move |_, msg| {
            let pipeline = match pipeline_weak.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };

            match msg.view() {
                gst::MessageView::Eos(..) => {
                    println!("End of stream reached");
                    pipeline
                        .set_state(gst::State::Ready)
                        .expect("Unable to set pipeline to the ready state");
                }
                gst::MessageView::Error(err) => {
                    println!(
                        "Error from {:?}: {} ({:?})",
                        err.get_src().map(|s| s.get_path_string()),
                        err.get_error(),
                        err.get_debug()
                    );
                }
                gst::MessageView::StateChanged(state_changed) => {
                    if state_changed
                        .get_src()
                        .map(|s| s == pipeline)
                        .unwrap_or(false)
                    {
                        println!("State set to {:?}", state_changed.get_current());
                    }
                }
                _ => (),
            }
        });
        // start [;auomg]
        playbin
            .set_state(gst::State::Playing)
            .expect("Unable to set the playbin to the `Playing` state");
        // Start the GTK main loop. We will not regain control until gtk::main_quit(); is called.
        gtk::main();

        // Cleaning up
        playbin
            .set_state(gst::State::Null)
            .expect("Unable to set the playbin to the Null state");
    }

    fn initialize_gtk_gstreaner() {
        gtk::init().unwrap();
        gst::init().unwrap();
    }

    fn create_ui(playbin: &gst::Element) {
        // Instanciate window, button, sliders and register their event handlers
        let main_window = Window::new(WindowType::Toplevel);
        main_window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
        let pipeline = playbin.clone();
        let play_button = gtk::Button::new_from_icon_name(
            Some("media-playback-start"),
            gtk::IconSize::SmallToolbar,
        );
        play_button.connect_clicked(move |_| {
            // Add event handler to the event  when the button was clicked
            let pipeline = &pipeline;
            pipeline
                .set_state(gst::State::Playing)
                .expect("Unable to set the pipeline to `Playing` state");
        });

        let pause_button = gtk::Button::new_from_icon_name(
            Some("media-playback-pause"),
            gtk::IconSize::SmallToolbar,
        );
        let pipeline = playbin.clone();
        pause_button.connect_clicked(move |_| {
            // Add event handler to the event  when the button was clicked
            let pipeline = &pipeline;

            pipeline
                .set_state(gst::State::Paused)
                .expect("Unable to set the pipeline to the `Paused` state");
        });

        let stop_button = gtk::Button::new_from_icon_name(
            Some("media-playback_stop"),
            gtk::IconSize::SmallToolbar,
        );
        let pipeline = playbin.clone();
        stop_button.connect_clicked(move |_| {
            // Add event handler to the event when the button was clicked
            let pipeline = &pipeline;
            pipeline
                .seek_simple(
                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                    0 * gst::MSECOND,
                )
                .expect("Failed to seek to start");
            pipeline
                .set_state(gst::State::Paused)
                .expect("Unable to set the pipeline to the `Ready` state");
        });

        let slider =
            gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0 as f64, 100.0 as f64, 1.0);
        let pipeline = playbin.clone();

        // Add event handler to the event when the slider was moved
        let slider_update_signal_id = slider.connect_value_changed(move |slider| {
            let pipeline = &pipeline;

            let value = slider.get_value() as u64;
            if pipeline
                .seek_simple(
                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                    value * gst::SECOND,
                )
                .is_err()
            {
                eprintln!("Seeking to {} failed", value)
            }
        });
        slider.set_draw_value(false);

        // Query the position of the stream every 1 sec
        let pipeline = playbin.clone();
        let lslider = slider.clone();
        gtk::timeout_add_seconds(1, move || {
            let pipeline = &pipeline;
            let lslider = &lslider;

            if let Some(dur) = pipeline.query_duration::<gst::ClockTime>() {
                let seconds = dur / gst::SECOND;
                lslider.set_range(0.0, seconds.map(|v| v as f64).unwrap_or(0.0));
            }

            if let Some(pos) = pipeline.query_position::<gst::ClockTime>() {
                let seconds = pos / gst::SECOND;
                lslider.block_signal(&slider_update_signal_id);
                lslider.set_value(seconds.map(|v| v as f64).unwrap_or(0.0));
                lslider.unblock_signal(&slider_update_signal_id);
            }

            Continue(true)
        });

        // Pack UI in tool bar
        let controls = Box::new(Orientation::Horizontal, 0);
        controls.pack_start(&play_button, false, false, 0);
        controls.pack_start(&pause_button, false, false, 0);
        controls.pack_start(&stop_button, false, false, 0);
        controls.pack_start(&slider, true, true, 2);

        // Create video area
        let video_window = DrawingArea::new();
        let video_overlay = playbin
            .clone()
            .dynamic_cast::<gst_video::VideoOverlay>()
            .unwrap();
        video_window.connect_realize(move |video_window| {
            let video_overlay = &video_overlay;
            let gdk_window = video_window.get_window().unwrap();

            if !gdk_window.ensure_native() {
                println!("Can't create native window for widget");
                process::exit(-1);
            }

            let display_type_name = gdk_window.get_display().get_type().name();
            if display_type_name == "GdkX11Display" {
                extern "C" {
                    pub fn gdk_x11_window_get_xid(
                        window: *mut glib::object::GObject,
                    ) -> *mut c_void;
                }

                #[allow(clippy::cast_ptr_alignment)]
                unsafe {
                    // Call native API to obtain the window pointer
                    let xid = gdk_x11_window_get_xid(gdk_window.as_ptr() as *mut _);
                    // Set destination with the handler
                    video_overlay.set_window_handle(xid as usize);
                }
            } else {
                println!("Add support for display type {}", display_type_name);
                process::exit(-1);
            }
        });

        // Initialize stream list which shows the stream description available in the media file
        let streams_list = gtk::TextView::new();
        streams_list.set_editable(false);
        let pipeline_weak = playbin.downgrade();
        let streams_list_weak = glib::SendWeakRef::from(streams_list.downgrade());
        let bus = playbin.get_bus().unwrap();
        #[allow(clippy::single_match)]
        bus.connect_message(move |_, msg| match msg.view() {
            // application message is the message engineer can control
            // You can send arbitary message, you can see some messages are sent from post_app_message
            gst::MessageView::Application(application) => {
                let pipeline = match pipeline_weak.upgrade() {
                    Some(pipeline) => pipeline,
                    None => return,
                };
                let streams_list = match streams_list_weak.upgrade() {
                    Some(streams_list) => streams_list,
                    None => return,
                };

                if application.get_structure().map(|s| s.get_name()) == Some("tags-changed") {
                    let textbuf = streams_list
                        .get_buffer()
                        .expect("Couldn't get buffer from text_view");
                    analyze_streams(&pipeline, &textbuf);
                }
            }
            _ => (),
        });

        // Pack video region and stream info side bar
        let vbox = Box::new(Orientation::Horizontal, 0);
        vbox.pack_start(&video_window, true, true, 0);
        vbox.pack_start(&streams_list, false, false, 2);

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.pack_start(&controls, false, false, 0);
        main_box.pack_start(&vbox, true, true, 0);
        main_window.add(&main_box);
        main_window.set_default_size(640, 480);
        main_window.show_all();
    }

    fn analyze_streams(playbin: &gst::Element, textbuf: &gtk::TextBuffer) {
        textbuf.set_text("");
        add_streams_info(playbin, textbuf, "video");
        add_streams_info(playbin, textbuf, "audio");
        add_streams_info(playbin, textbuf, "text");
    }

    fn add_streams_info(playbin: &gst::Element, textbuf: &gtk::TextBuffer, stype: &str) {
        let propname: &str = &format!("n-{}", stype);
        let signame: &str = &format!("get-{}-tags", stype);
        // Stringify the stream information into gtk::TextBuffer
        match playbin.get_property(propname).unwrap().get() {
            Ok(Some(x)) => {
                for i in 0..x {
                    let tags = playbin.emit(signame, &[&i]).unwrap().unwrap();

                    if let Ok(Some(tags)) = tags.get::<gst::TagList>() {
                        textbuf.insert_at_cursor(&format!("{} stream {}:\n ", stype, i));

                        if let Some(codec) = tags.get::<gst::tags::VideoCodec>() {
                            textbuf.insert_at_cursor(&format!(
                                "    codec: {} \n",
                                codec.get().unwrap()
                            ));
                        }

                        if let Some(lang) = tags.get::<gst::tags::LanguageCode>() {
                            textbuf.insert_at_cursor(&format!(
                                "    language: {} \n",
                                lang.get().unwrap()
                            ));
                        }

                        if let Some(bitrate) = tags.get::<gst::tags::Bitrate>() {
                            textbuf.insert_at_cursor(&format!(
                                "    bitrate: {} \n",
                                bitrate.get().unwrap()
                            ));
                        }
                    }
                }
            }
            _ => {
                eprintln!("Could not get {}!", propname);
            }
        }
    }



    fn post_app_message(playbin: &gst::Element) {
        /*
        * API is under changing in new gstreamer-rs version.
        After the new version relased, you may need to code like below.
        let _ = playbin.post_message(&gst::message::Application::new(gst::Structure::new_empty(
            "tags-changed",
        )));
        */
        let _ = playbin.post_message(
            &gst::message::Message::new_application(gst::Structure::new_empty("tags-changed"))
                .build(),
        );
    }
}

#[cfg(feature = "tutorial5")]
fn main() {
    tutorial5::run();
}

#[cfg(not(feature = "tutorial5"))]
fn main() {
    println!("Please compile with --features tutorial5");
}
