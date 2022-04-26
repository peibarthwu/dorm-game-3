#![allow(dead_code)]

use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::Camera;
use frenderer::renderer::flat::SingleRenderState as FFlat;
use frenderer::renderer::skinned::SingleRenderState as FSkinned;
use frenderer::renderer::sprites::SingleRenderState as FSprite;
use frenderer::renderer::textured::SingleRenderState as FTextured;
use frenderer::types::*;
use frenderer::{Engine, FrendererSettings, Key, Result, SpriteRendererSettings};
use scene3d::types::{self, *};
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;
const SPEED: f32 = 0.25;
const ROOMSIZE: f32 = 50.0;
const COLLISION_RADIUS: f32 = 10.0;
const SCALE: f32 = 10.0;


#[derive(Clone)]

pub struct GameState {
    pub current_room: usize, //index of room in rooms
    pub max_rooms: usize,
    pub key_index: usize,
    //pub inventory: Vec<GameObject>,
    pub rooms: Vec<Room>,
    pub is_finished: bool,
}

//game states: main screen, instruction, playing, finalscreen
//make one room first with lockedchest
//if goes in through a door, generate a new room add it to rooms and
//have a checker that makes sure rooms.len < max_room and rooms.len != key_index
//have the character (collision?) get the key and put it in inventory
//go through rooms until main room

struct GameObject {
    trf: Similarity3,
    model: Rc<frenderer::renderer::skinned::Model>,
    animation: AnimRef,
    state: AnimationState,
}
impl GameObject {
    fn tick_animation(&mut self) {
        self.state.tick(DT);
    }
}
struct Sprite {
    trf: Isometry3,
    tex: frenderer::assets::TextureRef,
    cel: Rect,
    size: Vec2,
}
impl Sprite{
    pub fn move_by(&mut self, vec: Vec3) {
        self.trf.append_translation(vec);
    }
    pub fn check_collisions(&mut self, door: Door){
        let door_worldspace = get_trf(door.direction, ROOMSIZE, SCALE);
        if (self.trf.translation.x - door_worldspace.translation.x).abs() <= self.size.x
        && (self.trf.translation.z - door_worldspace.translation.z).abs() <= self.size.y
        {
            println!("door hit");
        }
    }
}
struct World {
    camera: Camera,
    things: Vec<GameObject>,
    sprites: Vec<Sprite>,
    flats: Vec<Flat>,
    textured: Vec<Textured>,
    door1:Textured,
    door2:Textured,
    door3:Textured,
    door4:Textured,
    state: GameState,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
struct Textured {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    name: String
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        let yaw = input.key_axis(Key::Q, Key::W) * PI / 4.0 * DT as f32;
        let pitch = input.key_axis(Key::A, Key::S) * PI / 4.0 * DT as f32;
        let roll = input.key_axis(Key::Z, Key::X) * PI / 4.0 * DT as f32;
        let dscale = input.key_axis(Key::E, Key::R) * 1.0 * DT as f32;
        let rot = Rotor3::from_euler_angles(roll, pitch, yaw);


        for obj in self.things.iter_mut() {
            obj.trf.append_rotation(rot);
            obj.trf.scale = (obj.trf.scale + dscale).max(0.01);
            // dbg!(obj.trf.rotation);
            obj.tick_animation();
        }
        for s in self.sprites.iter_mut() {
            // s.trf.append_rotation(rot);
            // s.size.x += dscale;
            // s.size.y += dscale;
            if input.is_key_down(Key::W){
                s.move_by(Vec3::new(0.0, 0.0, -SPEED));
            }
            if input.is_key_down(Key::S){
                s.move_by(Vec3::new(0.0, 0.0, SPEED));
            }
            if input.is_key_down(Key::A){
                s.move_by(Vec3::new(-SPEED, 0.0, 0.0));
            }
            if input.is_key_down(Key::D){
                s.move_by(Vec3::new(SPEED, 0.0, 0.0));
            }
            for door in self.state.rooms[self.state.current_room].doors.iter() {
                s.check_collisions(*door);
            }

        }
        for m in self.flats.iter_mut() {
            m.trf.append_rotation(rot);
            m.trf.scale += dscale;
        }
        for m in self.textured.iter_mut() {
            m.trf.append_rotation(rot);
            m.trf.scale += dscale;
        }
        let camera_drot = input.key_axis(Key::Left, Key::Right) * PI / 4.0 * DT as f32;
        self.camera
            .transform
            .prepend_rotation(Rotor3::from_rotation_xz(camera_drot));
    }
    fn render(
        &mut self,
        _a: &mut frenderer::assets::Assets,
        rs: &mut frenderer::renderer::RenderState,
    ) {
        rs.set_camera(self.camera);
        //put doors in the correct positions
        let door_list = &self.state.rooms[self.state.current_room].doors;
        if door_list.len() > 0{
            rs.render_textured(0 as usize, self.door1.model.clone(), FTextured::new(get_trf(door_list[0].direction, ROOMSIZE, self.door1.trf.scale))); 
        }
        if door_list.len() > 1{
            rs.render_textured(1 as usize, self.door2.model.clone(), FTextured::new(get_trf(door_list[1].direction, ROOMSIZE, self.door2.trf.scale))); 
        }
        if door_list.len() > 2{
            rs.render_textured(2 as usize, self.door3.model.clone(), FTextured::new(get_trf(door_list[2].direction, ROOMSIZE, self.door3.trf.scale))); 
        }
        if door_list.len() > 3{
            rs.render_textured(3 as usize, self.door4.model.clone(), FTextured::new(get_trf(door_list[3].direction, ROOMSIZE, self.door4.trf.scale))); 
        }
        
        // for (obj_i, obj) in self.things.iter_mut().enumerate() {
        //     rs.render_skinned(
        //         obj_i,
        //         obj.model.clone(),
        //         FSkinned::new(obj.animation, obj.state, obj.trf),
        //     );
        // }
        for (s_i, s) in self.sprites.iter_mut().enumerate() {
            rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
        }
        // for (m_i, m) in self.flats.iter_mut().enumerate() {
        //     rs.render_flat(m_i, m.model.clone(), FFlat::new(m.trf));
        // }
        // for (t_i, t) in self.textured.iter_mut().enumerate() {
        //     rs.render_textured(t_i, t.model.clone(), FTextured::new(t.trf));
        // }
    }
}
fn main() -> Result<()> {
    frenderer::color_eyre::install()?;

    let mut engine: Engine = Engine::new(
        FrendererSettings {
            sprite: SpriteRendererSettings {
                cull_back_faces: false,
                ..SpriteRendererSettings::default()
            },
            ..FrendererSettings::default()
        },
        DT,
    );

    let camera = Camera::look_at(
        Vec3::new(0., 0., 100.),
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
    );

    let tex = engine.load_texture(std::path::Path::new("content/robot.png"))?;
    let model = engine.load_textured(std::path::Path::new("content/characterSmall.fbx"))?;
    let char_model = engine.create_textured_model(model, vec![tex]);

    let door = engine.load_textured(std::path::Path::new("content/door.fbx"))?;
    let door_model = engine.create_textured_model(door, vec![tex]);

    // let model = engine.load_textured(std::path::Path::new("content/characterSmall.fbx"))?;
    // let char_model = engine.create_textured_model(model, vec![tex]);

    // let meshes = engine.load_skinned(
    //     std::path::Path::new("content/characterSmall.fbx"),
    //     &["RootNode", "Root"],
    // )?;
    // let animation = engine.load_anim(
    //     std::path::Path::new("content/kick.fbx"),
    //     meshes[0],
    //     AnimationSettings { looping: true },
    //     "Root|Kick",
    // )?;
    // let model = engine.create_skinned_model(meshes, vec![tex]);

    let door_1 = Door {
        direction: Direction::North,
        target: 0,
    };
    let door_2 = Door {
        direction: Direction::East,
        target: 0,
    };
    let door_3 = Door {
        direction: Direction::South,
        target: 0,
    };
    let door_4 = Door {
        direction: Direction::West,
        target: 0,
    };
    let starting_room = Room {
        doors: vec![door_1, door_2, door_3, door_4],
        floor: engine.load_texture(std::path::Path::new("content/robot.png"))?,
        objects: vec![],
    };

    let game_state = GameState {
        current_room: 0, //index of room in rooms
        max_rooms: 3,
        key_index: 2,
        //inventory: vec![],
        rooms: vec![starting_room],
        is_finished: false,
    };

    let world = World {
        camera,
        things: vec![
            // GameObject {
            //     trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 1.0),
            //     model,
            //     animation,
            //     state: AnimationState { t: 0.0 },
            // }
        ],
        sprites: vec![
            Sprite {
                trf: Isometry3::new(Vec3::new(20.0, 5.0, -10.0), Rotor3::identity()),
                size: Vec2::new(16.0, 16.0),
                cel: Rect::new(0.5, 0.0, 0.5, 0.5),
                tex: tex,
            }
        ],
        flats: vec![],
        textured: vec![
        // Textured {
        //     trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0),Rotor3::from_euler_angles(0.0, -PI / 2.0, 0.0), 1.0),
        //     model: char_model,
        //     name: String::from("Character")
        // }
        ],
        door1: Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0),Rotor3::from_euler_angles(0.0, 0.0, 0.0), 10.0),
            model: door_model.clone(),
            name: String::from("Door1")
        },
        door2: Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0),Rotor3::from_euler_angles(0.0, 0.0, 0.0), 10.0),
            model: door_model.clone(),
            name: String::from("Door2")
        },
        door3: Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0),Rotor3::from_euler_angles(0.0, 0.0, 0.0), 10.0),
            model: door_model.clone(),
            name: String::from("Door1")
        },
        door4: Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0),Rotor3::from_euler_angles(0.0, 0.0, 0.0), 10.0),
            model: door_model.clone(),
            name: String::from("Door2")
        },
        state: game_state,
    };
    engine.play(world)
}

fn get_trf(dir: Direction, room_size: f32, scale: f32)-> Similarity3{
    match dir {
        Direction::North => Similarity3::new(Vec3::new(0.0, 0.0, room_size/2.),Rotor3::from_euler_angles(0.0, -PI / 2.0, 0.0), scale),
        Direction::East => Similarity3::new(Vec3::new(room_size/2., 0.0, 0.0),Rotor3::from_euler_angles(PI / 2.0, -PI / 2.0, 0.0), scale),
        Direction::South => Similarity3::new(Vec3::new(0.0, 0.0, -room_size/2.),Rotor3::from_euler_angles(0.0, -PI / 2.0, 0.0), scale),
        Direction::West => Similarity3::new(Vec3::new(-room_size/2., 0.0, 0.0),Rotor3::from_euler_angles(-PI / 2.0, -PI / 2.0, 0.0), scale),
        // Direction::Other(n) => n as usize,
    }
}
