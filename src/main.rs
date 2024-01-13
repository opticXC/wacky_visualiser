#[feature(ascii_char)]
pub mod libs;
use std::ops::Deref;
use std::os::raw::{c_char, c_schar};
use std::{f64::consts::PI, os::raw::c_void};

use concrete_fft::{
    c64,
    ordered::{Method, Plan},
};

use raylib_ffi::{
    self,
    colors::{LIGHTGRAY, RED},
    Rectangle,
};

fn main() {
    let mut width = 800;
    let mut height = 450;

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <path to audio file>", args[0]);
        std::process::exit(1);
    }
    let mut path = args[1].deref().to_string();
    path += "\0";
    let audio_title = path.split("/").last().unwrap();

    let mut title = String::from(audio_title);
    title += "\0";

    unsafe {
        raylib_ffi::InitWindow(width, height, title.as_ptr() as *const i8);
        raylib_ffi::InitAudioDevice();
        //raylib_ffi::SetTargetFPS(60);

        println!("FILE: {:?}", *(path.as_ptr() as *const i8));

        let audio = raylib_ffi::LoadMusicStream(path.as_ptr() as *const i8);
        raylib_ffi::PlayMusicStream(audio);

        let mut playing = true;
        let mut draw_fps = false;
        let mut v_mode = VISUAL_MODE::FftSpectrum;

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
                    VISUAL_MODE::WaveForm => VISUAL_MODE::FftSpectrum,
                    VISUAL_MODE::FftSpectrum => VISUAL_MODE::WaveForm,
                }
            }

            if playing {
                raylib_ffi::UpdateMusicStream(audio);
            }

            raylib_ffi::BeginDrawing();
            raylib_ffi::ClearBackground(raylib_ffi::colors::RAYWHITE);
            raylib_ffi::DrawText(
                format!("Currently Playing {}\0", audio_title).as_ptr() as *const i8,
                20,
                10,
                20,
                raylib_ffi::colors::LIGHTGRAY,
            );
            raylib_ffi::DrawRectangle(20, 40, width - 40, 20, LIGHTGRAY);
            raylib_ffi::DrawRectangle(20, 40, bar as i32, 20, raylib_ffi::colors::MAROON);

            let visualizer_box = Rectangle {
                x: 20.0,
                y: 80.0,
                width: width as f32 - 40.0,
                height: height as f32 - 100.0,
            };

            raylib_ffi::DrawRectangleRec(visualizer_box, raylib_ffi::colors::LIGHTGRAY);


            match v_mode {
                VISUAL_MODE::WaveForm => {
                    draw_waveform(
                        visualizer_box.x as i32,
                        visualizer_box.y as i32,
                        visualizer_box.width as i32,
                        visualizer_box.height as i32,
                        RED,
                    );
                },

                VISUAL_MODE::FftSpectrum =>{ 
                    if playing{
                        update_freq_buffer();
                    }

                    draw_fft(
                        visualizer_box.x as i32,
                        visualizer_box.y as i32,
                        visualizer_box.width as i32,
                        visualizer_box.height as i32,
                        RED,
                    );
                },
            };

            if draw_fps {
                raylib_ffi::DrawFPS(width - 100, height - 40);
            }

            raylib_ffi::EndDrawing();
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct Frame {
    left: f32,
    right: f32,
}

const FFT_SIZE: usize = 512;

static mut FFT_RAW_IN: [f64; FFT_SIZE] = [0.0; FFT_SIZE];

static mut FFT_RAW_OUT: [c64; FFT_SIZE] = [c64::new(0.0, 0.0); FFT_SIZE];

static mut MAX_AMP: f64 = 0.0;

unsafe extern "C" fn process_audio(data: *mut c_void, samples_count: u32) {
    if samples_count as usize != FFT_SIZE{
        return;
    }

    let mut frames = data as *mut Frame;
    for i in 0..samples_count as usize {
        FFT_RAW_IN[i] = ((*frames).left + (*frames).right) as f64;
        FFT_RAW_IN[i] = FFT_RAW_IN[i] / 2.0;

        frames = frames.add(1);
    }
    
    // apply hann window on raw input

    for i in 0..FFT_SIZE{
        let t:f64 = i as f64 / (FFT_SIZE -1) as f64;
        let hann:f64 = 0.5 - (0.5 * (2.0*PI*t).cos());
        FFT_RAW_IN[i] = FFT_RAW_IN[i] * hann;
    }



    fft(FFT_RAW_IN.as_ptr(), 1, FFT_RAW_OUT.as_mut_ptr(), FFT_SIZE);

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
    return (real * real + im * im).sqrt();
}

static mut WAVEFORM_BUFFER: [f64; FFT_SIZE] = [0.0; FFT_SIZE];

unsafe fn draw_waveform(x: i32, y: i32, w: i32, h: i32, color: raylib_ffi::Color) {
    let bar_width = w / FFT_SIZE as i32;
    let mut current = 0.0;
    for inp in FFT_RAW_OUT.iter() {
        current += amplitude(inp).abs();
    }
    current /= FFT_SIZE as f64;
    for i in 0..FFT_SIZE - 1 {
        WAVEFORM_BUFFER[i] = WAVEFORM_BUFFER[i + 1];
    }

    WAVEFORM_BUFFER[FFT_SIZE - 1] = current.abs();
    for i in 0..FFT_SIZE - 1 {
        let height = (h as f64 * WAVEFORM_BUFFER[i] * 0.1f64).floor() as i32;
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


const VB_SIZE:usize = FFT_SIZE/2;
static mut VISUAL_BUFFER:[f64;VB_SIZE] = [0.0; VB_SIZE];

const CRUSHER:usize = 4;

unsafe fn update_freq_buffer(){
    for i in 0..VB_SIZE{
        let mut amp = 0.0f64;
        
        if  i>(CRUSHER/2) && i+(CRUSHER/2) < VB_SIZE {
            for j in -(CRUSHER as i32/2)..(CRUSHER/2) as i32{
                amp += amplitude(&FFT_RAW_OUT[ (i as i32 + j) as usize ]);
            }
            amp /= CRUSHER as f64;
        }else{
            amp = amplitude(&FFT_RAW_OUT[i]);
        }

        VISUAL_BUFFER[i] = amp/MAX_AMP;
    }
}

unsafe fn draw_fft(x: i32, y: i32, w: i32, h: i32, color: raylib_ffi::Color) {
    let bar_width = w as f64 / VB_SIZE as f64;
    
    for i in 0..VB_SIZE{
        let c = VISUAL_BUFFER[i];
        let height = h as f64 * c;

        raylib_ffi::DrawRectangle(
            x + (bar_width * i as f64).floor() as i32,
            y + h - height.floor() as i32,
            bar_width.floor() as i32,
            height.floor() as i32,
            color
        );
    }

}

enum VISUAL_MODE {
    WaveForm,
    FftSpectrum,
}
