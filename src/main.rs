#![allow(dead_code)]
use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::{self, Camera};
use frenderer::renderer::skinned::SingleRenderState as FSkinned;
use frenderer::renderer::textured::SingleRenderState as FTextured;
use frenderer::types::*;
use frenderer::{Engine, FrendererSettings, Key, Result, SpriteRendererSettings};
use kira::instance::InstanceSettings;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::handle::SoundHandle;
use kira::sound::SoundSettings;
use rand::Rng;
use scene3d::types::*;
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;
const SPEED: f32 = 0.5;
const ROOMSIZE: f32 = 60.0;
const COLLISION_RADIUS: f32 = 1.0;
const SCALE: f32 = 10.0;
const BUFFER: f32 = 5.0;
const DOOR_WIDTH: f32 = 0.177;
const DOOR_DEPTH: f32 = 0.07;
const NUM_ROOMS: i32 = 4;
const DIFFICULTY: usize = 3;

#[derive(Clone)]

pub struct GameState {
    pub current_room: usize, //index of room in rooms
    pub max_rooms: usize,
    pub key_index: usize,
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    pub is_finished: bool,
    pub has_key: bool,
    pub gameplaystate: GameplayState,
    pub audio_play: bool,
    pub has_rotated: bool,
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
        //dbg!(self.state);
    }

    pub fn move_by(&mut self, vec: Vec3) {
        if self.trf.translation.x + vec.x < ROOMSIZE / 2.0
            && self.trf.translation.x + vec.x > -ROOMSIZE / 2.0
            && self.trf.translation.z + vec.z < ROOMSIZE / 2.0
            && self.trf.translation.z + vec.z > -ROOMSIZE / 2.0
        {
            self.trf.append_translation(vec);
        }
    }

    pub fn get_dir(&mut self) -> Direction {
        if self.trf.rotation == Rotor3::from_euler_angles(0.0, 0.0, 0.0) {
            return Direction::South;
        } else if self.trf.rotation == Rotor3::from_euler_angles(0.0, 0.0, PI / 2.0 as f32) {
            return Direction::West;
        } else if self.trf.rotation == Rotor3::from_euler_angles(0.0, 0.0, PI as f32) {
            return Direction::North;
        } else {
            return Direction::East;
        }
    }
}

struct Sprite {
    trf: Isometry3,
    tex: frenderer::assets::TextureRef,
    cel: Rect,
    size: Vec2,
    tex_model: Textured,
}
impl Sprite {
    pub fn move_by(&mut self, vec: Vec3) {
        self.trf.append_translation(vec);
        self.tex_model.trf.append_translation(vec);
    }

    pub fn check_item_collisions(
        &mut self,
        direction: Direction,
        obj_edge_length_x: f32,
        obj_edge_length_z: f32,
        object: &Textured,
    ) -> bool {
        if direction == Direction::North || direction == Direction::South {
            return self.trf.translation.x <= object.trf.translation.x + obj_edge_length_x
                && self.trf.translation.x >= object.trf.translation.x - obj_edge_length_x
                && self.trf.translation.z <= object.trf.translation.z + obj_edge_length_z
                && self.trf.translation.z >= object.trf.translation.z - obj_edge_length_z;
        } else if direction == Direction::East || direction == Direction::West {
            return self.trf.translation.z <= object.trf.translation.z + obj_edge_length_z
                && self.trf.translation.z >= object.trf.translation.z - obj_edge_length_z
                && self.trf.translation.x >= object.trf.translation.x - obj_edge_length_x
                && self.trf.translation.x <= object.trf.translation.x + obj_edge_length_x;
        } else {
            return false;
        }
    }

    pub fn check_collisions(&mut self, door: Door) -> bool {
        let door_worldspace = get_trf(door.direction, ROOMSIZE, SCALE);
        if door.direction == Direction::North {
            return self.trf.translation.z + BUFFER / 2.0
                >= door_worldspace.translation.z - DOOR_DEPTH * SCALE
                && self.trf.translation.x
                    <= door_worldspace.translation.x + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.x
                    >= door_worldspace.translation.x - DOOR_WIDTH * SCALE * 3.0;
        } else if door.direction == Direction::South {
            return self.trf.translation.z - BUFFER / 2.0
                <= door_worldspace.translation.z + DOOR_DEPTH * SCALE
                && self.trf.translation.x
                    <= door_worldspace.translation.x + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.x
                    >= door_worldspace.translation.x - DOOR_WIDTH * SCALE * 3.0;
        } else if door.direction == Direction::East {
            return self.trf.translation.x + BUFFER / 2.0
                >= door_worldspace.translation.x - DOOR_DEPTH * SCALE
                && self.trf.translation.z
                    <= door_worldspace.translation.z + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.z
                    >= door_worldspace.translation.z - DOOR_WIDTH * SCALE * 3.0;
        } else if door.direction == Direction::West {
            return self.trf.translation.x - BUFFER / 2.0
                <= door_worldspace.translation.x + DOOR_DEPTH * SCALE
                && self.trf.translation.z
                    <= door_worldspace.translation.z + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.z
                    >= door_worldspace.translation.z - DOOR_WIDTH * SCALE * 3.0;
        } else {
            return false;
        }
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
    audio: Vec<SoundHandle>,
    things: Vec<GameObject>,
    sprites: Vec<Sprite>,
    main_screen_textured: Vec<Textured>,
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

impl Textured {
    pub fn move_by(&mut self, vec: Vec3) {
        self.trf.append_translation(vec);
    }
}
impl frenderer::World for World {
    fn update(&mut self, input: &frenderer::Input, _assets: &mut frenderer::assets::Assets) {
        //currently WAS

        if self.state.audio_play {
            self.audio[0].play(InstanceSettings::default());
            self.state.audio_play = false;
        }

        // let yaw = input.key_axis(Key::Q, Key::W) * PI / 4.0 * DT as f32;
        // let pitch = input.key_axis(Key::A, Key::S) * PI / 4.0 * DT as f32;
        // let roll = input.key_axis(Key::Z, Key::X) * PI / 4.0 * DT as f32;
        // let dscale = input.key_axis(Key::E, Key::R) * 1.0 * DT as f32;
        // let rot = Rotor3::from_euler_angles(roll, pitch, yaw);

        //Press S to play
        if self.state.gameplaystate == GameplayState::Mainscreen {
            if input.is_key_down(Key::S) {
                //self.textured[0].trf.append_translation(to_the_moon);
                self.state.gameplaystate = GameplayState::Instructions;
            }
        }

        //Press P to play
        if self.state.gameplaystate == GameplayState::Instructions {
            if input.is_key_down(Key::P) {
                //self.textured[0].trf.append_translation(to_the_moon);
                self.state.gameplaystate = GameplayState::Play;
            }
        }
        //controls for gameplaystate play
        else if self.state.gameplaystate == GameplayState::Play {
            if !self.state.has_rotated {
                self.camera
                    .transform
                    .prepend_rotation(Rotor3::from_rotation_xz(PI / 4.0));
                self.state.has_rotated = true;
            }

            for s in self.sprites.iter_mut() {
                if s.check_item_collisions(self.things[0].get_dir(), 7.75, 7.75, &self.textured[0])
                    && self.state.current_room == 0
                {
                    // check if collide with item and spawn back
                    dbg!(self.things[0].get_dir());
                    if self.things[0].get_dir() == Direction::North {
                        s.trf.translation += Vec3::new(0.0, 0.0, BUFFER / 2.0);
                        s.tex_model.trf.translation += Vec3::new(0.0, 0.0, BUFFER / 2.0);
                        self.things[0].trf.translation += Vec3::new(0.0, 0.0, BUFFER / 2.0);
                    } else if self.things[0].get_dir() == Direction::South {
                        s.trf.translation += Vec3::new(0.0, 0.0, -BUFFER / 2.0);
                        s.tex_model.trf.translation += Vec3::new(0.0, 0.0, -BUFFER / 2.0);
                        self.things[0].trf.translation += Vec3::new(0.0, 0.0, -BUFFER / 2.0);
                    } else if self.things[0].get_dir() == Direction::East {
                        s.trf.translation += Vec3::new(-BUFFER / 2.0, 0.0, 0.0);
                        s.tex_model.trf.translation += Vec3::new(-BUFFER / 2.0, 0.0, 0.0);
                        self.things[0].trf.translation += Vec3::new(-BUFFER / 2.0, 0.0, 0.0);
                    } else {
                        s.trf.translation += Vec3::new(BUFFER / 2.0, 0.0, 0.0);
                        s.tex_model.trf.translation += Vec3::new(BUFFER / 2.0, 0.0, 0.0);
                        self.things[0].trf.translation += Vec3::new(BUFFER / 2.0, 0.0, 0.0);
                    }
                } else {
                    if input.is_key_down(Key::W)
                        || input.is_key_down(Key::A)
                        || input.is_key_down(Key::S)
                        || input.is_key_down(Key::D)
                    {
                        self.things[0].tick_animation();
                    }
                    if input.is_key_down(Key::W) {
                        s.move_by(Vec3::new(0.0, 0.0, -SPEED));
                        if self.things[0].get_dir() != Direction::North {
                            self.things[0].trf.rotation =
                                Rotor3::from_euler_angles(0.0, 0.0, PI as f32);
                        }
                        self.things[0].move_by(Vec3::new(0.0, 0.0, -SPEED));
                    }
                    if input.is_key_down(Key::S) {
                        s.move_by(Vec3::new(0.0, 0.0, SPEED));
                        if self.things[0].get_dir() != Direction::South {
                            self.things[0].trf.rotation = Rotor3::from_euler_angles(0.0, 0.0, 0.0);
                        }
                        self.things[0].move_by(Vec3::new(0.0, 0.0, SPEED));
                    }
                    if input.is_key_down(Key::A) {
                        s.move_by(Vec3::new(-SPEED, 0.0, 0.0));
                        if self.things[0].get_dir() != Direction::West {
                            self.things[0].trf.rotation =
                                Rotor3::from_euler_angles(0.0, 0.0, PI / 2.0 as f32);
                        }
                        self.things[0].move_by(Vec3::new(-SPEED, 0.0, 0.0));
                    }
                    if input.is_key_down(Key::D) {
                        s.move_by(Vec3::new(SPEED, 0.0, 0.0));
                        if self.things[0].get_dir() != Direction::East {
                            self.things[0].trf.rotation =
                                Rotor3::from_euler_angles(0.0, 0.0, -PI / 2.0 as f32);
                        }
                        self.things[0].move_by(Vec3::new(SPEED, 0.0, 0.0));
                    }
                }

                for dooridx in self.state.rooms[self.state.current_room].doors.iter() {
                    let door = self.state.doors[*dooridx as usize];
                    if s.check_collisions(door) {
                        self.state.current_room = door.target;
                        s.trf.translation = get_spawn_pos(door.direction);
                        s.tex_model.trf.translation = get_spawn_pos(door.direction);
                        self.things[0].trf.translation = get_spawn_pos(door.direction);
                        dbg!(self.state.current_room);
                    }
                }

                //checking collision with key
                if self.state.current_room == self.state.key_index
                    && s.check_item_collisions(
                        self.things[0].get_dir(),
                        7.75,
                        3.0,
                        &self.textured[1],
                    )
                {
                    //dbg!({ "" }, self.textured[0].trf.translation);
                    self.state.has_key = true;
                }

                //if we have the key, are in first room, and are collided with the chest
                if self.state.current_room == 0
                    && self.state.has_key
                    && s.check_item_collisions(
                        self.things[0].get_dir(),
                        7.75,
                        7.75,
                        &self.textured[0],
                    )
                {
                    self.state.is_finished = true;
                    self.state.gameplaystate = GameplayState::FinalScreen;
                }
            }

            // let camera_drot = input.key_axis(Key::Left, Key::Right) * PI / 4.0 * DT as f32;
            // dbg!({ "" }, self.camera.transform.translation);
            // dbg!({ "" }, self.camera.transform.rotation);
            // self.camera
            //     .transform
            //     .prepend_rotation(Rotor3::from_rotation_xz(-0.3887));
        }
        //restart the game by pressing S, and randomize
        else if self.state.gameplaystate == GameplayState::FinalScreen {
            if input.is_key_down(Key::R) {
                self.state = restart(self.state.max_rooms);
            }
        }
    }
    fn render(
        &mut self,
        _a: &mut frenderer::assets::Assets,
        rs: &mut frenderer::renderer::RenderState,
    ) {
        rs.set_camera(self.camera);

        //gameplaystate:: Mainscreen
        //could do a match instead
        if self.state.gameplaystate == GameplayState::Mainscreen {
            rs.render_textured(
                0,
                self.main_screen_textured[0].model.clone(),
                FTextured::new(self.main_screen_textured[0].trf),
            );
        } else if self.state.gameplaystate == GameplayState::Instructions {
            rs.render_textured(
                1,
                self.main_screen_textured[1].model.clone(),
                FTextured::new(self.main_screen_textured[1].trf),
            );
        }
        //gameplaystate:: play
        else if self.state.gameplaystate == GameplayState::Play {
            //render the doors in the correct positions
            let door_list = &self.state.rooms[self.state.current_room].doors;

            //render the key if we are in the right room and if we don't have the key
            if self.state.key_index == self.state.current_room && !self.state.has_key {
                //render the key
                rs.render_textured(
                    10,
                    self.textured[1].model.clone(),
                    FTextured::new(self.textured[1].trf),
                );
            }
            //render the chest in the starting room
            else if self.state.current_room == 0 {
                //render the block
                rs.render_textured(
                    9,
                    self.textured[0].model.clone(),
                    FTextured::new(self.textured[0].trf),
                );
            }

            //place doors
            if door_list.len() > 0 {
                rs.render_textured(
                    0 as usize,
                    self.door1.model.clone(),
                    FTextured::new(get_trf(
                        self.state.doors[door_list[0]].direction,
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
                        self.state.doors[door_list[1]].direction,
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
                        self.state.doors[door_list[2]].direction,
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
                        self.state.doors[door_list[3]].direction,
                        ROOMSIZE,
                        self.door4.trf.scale,
                    )),
                );
            }

            //render the game object
            rs.render_skinned(
                5 as usize,
                self.things[0].model.clone(),
                FSkinned::new(
                    self.things[0].animation,
                    self.things[0].state,
                    self.things[0].trf,
                ),
            );

            rs.render_skinned(
                7 as usize,
                self.things[0].model.clone(),
                FSkinned::new(
                    self.things[0].animation,
                    self.things[0].state,
                    self.things[0].trf,
                ),
            );

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

            // render the sprites
            // rs.render_textured(
            //     8,
            //     self.sprites[0].tex_model.model.clone(),
            //     FTextured::new(self.sprites[0].tex_model.trf),
            // );
        } else if self.state.gameplaystate == GameplayState::FinalScreen {
            self.camera = Camera::look_at(
                Vec3::new(0., 40.0, 100.),
                Vec3::new(0., 40.0, 0.),
                Vec3::new(0., 1., 0.),
                camera::Projection::Perspective { fov: PI / 2.0 },
            );

            rs.render_textured(
                3,
                self.main_screen_textured[2].model.clone(),
                FTextured::new(self.main_screen_textured[2].trf),
            );
        }
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
        Vec3::new(0., 40.0, 100.),
        Vec3::new(0., 40.0, 0.),
        Vec3::new(0., 1., 0.),
        camera::Projection::Perspective { fov: PI / 2.0 },
    );

    //character model
    let tex = engine
        .assets()
        .load_texture(std::path::Path::new("content/robot.png"))?;
    let model = engine
        .assets()
        .load_textured(std::path::Path::new("content/characterSmall.fbx"))?;
    let char_model = engine.assets().create_textured_model(model, vec![tex]);

    //door model
    let door_tex = engine
        .assets()
        .load_texture(std::path::Path::new("content/floor.png"))?;
    let door = engine
        .assets()
        .load_textured(std::path::Path::new("content/door.fbx"))?;
    let door_model = engine.assets().create_textured_model(door, vec![door_tex]);

    // room model
    let room_tex = engine
        .assets()
        .load_texture(std::path::Path::new("content/tex2.png"))?;
    let room = engine
        .assets()
        .load_textured(std::path::Path::new("content/room.fbx"))?;
    let room_model = engine.assets().create_textured_model(room, vec![room_tex]);

    let key_tex = engine
        .assets()
        .load_texture(std::path::Path::new("content/blue color.png"))?;
    let key_mesh = engine
        .assets()
        .load_textured(std::path::Path::new("content/key please.fbx"))?;
    let key_model = engine
        .assets()
        .create_textured_model(key_mesh, vec![key_tex]);

    let block_tex = engine
        .assets()
        .load_texture(std::path::Path::new("content/gold metal .png"))?;
    let block_mesh = engine
        .assets()
        .load_textured(std::path::Path::new("content/block y.fbx"))?;
    let block_model = engine
        .assets()
        .create_textured_model(block_mesh, vec![block_tex]);

    let text_plane_main = engine
        .assets()
        .load_texture(std::path::Path::new("content/main screen tex.png"))?;

    let text_plane_main_screen_mesh = engine
        .assets()
        .load_textured(std::path::Path::new("content/text_plane.fbx"))?;

    let text_plane_main_screen_model = engine
        .assets()
        .create_textured_model(text_plane_main_screen_mesh, vec![text_plane_main]);

    let text_plane_instructions_mesh = engine
        .assets()
        .load_textured(std::path::Path::new("content/text_plane.fbx"))?;

    let text_plane_instructions = engine
        .assets()
        .load_texture(std::path::Path::new("content/instructions tex.png"))?;

    let text_plane_instructions_model = engine
        .assets()
        .create_textured_model(text_plane_instructions_mesh, vec![text_plane_instructions]);

    let text_plane_final_mesh = engine
        .assets()
        .load_textured(std::path::Path::new("content/text_plane.fbx"))?;

    let text_plane_final = engine
        .assets()
        .load_texture(std::path::Path::new("content/final tex.png"))?;

    let text_plane_final_model = engine
        .assets()
        .create_textured_model(text_plane_final_mesh, vec![text_plane_final]);

    let mut audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
    let ghost_choir = audio_manager
        .load_sound("content/ghost-choir.ogg", SoundSettings::default())
        .unwrap();

    //code for skinned model and gameObject
    let sprite_meshes = engine.assets().load_skinned(
        std::path::Path::new("content/characterSmall.fbx"),
        &["RootNode", "Root"],
    )?;

    let sprite_animation_run = engine.assets().load_anim(
        std::path::Path::new("content/run.fbx"),
        sprite_meshes[0],
        AnimationSettings { looping: true },
        "Root|Run",
    )?;

    let sprite_animation_stand = engine.assets().load_anim(
        std::path::Path::new("content/idle.fbx"),
        sprite_meshes[0],
        AnimationSettings { looping: false },
        "Root|Idle",
    )?;

    let animation_vec = vec![sprite_animation_run, sprite_animation_stand];

    let sprite_texture = engine
        .assets()
        .load_texture(std::path::Path::new("content/robot.png"))?;
    let sprite_model = engine
        .assets()
        .create_skinned_model(sprite_meshes, vec![sprite_texture]);

    // sprite gameobject
    let sprite_obj = GameObject::new(
        Similarity3::new(
            Vec3::new(20.0, 0.0, 0.0),
            Rotor3::from_euler_angles(0.0, 0.0, PI as f32),
            0.05,
        ),
        sprite_model,
        //run animation
        animation_vec[0],
        AnimationState { t: 0.0 },
    );

    let game_sprite = Sprite {
        trf: Isometry3::new(
            Vec3::new(20.0, 0.0, 0.0),
            Rotor3::from_euler_angles(-PI / 2.0, -PI / 2.0, 0.0),
        ), //change to Rotor3::identity() to see the plane
        size: Vec2::new(16.0, 16.0),
        cel: Rect::new(0.5, 0.0, 0.5, 0.5),
        tex: tex,
        tex_model: Textured {
            trf: Similarity3::new(Vec3::new(20.0, 0.0, 0.0), Rotor3::identity(), 0.01),
            model: char_model.clone(),
            name: String::from("Sprite"),
        },
    };

    //create n rooms
    let (room_list, door_list) = generate_room_map(NUM_ROOMS as u32, 2);

    let game_state = GameState {
        current_room: 0, //index of room in rooms
        max_rooms: 4,
        key_index: 1,
        //inventory: vec![],
        // rooms: vec![Room::new(vec![0]), Room::new(vec![1])],
        rooms: room_list,
        doors: door_list,
        // doors: vec![
        //     Door::new(Direction::North, 1, Direction::South),
        //     Door::new(Direction::South, 0, Direction::North),
        // ],
        is_finished: false,
        has_key: false,
        gameplaystate: GameplayState::Mainscreen,
        audio_play: true,
        has_rotated: false,
    };

    let world = World {
        camera,
        audio: vec![ghost_choir],
        things: vec![sprite_obj],
        sprites: vec![game_sprite],
        main_screen_textured: vec![
            Textured {
                trf: Similarity3::new(Vec3::new(0.0, 30.0, 0.0), Rotor3::identity(), 80.0),
                model: text_plane_main_screen_model.clone(),
                name: String::from("main screen text plane"),
            },
            Textured {
                trf: Similarity3::new(Vec3::new(0.0, 30.0, 0.0), Rotor3::identity(), 80.0),
                model: text_plane_instructions_model.clone(),
                name: String::from("instructions text plane"),
            },
            Textured {
                trf: Similarity3::new(Vec3::new(0.0, 30.0, 0.0), Rotor3::identity(), 80.0),
                model: text_plane_final_model.clone(),
                name: String::from("final text plane"),
            },
        ],

        //key model
        textured: vec![
            Textured {
                trf: Similarity3::new(Vec3::new(0.0, 5.0, 0.0), Rotor3::identity(), 5.0),
                model: block_model.clone(),
                name: String::from("block model"),
            },
            Textured {
                trf: Similarity3::new(Vec3::new(0.0, 10.0, 0.0), Rotor3::identity(), 0.1),
                model: key_model.clone(),
                name: String::from("key model"),
            },
        ],
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
        door_collider: Vec2::new(DOOR_WIDTH * SCALE, DOOR_DEPTH * SCALE),
        state: game_state,
    };
    engine.play(world)
}

//fix this so that max_rooms and key_index are randomized
fn restart(curr_number_rooms: usize) -> GameState {
    let (room_list, door_list) =
        generate_room_map((curr_number_rooms + DIFFICULTY) as u32, DIFFICULTY);
    let mut rng = rand::thread_rng();
    let keyidx = rng.gen_range(1..curr_number_rooms + DIFFICULTY);
    dbg!({ "key loca: " }, keyidx);
    return GameState {
        current_room: 0, //index of room in rooms
        max_rooms: curr_number_rooms + DIFFICULTY,
        key_index: keyidx,
        rooms: room_list,
        doors: door_list,
        is_finished: false,
        has_key: false,
        gameplaystate: GameplayState::Play,
        audio_play: false,
        has_rotated: false,
    };
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

fn get_dir(num: u32) -> Direction {
    match num {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        3 => Direction::West,
        _other => Direction::West,
    }
}

fn get_spawn_dir(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
    }
}

fn get_spawn_pos(dir: Direction) -> Vec3 {
    let spawn_dir = get_spawn_dir(dir);
    let mut spawn_loca = get_trf(spawn_dir, ROOMSIZE, SCALE).translation;
    //need to adjust so no oscilarting
    if spawn_dir == Direction::North {
        spawn_loca.z -= BUFFER;
    } else if spawn_dir == Direction::South {
        spawn_loca.z += BUFFER;
    } else if spawn_dir == Direction::East {
        spawn_loca.x -= BUFFER;
    } else {
        spawn_loca.x += BUFFER;
    }
    return spawn_loca;
}

fn generate_room_map(num_rooms: u32, num_dead_ends: usize) -> (Vec<Room>, Vec<Door>) {
    let mut rooms = Vec::<Room>::new();
    let mut doors = Vec::<Door>::new();
    let mut n = 0;
    //create n rooms
    let mut curr_dead_ends = 0;
    let mut rng = rand::thread_rng();

    while n < num_rooms - 1 {
        //if we arent on the last room we can add a door
        if rooms.len() == 0 {
            let mut room = Room::new(vec![]); //create room with the first door
            let door = gen_valid_door(&room, n as usize + 1, &doors); //generate random door //NEED TO CHECK IF VALID DOOR
            let back_door = create_bidirectional_door(door, n as usize); //generate door that points back at first door

            doors.push(door); //add door to the list of doors
            room.doors.push(n as usize); //add door to room
            doors.push(back_door); //add door to the list of doors
            let room2 = Room::new(vec![n as usize + 1]); //create next room

            rooms.push(room);
            rooms.push(room2);
        } else {
            let room = &mut rooms[n as usize]; //get last room
                                               // add another door
            let door = gen_valid_door(&room, n as usize + 1, &doors); //generate random door //NEED TO CHECK IF VALID DOOR
            doors.push(door); //add door to the list of doors
            let num_doors = doors.len() - 1;
            room.doors.push(num_doors as usize); //add door to room

            let back_door = create_bidirectional_door(door, n as usize); //generate door that points back at first door
            doors.push(back_door); //add door to the list of doors
            let room2 = Room::new(vec![num_doors as usize + 1]); //create next room
            rooms.push(room2);
        }
        n += 1;
        dbg!({ "while 1" });
    }

    //generate dead ends
    while curr_dead_ends < num_dead_ends {
        let roomidx = rng.gen_range(0..rooms.len() - 1);
        let srcroom = &mut rooms[roomidx]; //get room index
                                           //gen new door
        let door = gen_valid_door(srcroom, num_rooms as usize + curr_dead_ends, &doors);
        doors.push(door);
        srcroom.doors.push(doors.len() - 1);
        //make room that points back to that door
        let back_door = create_bidirectional_door(door, roomidx);
        doors.push(back_door);
        let dest_room = Room::new(vec![doors.len() - 1]); //create next room
        rooms.push(dest_room);

        curr_dead_ends += 1;
        dbg!({ "while 2" });
    }
    dbg!(&doors);
    dbg!(&rooms);
    return (rooms, doors);
}

fn gen_valid_door(room: &Room, target: usize, doors: &[Door]) -> Door {
    let mut door = generate_door(target);
    // if this is the first door return door
    if room.doors.len() == 0 {
        return door;
    }
    //else we need to make sure there are no repeats
    let mut check = true; //unique direction
    let mut check2 = true; //unique target
    for n in 0..room.doors.len() {
        if door.direction == doors[room.doors[n]].direction {
            check = false;
        }
        if door.target == doors[room.doors[n]].target {
            check2 = false;
        }
    }
    while check == false || check2 == false {
        door = generate_door(target);
        for n in 0..room.doors.len() {
            if door.direction == doors[room.doors[n]].direction {
                check = false;
            } else {
                check = true;
            }
            if door.target == doors[room.doors[n]].target {
                check2 = false;
            } else {
                check2 = true;
            }
        }
        dbg!({ "while 3" });
    }
    return door;
}

fn check_valid_door(door: Door, room: Room, doors: Vec<Door>) -> bool {
    //room is current room
    let mut check = true;
    for n in 0..room.doors.len() {
        if door.direction == doors[room.doors[n]].direction {
            check = false;
        }
    }
    return check;
}

//return a new door on oppoisite side that points back to the previous room
fn create_bidirectional_door(door: Door, cur_room: usize) -> Door {
    return Door::new(get_spawn_dir(door.direction), cur_room, door.direction);
}

//generate a door with random direction and target
fn generate_door(target: usize) -> Door {
    let mut rng = rand::thread_rng();
    let direction = get_dir(rng.gen_range(0..4));
    return Door::new(direction, target, get_spawn_dir(direction));
}
