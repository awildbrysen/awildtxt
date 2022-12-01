use std::ptr::addr_of;

use sdl2::sys::SDL_PushEvent;

pub fn push_sdl_text_input_event(event: sdl2::sys::SDL_TextInputEvent) {
    unsafe {
        let addr = addr_of!(event);
        SDL_PushEvent(addr as *mut sdl2::sys::SDL_Event);
    }
}

pub fn create_sdl_text_input_event(s: &str) -> sdl2::sys::SDL_TextInputEvent {
    let mut v: [i8; 32] = [0; 32];

    for i in 0..32 {
        v[i] = if let Some(c) = s.chars().nth(i) {
            c as i8
        } else {
            0
        }
    }

    sdl2::sys::SDL_TextInputEvent {
        type_: sdl2::sys::SDL_EventType::SDL_TEXTINPUT as u32,
        windowID: 0, 
        timestamp: 0,
        text: v
    }
}
