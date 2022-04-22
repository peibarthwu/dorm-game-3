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
use std::rc::Rc;

const DT: f64 = 1.0 / 60.0;
pub struct GameState {
    pub current_room: usize, //index of room in rooms
    pub max_rooms: usize,
    pub key_index: usize,
    pub inventory: Vec<GameObject>,
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
struct World {
    camera: Camera,
    things: Vec<GameObject>,
    sprites: Vec<Sprite>,
    flats: Vec<Flat>,
    textured: Vec<Textured>,
}
struct Flat {
    trf: Similarity3,
    model: Rc<frenderer::renderer::flat::Model>,
}
struct Textured {
    trf: Similarity3,
    model: Rc<frenderer::renderer::textured::Model>,
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
            s.trf.append_rotation(rot);
            s.size.x += dscale;
            s.size.y += dscale;
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
        for (obj_i, obj) in self.things.iter_mut().enumerate() {
            rs.render_skinned(
                obj_i,
                obj.model.clone(),
                FSkinned::new(obj.animation, obj.state, obj.trf),
            );
        }
        for (s_i, s) in self.sprites.iter_mut().enumerate() {
            rs.render_sprite(s_i, s.tex, FSprite::new(s.cel, s.trf, s.size));
        }
        for (m_i, m) in self.flats.iter_mut().enumerate() {
            rs.render_flat(m_i, m.model.clone(), FFlat::new(m.trf));
        }
        for (t_i, t) in self.textured.iter_mut().enumerate() {
            rs.render_textured(t_i, t.model.clone(), FTextured::new(t.trf));
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
    );

    let tex = engine.load_texture(std::path::Path::new("content/robot.png"))?;
    let model = engine.load_textured(std::path::Path::new("content/characterSmall.fbx"))?;
    let char_model = engine.create_textured_model(tex, vec![model]);

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
    assert_eq!(meshes.len(), 1);
    // let model = engine.create_skinned_model(meshes, vec![tex]);

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
        sprites: vec![],
        flats: vec![],
        textured: vec![Textured {
            trf: Similarity3::new(Vec3::new(0.0, 0.0, 0.0), Rotor3::identity(), 1.0),
            model: char_model,
        }],
    };
    engine.play(world)
}
