extern crate gstreamer as gst;

use gst::prelude::*;
use std::io;
use std::io::Write;

// Custom data type representing application state
struct PlayerState {
    playbin: gst::Element,
    playing: bool,
    terminate: bool,
    seek_enabled: bool,
    first_seek_done: bool,
    duration: gst::ClockTime,
}

fn main() {
    gst::init().unwrap();

    // Create an element.
    let playbin = gst::ElementFactory::make("playbin", Some("playbin"))
        .expect("Failed to create playbin element");

    // Set the URI to play
    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
    playbin
        .set_property("uri", &uri)
        .expect("Can't set uri property on playbin");

    // Start playing
    playbin
        .set_state(gst::State::Playing)
        .expect("Unable to set the playbin to the playing state");

    // Monitor messages until player_state.terminate became true
    let bus = playbin.get_bus().unwrap();
    let mut player_state = PlayerState {
        playbin,
        playing: false,
        terminate: false,
        seek_enabled: false,
        first_seek_done: false,
        duration: gst::CLOCK_TIME_NONE,
    };
    while !player_state.terminate {
        let msg = bus.timed_pop(100 * gst::MSECOND);
        match msg {
            Some(msg) => handle_message(&mut player_state, &msg),
            None => {
                if player_state.playing {
                    // Update position by query
                    let position = player_state
                        .playbin
                        .query_position::<gst::ClockTime>()
                        .expect("Could not query current position");
                    // Query duration if player_state.duration was default value
                    if player_state.duration == gst::CLOCK_TIME_NONE {
                        player_state.duration = player_state
                            .playbin
                            .query_duration()
                            .expect("Could not query current duration");
                    }
                    // Peform a first seek because it begins from 0. I want to play 30s - 35s
                    if !player_state.first_seek_done && player_state.seek_enabled {
                        player_state
                            .playbin
                            .seek_simple(
                                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                30 * gst::SECOND,
                            )
                            .expect("Failed to seek");
                        player_state.first_seek_done = true;
                    } else {
                        // Printing progress and duration of the video
                        print!("\rPosition {} / {}", position, player_state.duration);
                        io::stdout().flush().unwrap();

                        // Perform a seek if the position was over 30s
                        if player_state.seek_enabled && position > 35 * gst::SECOND {
                            println!("\n Reached 5s performing seek...");
                            player_state
                                .playbin
                                .seek_simple(
                                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                    30 * gst::SECOND,
                                )
                                .expect("Failed to seek");
                        }
                    }
                }
            }
        }
    }

    // Cleaning up
    player_state
        .playbin
        .set_state(gst::State::Null)
        .expect("Unable to set playbin to the Null state");
}

fn handle_message(player_state: &mut PlayerState, msg: &gst::Message) {
    match msg.view() {
        gst::MessageView::Error(err) => {
            println!(
                "Error received from element {:?}: {} ({:?})",
                err.get_src().map(|s| s.get_path_string()),
                err.get_error(),
                err.get_debug()
            );
            player_state.terminate = true;
        }
        gst::MessageView::Eos(..) => {
            println!("EOS");
            player_state.terminate = true;
        }
        gst::MessageView::DurationChanged(_) => {
            player_state.duration = gst::CLOCK_TIME_NONE;
        }
        gst::MessageView::StateChanged(state_changed) => {
            if state_changed
                .get_src()
                .map(|s| s == player_state.playbin)
                .unwrap_or(false)
            {
                let new_state = state_changed.get_current();
                let old_state = state_changed.get_old();

                println!(
                    "Pipeline state changed from {:?} to {:?}",
                    old_state, new_state
                );

                // If player state became first, we need to check that stream being seekable
                // Some streams can be unseekale.
                player_state.playing = new_state == gst::State::Playing;
                if player_state.playing {
                    let mut seeking = gst::Query::new_seeking(gst::Format::Time);
                    if player_state.playbin.query(&mut seeking) {
                        let (seekable, start, end) = seeking.get_result();
                        player_state.seek_enabled = seekable;
                        if seekable {
                            println!("Seeking is ENABLED from {:?} to {:?}", start, end)
                        } else {
                            println!("Seeking is DISABLED for this stream.")
                        }
                    } else {
                        eprintln!("Seeking query failed.")
                    }
                }
            }
        }
        _ => (),
    }
}
