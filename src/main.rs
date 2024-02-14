pub mod libs;
use std::ops::Deref;

use std::{f64::consts::PI, os::raw::c_void};

use concrete_fft::c64;

use libs::catppuccin;
use raylib_ffi::{self, Rectangle};

fn main() {
    let mut width = 800;
    let mut height = 450;

    let mut theme = libs::catppuccin::mocha(libs::catppuccin::Accents::Flamingo);

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        let help_string = format!("{} <path to audio file>", args[0]);
        println!("{}", help_string);
        std::process::exit(1);
    }
    let mut path = String::new();

    let mut theme_idx = 0;
    while theme_idx < args.len() {
        match args[theme_idx].as_str() {
            "--accent" => {
                theme_idx += 1;
                let theme_name = &args[theme_idx];
                let accent = match theme_name.to_lowercase().as_str() {
                    "rosewater" => catppuccin::Accents::Rosewater,
                    "flamingo" => catppuccin::Accents::Flamingo,
                    "pink" => catppuccin::Accents::Pink,
                    "mauve" => catppuccin::Accents::Mauve,
                    "red" => catppuccin::Accents::Red,
                    "maroon" => catppuccin::Accents::Maroon,
                    "peach" => catppuccin::Accents::Peach,
                    "yellow" => catppuccin::Accents::Yellow,
                    "green" => catppuccin::Accents::Green,
                    "teal" => catppuccin::Accents::Teal,
                    "sky" => catppuccin::Accents::Sky,
                    "saphire" => catppuccin::Accents::Saphire,
                    "blue" => catppuccin::Accents::Blue,
                    "lavender" => catppuccin::Accents::Lavender,

                    _ => catppuccin::Accents::Flamingo,
                };
                theme = catppuccin::mocha(accent);
            }
            _ => path = args[theme_idx].deref().to_string(),
        }
        theme_idx += 1;
    }

    // C strings should be null ternimated
    // you need to add that yourselves
    // wish i knew why....
    path += "\0";
    let audio_title = path.split("/").last().unwrap();

    let mut title = String::from(audio_title);
    title += "\0";

    unsafe {
        raylib_ffi::InitWindow(width, height, title.as_ptr() as *const i8);
        raylib_ffi::InitAudioDevice();
        raylib_ffi::SetTargetFPS(60);

        println!("FILE: {:?}", *(path.as_ptr() as *const i8));

        let audio = raylib_ffi::LoadMusicStream(path.as_ptr() as *const i8);
        raylib_ffi::PlayMusicStream(audio);

        let mut playing = true;
        let mut draw_fps = false;
        let mut v_mode = VisualMode::FftSpectrum;

        // attaching a callback to the stream, runs everytime the stream is updated;
        // the callback needs 2 args
        // 1 is a void ptr to the raw output amplitudes of the wave
        // standard miniaudio stuff,
        //
        // 2 is number of frames in the current sample
        // handle this properly !!!
        raylib_ffi::AttachAudioStreamProcessor(audio.stream, Some(process_audio));

        raylib_ffi::PlayMusicStream(audio);

        let len = raylib_ffi::GetMusicTimeLength(audio);

        while !raylib_ffi::WindowShouldClose() {
            width = raylib_ffi::GetScreenWidth();
            height = raylib_ffi::GetScreenHeight();
            let pos = raylib_ffi::GetMusicTimePlayed(audio);
            let bar = (pos as f32 / len as f32) * width as f32;

            if raylib_ffi::IsKeyPressed(raylib_ffi::enums::KeyboardKey::Space as i32) {
                playing = !playing;
            }
            if raylib_ffi::IsKeyPressed(raylib_ffi::enums::KeyboardKey::Right as i32) {
                raylib_ffi::SeekMusicStream(audio, pos + 5.0);
            }

            if raylib_ffi::IsKeyPressed(raylib_ffi::enums::KeyboardKey::Left as i32) {
                raylib_ffi::SeekMusicStream(audio, pos - 5.0);
            }

            if raylib_ffi::IsKeyPressed(raylib_ffi::enums::KeyboardKey::F as i32) {
                draw_fps = !draw_fps;
            }

            if raylib_ffi::IsKeyPressed(raylib_ffi::enums::KeyboardKey::V as i32) {
                v_mode = match v_mode {
                    VisualMode::WaveForm => VisualMode::FftSpectrum,
                    VisualMode::FftSpectrum => VisualMode::WaveForm,
                }
            }

            if playing {
                raylib_ffi::UpdateMusicStream(audio);
            }

            raylib_ffi::BeginDrawing();
            raylib_ffi::ClearBackground(theme.background);
            raylib_ffi::DrawText(
                format!("Currently Playing {}\0", audio_title).as_ptr() as *const i8,
                20,
                10,
                20,
                theme.text,
            );
            raylib_ffi::DrawRectangle(20, 40, width - 40, 20, theme.foreground);
            raylib_ffi::DrawRectangle(20, 40, bar as i32, 20, theme.accent);

            let visualizer_box = Rectangle {
                x: 20.0,
                y: 80.0,
                width: width as f32 - 40.0,
                height: height as f32 - 100.0,
            };

            raylib_ffi::DrawRectangleRec(visualizer_box, theme.foreground);

            match v_mode {
                VisualMode::WaveForm => {
                    if playing {
                        update_waveform_buffer();
                    }
                    draw_waveform(
                        visualizer_box.x as i32,
                        visualizer_box.y as i32,
                        visualizer_box.width as i32,
                        visualizer_box.height as i32,
                        theme.accent,
                    );
                }

                VisualMode::FftSpectrum => {
                    if playing {
                        update_freq_buffer();
                    }

                    draw_fft(
                        visualizer_box.x as i32,
                        visualizer_box.y as i32,
                        visualizer_box.width as i32,
                        visualizer_box.height as i32,
                        theme.accent,
                    );
                }
            };

            if draw_fps {
                raylib_ffi::DrawFPS(width - 100, height - 40);
            }

            raylib_ffi::EndDrawing();
        }
    }
}

/*
 * Exists solely for helping in convertion from void ptrs
 * The Frames are 32 bit floats
 * alternating between the left and right sterio channel
 */
#[derive(Debug)]
#[repr(C)]
struct Frame {
    left: f32,
    right: f32,
}

/* FFT size should be in powers of 2
 * ignore anything smaller than 256, it wount be a good time....
 */
const FFT_SIZE: usize = 512;

static mut FFT_RAW_IN: [f64; FFT_SIZE] = [0.0; FFT_SIZE];
static mut FFT_RAW_OUT: [c64; FFT_SIZE] = [c64::new(0.0, 0.0); FFT_SIZE];

static mut MAX_AMP: f64 = 0.0;

unsafe extern "C" fn process_audio(data: *mut c_void, samples_count: u32) {
    if samples_count as usize != FFT_SIZE {
        return;
    }

    let mut frames = data as *mut Frame;
    for i in 0..samples_count as usize {
        FFT_RAW_IN[i] = ((*frames).left + (*frames).right) as f64;
        FFT_RAW_IN[i] = FFT_RAW_IN[i] / 2.0;

        frames = frames.add(1);
    }

    // starting smoothening stuff.... i dont get this very well

    // apply hann window on raw input
    for i in 0..FFT_SIZE {
        let t: f64 = i as f64 / (FFT_SIZE - 1) as f64;
        let hann: f64 = 0.5 - (0.5 * (2.0 * PI * t).cos());
        FFT_RAW_IN[i] = FFT_RAW_IN[i] * hann;
    }

    // actual fft
    fft(FFT_RAW_IN.as_ptr(), 1, FFT_RAW_OUT.as_mut_ptr(), FFT_SIZE);
    
    // update [MAX_AMP] based on current frame
    for &c in FFT_RAW_OUT.iter() {
        let amp = amplitude(&c);
        if MAX_AMP < amp {
            MAX_AMP = amp;
        }
    }
}

unsafe fn fft(input: *const f64, stride: usize, output: *mut c64, n: usize) {
    if n <= 0 {
        return;
    }

    if n == 1 {
        *output = c64::new(*input, 0.0);
    }

    fft(input, stride * 2, output, n / 2);
    fft(input.add(stride), stride * 2, output.add(n / 2), n / 2);

    for k in 0..n / 2 {
        let t: f64 = k as f64 / n as f64;
        let v: c64 = c64::exp(-2.0 * c64::i() * PI * t) * *(output.add(k + n / 2));
        let e: c64 = *output.add(k);
        *output.add(k) = e + v;
        *output.add(k + n / 2) = e - v;
    }
}

unsafe fn amplitude(sample: &c64) -> f64 {
    let real = sample.re;
    let im = sample.im;

    (real * real + im * im).sqrt()
}

static mut WAVEFORM_BUFFER: [f64; FFT_SIZE] = [0.0; FFT_SIZE];

unsafe fn update_waveform_buffer() {
    let mut current = 0.0;
    for inp in FFT_RAW_IN.iter() {
        current += inp;
    }
    current /= FFT_SIZE as f64;
    for i in 0..FFT_SIZE - 1 {
        WAVEFORM_BUFFER[i] = WAVEFORM_BUFFER[i + 1];
    }

    WAVEFORM_BUFFER[FFT_SIZE - 1] = current.abs();
}

unsafe fn draw_waveform(x: i32, y: i32, w: i32, h: i32, color: raylib_ffi::Color) {
    let bar_width = w / FFT_SIZE as i32;
    for i in 0..FFT_SIZE - 1 {
        let height = (h as f64 * WAVEFORM_BUFFER[i]).floor() as i32;
        raylib_ffi::DrawRectangle(
            x + i as i32 * bar_width as i32,
            y + h / 2 - height / 2,
            bar_width as i32,
            height,
            color,
        );
    }
}

//const FREQ_INDEXES: [usize; FREQ_SIZE] = [1,2,4,8,16,32,48,50,56,64,70,78,84,90 ,96,100,104,108, 120];

const VB_SIZE: usize = FFT_SIZE / 2;
const BUFFER_SIZE: usize = 3;

static mut VISUAL_BUFFER: [[f64; VB_SIZE]; BUFFER_SIZE] = [[0.0; VB_SIZE]; BUFFER_SIZE];

// why did i name this crusher....
// anyways, keep this a multiple of 2
const CRUSHER: usize = 2;

unsafe fn update_freq_buffer() {
    for i in 0..BUFFER_SIZE - 1 {
        VISUAL_BUFFER[i] = VISUAL_BUFFER[i + 1];
    }

    for i in 0..VB_SIZE {
        let mut amp = 0.0f64;

        if i > (CRUSHER / 2) && i + (CRUSHER / 2) < VB_SIZE {
            for j in -(CRUSHER as i32 / 2)..(CRUSHER / 2) as i32 {
                amp += amplitude(&FFT_RAW_OUT[(i as i32 + j) as usize]);
            }
            amp /= CRUSHER as f64;
        } else {
            amp = amplitude(&FFT_RAW_OUT[i]);
        }

        VISUAL_BUFFER[BUFFER_SIZE - 1][i] = amp / MAX_AMP;
    }
}

// who cares about the rest of the spectrum
// 14000 * 512 / 48000 = 149.3

const DRAW_UPTO: usize = 150;

unsafe fn draw_fft(x: i32, y: i32, w: i32, h: i32, color: raylib_ffi::Color) {
    let bar_width = w as f64 / DRAW_UPTO as f64;

    for i in 0..DRAW_UPTO {
        let mut c = 0.0f64;
        for j in 0..BUFFER_SIZE {
            c += VISUAL_BUFFER[j][i];
        }
        c /= BUFFER_SIZE as f64;

        let mut height = h as f64 * c;
        let modifier = 2.0;
        height *= modifier;
        height = height.min(h as f64);

        raylib_ffi::DrawRectangle(
            x + (bar_width * i as f64).floor() as i32,
            y + h - height.floor() as i32,
            bar_width.floor() as i32,
            height.floor() as i32,
            color,
        );
    }
}

enum VisualMode {
    WaveForm,
    FftSpectrum,
}
