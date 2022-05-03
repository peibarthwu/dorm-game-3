#![allow(dead_code)]
use frenderer::animation::{AnimationSettings, AnimationState};
use frenderer::assets::AnimRef;
use frenderer::camera::{self, Camera};
//use frenderer::image::Image;
use frenderer::renderer::skinned::SingleRenderState as FSkinned;
use frenderer::renderer::textured::SingleRenderState as FTextured;
use frenderer::types::*;
use frenderer::{Engine, FrendererSettings, Key, Result, SpriteRendererSettings};
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

#[derive(Clone)]

pub struct GameState {
    pub current_room: usize, //index of room in rooms
    pub max_rooms: usize,
    pub key_index: usize,
    //pub inventory: Vec<GameObject>,
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    pub is_finished: bool,
    pub has_key: bool,
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
        //dbg!(self.state);
    }

    pub fn move_by(&mut self, vec: Vec3) {
        self.trf.append_translation(vec);
        //figure out append translation for the object
        //self.model.append_translation(vec);
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

    //we will be checking collisions with the textured object
    //this is not a great fnc in that I don't have it taking into account the location of the object
    //and it is only from if the object is in the origin of the screen
    //easy fix, but not rn
    //block is 7.75 on all sides, the

    //obj_edge_length is length from middle of object
    pub fn check_item_collisions(
        &mut self,
        direction: Direction,
        obj_edge_length_x: f32,
        obj_edge_length_z: f32,
        object: &Textured,
    ) -> bool {
        if direction == Direction::North {
            return self.trf.translation.x <= object.trf.translation.x + obj_edge_length_x
                && self.trf.translation.x >= object.trf.translation.x - obj_edge_length_x
                && self.trf.translation.z <= object.trf.translation.z + obj_edge_length_z;
        } else if direction == Direction::East {
            return self.trf.translation.z <= object.trf.translation.z + obj_edge_length_z
                && self.trf.translation.z >= object.trf.translation.z - obj_edge_length_z
                && self.trf.translation.x <= object.trf.translation.x - obj_edge_length_x;
        } else if direction == Direction::West {
            return self.trf.translation.z <= object.trf.translation.z + obj_edge_length_z
                && self.trf.translation.z >= object.trf.translation.z - obj_edge_length_z
                && self.trf.translation.x <= object.trf.translation.x + obj_edge_length_x;
        }
        //facing south
        else {
            return self.trf.translation.x <= object.trf.translation.x + obj_edge_length_x
                && self.trf.translation.x >= object.trf.translation.x - obj_edge_length_x
                && self.trf.translation.x >= object.trf.translation.z - obj_edge_length_z;
        }
    }

    pub fn check_collisions(&mut self, door: Door) -> bool {
        let door_worldspace = get_trf(door.direction, ROOMSIZE, SCALE);
        if door.direction == Direction::North {
            return self.trf.translation.z >= door_worldspace.translation.z - DOOR_DEPTH * SCALE
                && self.trf.translation.x
                    <= door_worldspace.translation.x + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.x
                    >= door_worldspace.translation.x - DOOR_WIDTH * SCALE * 3.0;
        } else if door.direction == Direction::South {
            return self.trf.translation.z <= door_worldspace.translation.z + DOOR_DEPTH * SCALE
                && self.trf.translation.x
                    <= door_worldspace.translation.x + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.x
                    >= door_worldspace.translation.x - DOOR_WIDTH * SCALE * 3.0;
        } else if door.direction == Direction::East {
            return self.trf.translation.x >= door_worldspace.translation.x - DOOR_DEPTH * SCALE
                && self.trf.translation.z
                    <= door_worldspace.translation.z + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.z
                    >= door_worldspace.translation.z - DOOR_WIDTH * SCALE * 3.0;
        } else if door.direction == Direction::West {
            return self.trf.translation.x <= door_worldspace.translation.x + DOOR_DEPTH * SCALE
                && self.trf.translation.z
                    <= door_worldspace.translation.z + DOOR_WIDTH * SCALE * 3.0
                && self.trf.translation.z
                    >= door_worldspace.translation.z - DOOR_WIDTH * SCALE * 3.0;
        } else {
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

        let yaw = input.key_axis(Key::Q, Key::W) * PI / 4.0 * DT as f32;
        let pitch = input.key_axis(Key::A, Key::S) * PI / 4.0 * DT as f32;
        let roll = input.key_axis(Key::Z, Key::X) * PI / 4.0 * DT as f32;
        let dscale = input.key_axis(Key::E, Key::R) * 1.0 * DT as f32;
        let rot = Rotor3::from_euler_angles(roll, pitch, yaw);

        //Press S to play
        if self.state.gameplaystate == GameplayState::Mainscreen {
            if input.is_key_down(Key::S) {
                //self.textured[0].trf.append_translation(to_the_moon);
                self.state.gameplaystate = GameplayState::Play;
            }
        }
        //controls for gameplaystate play
        else if self.state.gameplaystate == GameplayState::Play {
            self.things[0].tick_animation();

            for s in self.sprites.iter_mut() {
                //to think about: collisions with chest in first room

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
                for dooridx in self.state.rooms[self.state.current_room].doors.iter() {
                    let door = self.state.doors[*dooridx as usize];
                    if s.check_collisions(door) {
                        self.state.current_room = door.target;
                        s.trf.translation = get_spawn_pos(door.direction);
                        s.tex_model.trf.translation = get_spawn_pos(door.direction);
                        self.things[0].trf.translation = get_spawn_pos(door.direction);
                        dbg!({ "" }, self.state.current_room);
                        dbg!({ "" }, s.tex_model.trf.translation);
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
                    self.state.has_key = true;
                }

                //if we have the key, are in first room, and are collided with the chest
                //we are finished with the game
                //something is wrong with the code right here
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
            // for m in self.flats.iter_mut() {
            //     m.trf.append_rotation(rot);
            //     m.trf.scale += dscale;
            // }
            // for m in self.textured.iter_mut() {
            //     m.trf.append_rotation(rot);
            //     m.trf.scale += dscale;
            // }
            let camera_drot = input.key_axis(Key::Left, Key::Right) * PI / 4.0 * DT as f32;
            self.camera
                .transform
                .prepend_rotation(Rotor3::from_rotation_xz(camera_drot));
        }
        //restart the game by pressing S, and randomize
        else if self.state.gameplaystate == GameplayState::FinalScreen {
            if input.is_key_down(Key::S) {
                self.state = restart();
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
            rs.render_textured(
                8,
                self.sprites[0].tex_model.model.clone(),
                FTextured::new(self.sprites[0].tex_model.trf),
            );

            // Other render code
            // for (s_i, s) in self.sprites.iter_mut().enumerate() {
            //     rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
            //     // rs.render_textured(s_i, s.tex_model.model.clone(), FTextured::new(s.tex_model.trf));
            // }

            // for (m_i, m) in self.flats.iter_mut().enumerate() {
            //     rs.render_flat(m_i, m.model.clone(), FFlat::new(m.trf));
            // }

            // for (t_i, t) in self.textured.iter_mut().enumerate() {
            //     rs.render_textured(4 as usize, t.model.clone(), FTextured::new(t.trf));
            // }

            // for (s_i, s) in self.sprites.iter_mut().enumerate() {
            //     rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
            // }
            //render sprite

            //we still need to render the plane
            // for (t_i, t) in self.textured.iter_mut().enumerate() {
            //     rs.render_textured(4 as usize, t.model.clone(), FTextured::new(t.trf));
            // }
        }
        //final screen
        else if self.state.gameplaystate == GameplayState::FinalScreen {
            rs.render_textured(
                0,
                self.main_screen_textured[0].model.clone(),
                FTextured::new(self.main_screen_textured[0].trf),
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
        Vec3::new(0., 0., 100.),
        Vec3::new(0., 0., 0.),
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
    let door = engine
        .assets()
        .load_textured(std::path::Path::new("content/door.fbx"))?;
    let door_model = engine.assets().create_textured_model(door, vec![tex]);

    // room model
    let room = engine
        .assets()
        .load_textured(std::path::Path::new("content/room2.fbx"))?;
    let room_model = engine.assets().create_textured_model(room, vec![tex]);

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

    //text plane
    let text_plane_test_tex = engine
        .assets()
        .load_texture(std::path::Path::new("content/temp title texture.png"))?;
    let text_plane_mesh = engine
        .assets()
        .load_textured(std::path::Path::new("content/text_plane.fbx"))?;
    let text_plane_model = engine
        .assets()
        .create_textured_model(text_plane_mesh, vec![text_plane_test_tex]);

    //code for skinned model and gameObject
    let sprite_meshes = engine.assets().load_skinned(
        std::path::Path::new("content/characterSmall.fbx"),
        &["RootNode", "Root"],
    )?;

    let sprite_animation = engine.assets().load_anim(
        std::path::Path::new("content/run.fbx"),
        sprite_meshes[0],
        AnimationSettings { looping: true },
        "Root|Run",
    )?;

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
        sprite_animation,
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

    // let rooms = generate_rooms(5);
    //THIS GENERATES THE ROOMS
    // let mut rooms = Vec::<Room>::new();
    // let mut doors = Vec::<Door>::new();
    // let mut n = 0;

    //create n rooms
    let (room_list, door_list) = generate_room_map(NUM_ROOMS as u32);

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
        gameplaystate: GameplayState::Play,
    };

    let world = World {
        camera,
        things: vec![sprite_obj],
        sprites: vec![game_sprite],
        flats: vec![],
        //textured: vec![],
        main_screen_textured: vec![Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 80.0),
            model: text_plane_model.clone(),
            name: String::from("text plane"),
        }],

        //key model
        textured: vec![
            Textured {
                trf: Similarity3::new(Vec3::new(0.0, 10.0, 0.0), Rotor3::identity(), 5.0),
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
fn restart() -> GameState {
    let (room_list, door_list) = generate_room_map(NUM_ROOMS as u32);

    return GameState {
        current_room: 0, //index of room in rooms
        max_rooms: 3,
        key_index: 2,
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
        gameplaystate: GameplayState::Play,
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

fn generate_room_map(num_rooms: u32) -> (Vec<Room>, Vec<Door>) {
    let mut rooms = Vec::<Room>::new();
    let mut doors = Vec::<Door>::new();
    let mut n = 0;
    let mut num_doors = 0;
    //create n rooms

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
            dbg!(&doors);
            rooms.push(room);
            rooms.push(room2);
        } else {
            let room = &mut rooms[n as usize]; //get last room
                                               // add another door
            let door = gen_valid_door(&room, n as usize + 1, &doors); //generate random door //NEED TO CHECK IF VALID DOOR
            doors.push(door); //add door to the list of doors
            num_doors = doors.len() - 1;
            room.doors.push(num_doors as usize); //add door to room

            let back_door = create_bidirectional_door(door, n as usize); //generate door that points back at first door
            doors.push(back_door); //add door to the list of doors
            let room2 = Room::new(vec![num_doors as usize + 1]); //create next room
            rooms.push(room2);
        }
        dbg!(&doors);
        dbg!(&rooms);
        n += 1;
    }
    return (rooms, doors);
}

fn gen_valid_door(room: &Room, target: usize, doors: &[Door]) -> Door {
    let mut door = generate_door(target);
    // if this is the first door return door
    if room.doors.len() == 0 {
        return door;
    }
    //else we need to make sure there are no repeats
    let mut check = true;
    for n in 0..room.doors.len() - 1 {
        if door.direction == doors[room.doors[n]].direction {
            check = false;
        }
        if door.target == doors[room.doors[n]].target {
            check = false;
        }
    }
    while !check {
        door = generate_door(target);
        // check = check_valid_door(door, room, doors);
        for n in 0..room.doors.len() - 1 {
            if door.direction == doors[room.doors[n]].direction {
                check = false;
            }
            if door.target == doors[room.doors[n]].target {
                check = false;
            }
        }
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
