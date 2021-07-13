use std::{sync::mpsc, thread::sleep, time::Duration};

use souvlaki::{MediaControlEvent, MediaControls};
use souvlaki::{MediaMetadata, MediaPlayback};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TestApp {
    playing: bool,
    song_index: u8,
}

fn main() {
    let event_loop = EventLoop::new();
    #[allow(unused_variables)]
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_os = "windows")]
    let mut controls = {
        use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

        let handle = match window.raw_window_handle() {
            RawWindowHandle::Windows(h) => h,
            _ => unreachable!(),
        };
        MediaControls::for_window(handle).unwrap()
    };
    #[cfg(target_os = "macos")]
    let mut controls = MediaControls::new();
    #[cfg(target_os = "linux")]
    let mut controls =
        MediaControls::new_with_name("souvlaki-example", "Souvlaki media keys example");

    #[cfg(all(
        not(target_os = "windows"),
        not(target_os = "macos"),
        not(target_os = "linux")
    ))]
    let mut controls = MediaControls::new();

    let (tx, rx) = mpsc::sync_channel(32);
    let mut app = TestApp {
        playing: true,
        song_index: 0,
    };

    controls.attach(move |e| tx.send(e).unwrap()).unwrap();
    controls.set_playback(MediaPlayback::Playing).unwrap();
    controls
        .set_metadata(MediaMetadata {
            title: Some("When The Sun Hits"),
            album: Some("Souvlaki"),
            artist: Some("Slowdive"),
            cover_url: Some("https://c.pxhere.com/photos/34/c1/souvlaki_authentic_greek_greek_food_mezes-497780.jpg!d"),
        })
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let mut change = false;

                for event in rx.try_iter() {
                    match event {
                        MediaControlEvent::Toggle => app.playing = !app.playing,
                        MediaControlEvent::Play => app.playing = true,
                        MediaControlEvent::Pause => app.playing = false,
                        MediaControlEvent::Next => app.song_index = app.song_index.wrapping_add(1),
                        MediaControlEvent::Previous => {
                            app.song_index = app.song_index.wrapping_sub(1)
                        }
                    }
                    change = true;
                }
                sleep(Duration::from_millis(50));

                if change {
                    controls
                        .set_playback(if app.playing {
                            MediaPlayback::Playing
                        } else {
                            MediaPlayback::Paused
                        })
                        .unwrap();

                    eprintln!(
                        "{} (song {})",
                        if app.playing { "Playing" } else { "Paused" },
                        app.song_index
                    );
                }
            }
            _ => (),
        }
    });
}
