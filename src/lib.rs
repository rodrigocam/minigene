pub extern crate bracket_lib;
#[macro_use]
extern crate pushdown_automaton_macro;
#[macro_use]
extern crate specs_declaration;
pub extern crate game_features;
pub extern crate specs;
#[macro_use]
extern crate specs_derive;
pub extern crate hibitset;
pub extern crate shrev;
#[macro_use]
extern crate derive_new;

#[cfg(feature = "terminal")]
#[macro_use]
extern crate crossterm;

pub use bracket_lib::prelude::*;
pub use game_features::*;
pub use game_time::*;
pub use hibitset::BitSet;
pub use shrev::*;
pub use specs::prelude::*;
pub use specs::storage::MaskedStorage;
pub use specs::world::EntitiesRes;
pub use stopwatch::*;

// macro re-export
pub use derive_new::*;
pub use specs_declaration::*;
pub use specs_derive::*;

mod dispatcher;

pub use crate::dispatcher::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::hash::Hash;
use std::fmt::Debug;

pub type MiniDispatcher = Box<dyn UnifiedDispatcher + 'static>;

state_machine!(StateMachine; State; world: &mut World, dispatcher: &mut Box<dyn UnifiedDispatcher + 'static>, ctx: &mut BTerm);

pub fn mini_loop<I: State + 'static>(
    world: &mut World,
    dispatcher: &mut Box<dyn UnifiedDispatcher + 'static>,
    ctx: &mut BTerm,
    init_state: I,
) {
    let mut state_machine = StateMachine::new(init_state);
    state_machine.start(world, dispatcher, ctx);
    while state_machine.is_running() {
        mini_frame(world, dispatcher, ctx, &mut state_machine);
    }
}

pub fn mini_frame(
    world: &mut World,
    dispatcher: &mut Box<dyn UnifiedDispatcher + 'static>,
    ctx: &mut BTerm,
    state_machine: &mut StateMachine,
) {
    #[cfg(not(feature = "wasm"))]
    world.get_mut::<Stopwatch>().unwrap().start();

    let input = INPUT.lock();
    for key in input.key_pressed_set().iter() {
        world
            .fetch_mut::<EventChannel<VirtualKeyCode>>()
            .single_write(*key);
    }
    dispatcher.run_now(world);
    state_machine.update(world, dispatcher, ctx);
    world.maintain();

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::sleep(std::time::Duration::from_millis(8));

    #[cfg(not(feature = "wasm"))]
    let elapsed = world.fetch::<Stopwatch>().elapsed();
    #[cfg(feature = "wasm")]
    let elapsed = std::time::Duration::from_millis(16);
    let time = world.get_mut::<Time>().unwrap();
    time.increment_frame_number();
    time.set_delta_time(elapsed);
    #[cfg(not(feature = "wasm"))]
    {
        let stopwatch = world.get_mut::<Stopwatch>().unwrap();
        stopwatch.stop();
        stopwatch.restart();
    }
}

pub fn mini_init(
    width: u32,
    height: u32,
    name: &str,
    spritesheet: Option<SpriteSheet>,
    dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    mut world: World,
    //mut dispatcher_builder: DispatcherBuilder<'static, 'static>,
) -> (World, Box<dyn UnifiedDispatcher + 'static>, BTerm) {
    #[cfg(feature = "terminal")]
    std::panic::set_hook(Box::new(|panic_info| {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();
        let location = panic_info.location().unwrap();
        println!("Panic occured at {}:{}", location.file(), location.line());
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("Panic occured: {:?}", s);
        }
        //execute!(std::io::stdout(), crossterm::terminal::EnableLineWrap);
    }));
    #[cfg(feature = "wasm")]
    web_worker::init_panic_hook();
    let mut context = BTermBuilder::new();
        #[cfg(feature = "opengl")]
        {
            if let Some(ss) = spritesheet {
                context = context.with_sprite_sheet(ss);
                context = context.with_sprite_console(width, height, 0);
            } else {
                println!("Using opengl mode without a spritesheet!");
            }
        }
    #[cfg(not(feature = "opengl"))]
    {
        context = context.with_simple_console(width, height, "terminal8x8.png");
    }

    let context = context.with_font("terminal8x8.png", 8, 8)
        .with_title(name)
        .with_vsync(false)
        .with_advanced_input(true)
        .build()
        .expect("Failed to build BTerm context.");
    //#[cfg(feature = "wasm")]
    //{
    //    dispatcher_builder = dispatcher_builder.with_pool(Arc::new(web_worker::default_thread_pool(None).expect("Failed to create web worker thread pool")));
    //}
    //let mut dispatcher = dispatcher_builder.build();
    //dispatcher.setup(&mut world);
    world.insert(EventChannel::<VirtualKeyCode>::new());
    world.insert(Stopwatch::new());
    world.insert(Time::default());

    //#[cfg(not(feature = "wasm"))]
    //{
    //    std::panic::set_hook(Box::new(|i| {
    //        if let Some(s) = i.payload().downcast_ref::<&str>() {
    //            eprintln!("panic occurred: {:?}", s);
    //        } else {
    //            eprintln!("panic occurred");
    //        }
    //        eprintln!("Occured in file {} line {}:{}", i.location().unwrap().file(), i.location().unwrap().line(), i.location().unwrap().column());
    //        std::fs::write("/tmp/err", "WE CRASHED").unwrap();
    //    }));
    //}

    (world, dispatcher, context)
}

/*#[cfg(test)]
mod tests {
    use crate::CollisionMap;
    #[test]
    fn collmap() {
        let mut m = CollisionMap::new(3, 3);
        m.set(2, 2);
        assert!(m.is_set(2, 2));
        assert_eq!(m.index_of(2, 2), 8);
        assert_eq!(m.position_of(8), (2, 2));
    }
}*/
