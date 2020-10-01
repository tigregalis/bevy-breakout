use bevy::prelude::*;

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .add_startup_system(add_pets.system())
            // .add_startup_system(add_pets.system())
            .add_resource(Timer::from_seconds(2.0, true))
            .add_startup_system(add_people.system())
            // .add_system(hello_world.system())
            // .add_system(greet_pets.system())
            .add_system(greet_people.system());
    }
}

struct Person;
// struct Pet;
struct Name(String);

// struct GreetTimer(Timer);

// fn hello_world() {
//     println!("hello world!");
// }

fn add_people(mut commands: Commands) {
    commands
        .spawn((Person, Name("Alice".to_string())))
        .spawn((Person, Name("Bob".to_string())))
        .spawn((Person, Name("Charlie".to_string())));
}

// fn add_pets(mut commands: Commands) {
//     commands
//         .spawn((Pet, Name("Dino".to_string())))
//         .spawn((Pet, Name("Eddy".to_string())))
//         .spawn((Pet, Name("Fido".to_string())));
// }

// fn greet_people(time: Res<Time>, mut timer: ResMut<Timer>, _person: &Person, name: &Name) {
//     // println!("time since last update is {}", time.delta_seconds);
//     timer.tick(time.delta_seconds);
//     if timer.finished {
//         println!("hello {}!", name.0);
//     }
// }

fn greet_people(time: Res<Time>, mut timer: ResMut<Timer>, mut query: Query<(&Person, &Name)>) {
    // println!("time since last update is {}", time.delta_seconds);
    timer.tick(time.delta_seconds);
    if timer.finished {
        for (_person, name) in &mut query.iter() {
            println!("hello {}!", name.0);
        }
    }
}

// fn greet_pets(_pet: &Pet, name: &Name) {
//     println!("well hello there little {}!", name.0)
// }

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(HelloPlugin)
        .run();
}