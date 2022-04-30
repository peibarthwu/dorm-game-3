#![allow(dead_code)]
use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::{AnimRef, MeshRef};
use frenderer::camera::Camera;
//use frenderer::image::Image;
use frenderer::renderer::flat::SingleRenderState as FFlat;
use frenderer::renderer::skinned::SingleRenderState as FSkinned;
use frenderer::renderer::sprites::SingleRenderState as FSprite;
use frenderer::renderer::textured::{Model, SingleRenderState as FTextured};
use frenderer::types::*;
use frenderer::{Engine, FrendererSettings, Key, Result, SpriteRendererSettings};
use rand::Rng;
use scene3d::types::{self, *};
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;
const SPEED: f32 = 0.25;
const ROOMSIZE: f32 = 60.0;
const COLLISION_RADIUS: f32 = 1.0;
const SCALE: f32 = 10.0;
const DOOR_WIDTH: f32 = 0.177;
const DOOR_DEPTH: f32 = 0.07;


#[derive(Clone)]

pub struct GameState {
    pub current_room: usize, //index of room in rooms
    pub max_rooms: usize,
    pub key_index: usize,
    //pub inventory: Vec<GameObject>,
    pub rooms: Vec<Room>,
    pub is_finished: bool,
    pub gameplaystate: GameplayState,
}

#[derive(Clone)]
struct GameObject {
    trf: Similarity3,
    model: Rc<frenderer::renderer::skinned::Model>,
    animation: AnimRef,
    state: AnimationState,
}
//tick animation forward
impl GameObject {
    fn new(
        trf: Similarity3,
        model: Rc<frenderer::renderer::skinned::Model>,
        animation: AnimRef,
        state: AnimationState,
    ) -> GameObject {
        GameObject {
            trf,
            model,
            animation,
            state,
        }
    }
    fn tick_animation(&mut self) {
        self.state.tick(DT);
    }
}

struct Sprite {
    trf: Isometry3,
    tex: frenderer::assets::TextureRef,
    cel: Rect,
    size: Vec2,
    tex_model: Textured,
    //figure out how to do this tex_model
    //
    // tex_model: Vec<MeshRef<frenderer::renderer::flat::Mesh>>,
    // animation: AnimRef,
    //model: Rc<Model>,
    //sprite_object: GameObject,
}
impl Sprite {
    pub fn move_by(&mut self, vec: Vec3) {
        self.trf.append_translation(vec);
        self.tex_model.trf.append_translation(vec);
    }
    pub fn check_collisions(&mut self, door: Door) -> bool {
        let door_worldspace = get_trf(door.direction, ROOMSIZE, SCALE);
        if door.direction == Direction::North{
            return (self.trf.translation.z >= door_worldspace.translation.z - DOOR_DEPTH*SCALE && 
                self.trf.translation.x <= door_worldspace.translation.x + DOOR_WIDTH*SCALE/2.0 && 
                self.trf.translation.x >= door_worldspace.translation.x - DOOR_WIDTH*SCALE/2.0);
        }
        else if door.direction == Direction::South{
            return (self.trf.translation.z <= door_worldspace.translation.z + DOOR_DEPTH*SCALE && 
                self.trf.translation.x <= door_worldspace.translation.x + DOOR_WIDTH*SCALE/2.0 && 
                self.trf.translation.x >= door_worldspace.translation.x - DOOR_WIDTH*SCALE/2.0);
        }
        else if door.direction == Direction::East{
            return (self.trf.translation.x >= door_worldspace.translation.x - DOOR_DEPTH*SCALE && 
                self.trf.translation.z <= door_worldspace.translation.z + DOOR_WIDTH*SCALE/2.0 && 
                self.trf.translation.z >= door_worldspace.translation.z - DOOR_WIDTH*SCALE/2.0);
        }
        else if door.direction == Direction::South{
            return (self.trf.translation.x <= door_worldspace.translation.x + DOOR_DEPTH*SCALE && 
                self.trf.translation.z <= door_worldspace.translation.z + DOOR_WIDTH*SCALE/2.0 && 
                self.trf.translation.z >= door_worldspace.translation.z - DOOR_WIDTH*SCALE/2.0);
        }else{
            return false;
        }
        // let door_worldspace = get_trf(door.direction, ROOMSIZE, SCALE);
        
        // let door_collider_rect = Rect::new(door_worldspace.translation.x, door_worldspace.translation.y, self.door_collider.x,  self.door_collider.x);
        // return (self.trf.translation.x <= door_worldspace.translation.x && self.pos.y <= other.pos.y && obr.x <= br.x && obr.y <= br.y

            
        //     self.trf.translation.x - door_worldspace.translation.x).abs() <= self.size.x / 2.
        //     && (self.trf.translation.z - door_worldspace.translation.z).abs() <= COLLISION_RADIUS;
        // // if self.cel.contains(door_worldspace.translation)
    }
}

#[derive(Clone, PartialEq)]
pub enum GameplayState {
    Mainscreen,
    Instructions,
    Play,
    FinalScreen,
}
struct World {
    camera: Camera,
    things: Vec<GameObject>,
    sprites: Vec<Sprite>,
    flats: Vec<Flat>,
    textured: Vec<Textured>,
    door1: Textured,
    door2: Textured,
    door3: Textured,
    door4: Textured,
    room: Textured,
    door_collider: Vec2,
    state: GameState,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
struct Textured {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
    name: String,
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        let yaw = input.key_axis(Key::Q, Key::W) * PI / 4.0 * DT as f32;
        let pitch = input.key_axis(Key::A, Key::S) * PI / 4.0 * DT as f32;
        let roll = input.key_axis(Key::Z, Key::X) * PI / 4.0 * DT as f32;
        let dscale = input.key_axis(Key::E, Key::R) * 1.0 * DT as f32;
        let rot = Rotor3::from_euler_angles(roll, pitch, yaw);
        let to_the_moon = Vec3::new(0.0, 100.0, 0.0);

        //controls for gameplaystate mainscreen
        if self.state.gameplaystate == GameplayState::Mainscreen {
            if input.is_key_down(Key::S) {
                self.state.gameplaystate = GameplayState::Play;
                for obj in self.textured.iter_mut() {
                    obj.trf.append_translation(to_the_moon);
                }
            }
        }
        //controls for gameplaystate play
        else if self.state.gameplaystate == GameplayState::Play {
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
                if input.is_key_down(Key::W) {
                    s.move_by(Vec3::new(0.0, 0.0, -SPEED));
                }
                if input.is_key_down(Key::S) {
                    s.move_by(Vec3::new(0.0, 0.0, SPEED));
                }
                if input.is_key_down(Key::A) {
                    s.move_by(Vec3::new(-SPEED, 0.0, 0.0));
                }
                if input.is_key_down(Key::D) {
                    s.move_by(Vec3::new(SPEED, 0.0, 0.0));
                }
                for door in self.state.rooms[self.state.current_room].doors.iter() {
                    if (s.check_collisions(*door)) {
                        self.state.current_room = door.target;
                        s.trf.translation = get_trf(door.direction, ROOMSIZE, SCALE).translation;
                        dbg!({ "" }, self.state.current_room);
                    }
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
    }
    fn render(
        &mut self,
        _a: &mut frenderer::assets::Assets,
        rs: &mut frenderer::renderer::RenderState,
    ) {
        rs.set_camera(self.camera);

        //gameplaystate:: Mainscreen
        if self.state.gameplaystate == GameplayState::Mainscreen {
            for (t_i, t) in self.textured.iter_mut().enumerate() {
                rs.render_textured(4 as usize, t.model.clone(), FTextured::new(t.trf));
            }
        }
        //gameplaystate:: play
        else if self.state.gameplaystate == GameplayState::Play {
            //render the doors in the correct positions
            let door_list = &self.state.rooms[self.state.current_room].doors;
            let mut tex_render_key = 0 as usize;
            if door_list.len() > 0 {
                rs.render_textured(
                    0 as usize,
                    self.door1.model.clone(),
                    FTextured::new(get_trf(
                        door_list[0].direction,
                        ROOMSIZE,
                        self.door1.trf.scale,
                    )),
                );
            }
            if door_list.len() > 1 {
                rs.render_textured(
                    1 as usize,
                    self.door2.model.clone(),
                    FTextured::new(get_trf(
                        door_list[1].direction,
                        ROOMSIZE,
                        self.door2.trf.scale,
                    )),
                );
            }
            if door_list.len() > 2 {
                rs.render_textured(
                    2 as usize,
                    self.door3.model.clone(),
                    FTextured::new(get_trf(
                        door_list[2].direction,
                        ROOMSIZE,
                        self.door3.trf.scale,
                    )),
                );
            }
            if door_list.len() > 3 {
                rs.render_textured(
                    3 as usize,
                    self.door4.model.clone(),
                    FTextured::new(get_trf(
                        door_list[3].direction,
                        ROOMSIZE,
                        self.door4.trf.scale,
                    )),
                );
            }

            for (obj_i, obj) in self.things.iter_mut().enumerate() {
                rs.render_skinned(
                    5 as usize,
                    obj.model.clone(),
                    FSkinned::new(obj.animation, obj.state, obj.trf),
                );
            }

            //render room
            rs.render_textured(
                6 as usize,
                self.room.model.clone(),
                FTextured::new(Similarity3::new(
                    Vec3::new(0.0, ROOMSIZE / 2., 0.0),
                    Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                    ROOMSIZE / 2.,
                )),
            );

<<<<<<< HEAD
            // //render the sprites
            for (s_i, s) in self.sprites.iter_mut().enumerate() {
                rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
                // rs.render_textured(s_i, s.tex_model.model.clone(), FTextured::new(s.tex_model.trf));
            }
        }
=======
        for (obj_i, obj) in self.things.iter_mut().enumerate() {
            rs.render_skinned(
                5 as usize,
                obj.model.clone(),
                FSkinned::new(obj.animation, obj.state, obj.trf),
            );
        }

         //render room
         rs.render_textured(
            6 as usize,
            self.room.model.clone(),
            FTextured::new(Similarity3::new(
                Vec3::new(0.0,  ROOMSIZE/2., 0.0),
                Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                ROOMSIZE/2.),
            )
        );

        // //render the sprites
<<<<<<< HEAD
        for (s_i, s) in self.sprites.iter_mut().enumerate() {
            rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
            // rs.render_textured(s_i, s.tex_model.model.clone(), FTextured::new(s.tex_model.trf));
        }
=======
        // for (s_i, s) in self.sprites.iter_mut().enumerate() {
        //     rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
        //     // rs.render_textured(s_i, s.tex_model.model.clone(), FTextured::new(s.tex_model.trf));
        // }
>>>>>>> 32e7da614f642d5f382cddbbab6c75d905a2ec0d
>>>>>>> 26dbbe5522d4623b3391d078a4afe6df972f55f3

        // for (m_i, m) in self.flats.iter_mut().enumerate() {
        //     rs.render_flat(m_i, m.model.clone(), FFlat::new(m.trf));
        // }
<<<<<<< HEAD
=======

        //tricky: if rendering somehting the "key" needs to be unique for each one if it's the same rs function
        rs.render_textured(7, self.sprites[0].tex_model.model.clone(), FTextured::new( self.sprites[0].tex_model.trf));

        // for (t_i, t) in self.textured.iter_mut().enumerate() {
        //     rs.render_textured(4 as usize, t.model.clone(), FTextured::new(t.trf));
        // }
>>>>>>> 32e7da614f642d5f382cddbbab6c75d905a2ec0d
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

    //character model
    let tex = engine.load_texture(std::path::Path::new("content/robot.png"))?;
    let model = engine.load_textured(std::path::Path::new("content/characterSmall.fbx"))?;
    let char_model = engine.create_textured_model(model, vec![tex]);

    //door model
    let door = engine.load_textured(std::path::Path::new("content/door.fbx"))?;
    let door_model = engine.create_textured_model(door, vec![tex]);

    // room model
    let room = engine.load_textured(std::path::Path::new("content/room2.fbx"))?;
    let room_model = engine.create_textured_model(room, vec![tex]);

    //text plane
    let text_plane_test_tex = engine.load_textured(std::path::Path::new("content/room2.fbx"))?;
    let text_plane_mesh = engine.load_textured(std::path::Path::new("content/text_plane.fbx"))?;
    let text_plane_model = engine.create_textured_model(text_plane_mesh, vec![tex]);

    //code for skinned model and gameObject
    let sprite_meshes = engine.load_skinned(
        std::path::Path::new("content/characterSmall.fbx"),
        &["RootNode", "Root"],
    )?;

    let sprite_animation = engine.load_anim(
        std::path::Path::new("content/kick.fbx"),
        sprite_meshes[0],
        AnimationSettings { looping: true },
        "Root|Kick",
    )?;
    let sprite_texture = engine.load_texture(std::path::Path::new("content/robot.png"))?;
    let sprite_model = engine.create_skinned_model(sprite_meshes, vec![sprite_texture]);

    // sprite gameobject
    let sprite_obj = GameObject::new(
        Similarity3::new(
            Vec3::new(0.0, 0.0, 0.0),
            Rotor3::from_euler_angles(0.0, 0.0, 0.0),
            1.0,
        ),
        sprite_model,
        sprite_animation,
        AnimationState { t: 0.0 },
    );

    let game_sprite = Sprite {
        trf: Isometry3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::from_euler_angles(-PI/2.0, -PI /2.0, 0.0)),
        size: Vec2::new(16.0, 16.0),
        cel: Rect::new(0.5, 0.0, 0.5, 0.5),
        tex: tex,
        tex_model: Textured {
<<<<<<< HEAD
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), SCALE),
=======
            trf: Similarity3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Rotor3::identity(),
                0.01,
            ),
>>>>>>> 32e7da614f642d5f382cddbbab6c75d905a2ec0d
            model: char_model.clone(),
            name: String::from("Sprite"),
        },
    };

    let rooms = generate_rooms(5);

    let game_state = GameState {
        current_room: 0, //index of room in rooms
        max_rooms: 3,
        key_index: 2,
        //inventory: vec![],
        rooms: rooms,
        is_finished: false,
        gameplaystate: GameplayState::Mainscreen,
    };

    let world = World {
        camera,
        //game object isn't rendering :(
        things: vec![sprite_obj],
        sprites: vec![game_sprite],
        flats: vec![],
        //textured: vec![],
        textured: vec![Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 80.0),
            model: text_plane_model.clone(),
            name: String::from("text plane"),
        }],
        door1: Textured {
            trf: Similarity3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                SCALE,
            ),
            model: door_model.clone(),
            name: String::from("Door1"),
        },
        door2: Textured {
            trf: Similarity3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                SCALE,
            ),
            model: door_model.clone(),
            name: String::from("Door2"),
        },
        door3: Textured {
            trf: Similarity3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                SCALE,
            ),
            model: door_model.clone(),
            name: String::from("Door1"),
        },
        door4: Textured {
            trf: Similarity3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                SCALE,
            ),
            model: door_model.clone(),
            name: String::from("Door2"),
        },
        room: Textured {
            trf: Similarity3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Rotor3::from_euler_angles(0.0, 0.0, 0.0),
                SCALE,
            ),
            model: room_model.clone(),
            name: String::from("Room"),
        },
        door_collider: Vec2::new(DOOR_WIDTH*SCALE, DOOR_DEPTH*SCALE),
        state: game_state,
    };
    engine.play(world)
}

fn get_trf(dir: Direction, room_size: f32, scale: f32) -> Similarity3 {
    match dir {
        Direction::North => Similarity3::new(
            Vec3::new(0.0, 0.0, room_size / 2.),
            Rotor3::from_euler_angles(0.0, -PI / 2.0, 0.0),
            scale,
        ),
        Direction::East => Similarity3::new(
            Vec3::new(room_size / 2., 0.0, 0.0),
            Rotor3::from_euler_angles(PI / 2.0, -PI / 2.0, 0.0),
            scale,
        ),
        Direction::South => Similarity3::new(
            Vec3::new(0.0, 0.0, -room_size / 2.),
            Rotor3::from_euler_angles(0.0, -PI / 2.0, 0.0),
            scale,
        ),
        Direction::West => Similarity3::new(
            Vec3::new(-room_size / 2., 0.0, 0.0),
            Rotor3::from_euler_angles(-PI / 2.0, -PI / 2.0, 0.0),
            scale,
        ),
        // Direction::Other(n) => n as usize,
    }
}

fn generate_rooms(num_rooms: u32) -> Vec<Room> {
    let mut vec = Vec::<Room>::new();
    let mut rng = rand::thread_rng();
    for n in 0..num_rooms {
        let doors = generate_doors(rng.gen_range(1..4), num_rooms);
        let room = Room::new(doors, vec![]);
        vec.push(room);
    }
    return vec;
}

fn generate_doors(num_doors: u32, num_rooms: u32) -> Vec<Door> {
    let mut vec = Vec::<Door>::new();
    let mut dirs = Vec::<Direction>::new();
    let mut rng = rand::thread_rng();
    //generate directions
    //no two doors can be in the same direction
    let mut contains = false;
    let mut length = dirs.len();
    while length < num_doors as usize {
        let next_direction = get_dir(rng.gen_range(0..3));
        for direction in &*dirs {
            if next_direction == *direction {
                contains = true;
            }
        }
        if !contains {
            dirs.push(next_direction);
            length += 1;
            dbg!(length);
        }
        contains = false;
    }
    for n in 0..num_doors {
        let mut target = rng.gen_range(0..num_rooms as usize);
        while target == n as usize {
            target = rng.gen_range(0..num_rooms as usize);
        }
        let door = Door::new(dirs[n as usize], target, get_spawn_dir(dirs[n as usize]));
        vec.push(door);
    }
    return vec;
}

fn get_dir(num: u32) -> Direction {
    match num {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        3 => Direction::West,
        Other => Direction::West,
    }
}

fn get_spawn_dir(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
        Other => Direction::West,
    }
}
