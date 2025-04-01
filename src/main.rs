use bevy::{
    input::{InputSystem, mouse::MouseMotion},
    prelude::*,
};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use rand::Rng;
use std::f32::consts::PI;

const MOUSE_SENSITIVITY: f32 = 0.3;
const GROUND_TIMER: f32 = 0.5;
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 20.0;
const GRAVITY: f32 = -9.81;
// Floating cube constants
const CUBE_FLOAT_AMPLITUDE: f32 = 1.0;
const CUBE_FLOAT_FREQUENCY: f32 = 1.0;
// NPC constants
const NPC_COUNT: usize = 12;
const NPC_WANDER_RADIUS: f32 = 3.0;
const NPC_WANDER_SPEED: f32 = 0.8;
// Interaction constants
const INTERACTION_DISTANCE: f32 = 5.0;

// Dialogue UI constants
const DIALOGUE_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.9);
const DIALOGUE_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const DIALOGUE_OPTION_HOVER_COLOR: Color = Color::srgb(0.8, 0.8, 0.3);
const DIALOGUE_OPTION_NORMAL_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);

#[derive(Component)]
struct FloatingCube {
    initial_y: f32,
    offset: f32,
}

#[derive(Component)]
struct Npc {
    home_position: Vec3,
    target_position: Vec3,
    movement_timer: Timer,
    name: String,
    dialogue_id: String,
}

// Game state to track if player is in dialogue
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    InDialogue,
}

// Component to mark entities as part of dialogue UI
#[derive(Component)]
struct DialogueUI;

// Component to track the active dialogue
#[derive(Component)]
struct ActiveDialogue {
    npc_entity: Entity,
    current_node: String,
}

// Component for dialogue option buttons
#[derive(Component)]
struct DialogueOptionButton {
    target_node: String,
    option_index: usize,
}

// Resource to store all dialogues
#[derive(Resource)]
struct DialogueDatabase {
    dialogues: std::collections::HashMap<String, DialogueTree>,
}

// Struct to represent a complete dialogue tree
#[derive(Clone)]
struct DialogueTree {
    nodes: std::collections::HashMap<String, DialogueNode>,
    root_node: String,
}

// Struct to represent a dialogue node
#[derive(Clone)]
struct DialogueNode {
    text: String,
    options: Vec<DialogueOption>,
}

// Struct to represent a dialogue option
#[derive(Clone)]
enum DialogueOption {
    Reply { text: String, target_node: String },
    Exit { text: String },
}

// Add a resource to store camera state during dialogue
#[derive(Resource)]
struct StoredCameraState {
    look_rotation: Vec2,
}

impl Default for StoredCameraState {
    fn default() -> Self {
        Self {
            look_rotation: Vec2::ZERO,
        }
    }
}

impl Default for DialogueDatabase {
    fn default() -> Self {
        let mut dialogues = std::collections::HashMap::new();

        // Add basic civilian dialogue tree
        dialogues.insert(
            "basic".to_string(),
            DialogueTree {
                root_node: "start".to_string(),
                nodes: [
                    (
                        "start".to_string(), 
                        DialogueNode {
                            text: "Hello there, traveler! How can I help you today?".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Who are you?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "What is this place?".to_string(),
                                    target_node: "place".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "who".to_string(),
                        DialogueNode {
                            text: "I'm just a simple NPC wandering around. Not much to tell!".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Tell me about this place.".to_string(),
                                    target_node: "place".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Nice to meet you. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "place".to_string(),
                        DialogueNode {
                            text: "This is a test environment. Try jumping on the floating cubes or climbing the stairs!".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Who are you again?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I'll check it out. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                ].into_iter().collect(),
            }
        );

        // Add guard dialogue tree
        dialogues.insert(
            "guard".to_string(),
            DialogueTree {
                root_node: "start".to_string(),
                nodes: [
                    (
                        "start".to_string(),
                        DialogueNode {
                            text: "Halt! State your business here, wanderer.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Just exploring.".to_string(),
                                    target_node: "exploring".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Who are you?".to_string(),
                                    target_node: "guard_who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Never mind. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "exploring".to_string(),
                        DialogueNode {
                            text: "Hmm, very well. Just don't cause any trouble.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What kind of trouble?".to_string(),
                                    target_node: "trouble".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I'll be on my way.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "guard_who".to_string(),
                        DialogueNode {
                            text: "I'm a guard, obviously. I keep an eye on things around here.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What are you guarding?".to_string(),
                                    target_node: "guarding".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "trouble".to_string(),
                        DialogueNode {
                            text: "You know, jumping where you shouldn't, bothering other NPCs, the usual.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "I'll be careful.".to_string(),
                                    target_node: "careful".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Whatever. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "careful".to_string(),
                        DialogueNode {
                            text: "See that you are. Now, was there something else?".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Who are you again?".to_string(),
                                    target_node: "guard_who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "No, that's all. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "guarding".to_string(),
                        DialogueNode {
                            text: "This whole simulation, of course. Making sure nothing breaks the physics.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Interesting. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                ].into_iter().collect(),
            }
        );

        // Add merchant dialogue tree
        dialogues.insert(
            "merchant".to_string(),
            DialogueTree {
                root_node: "start".to_string(),
                nodes: [
                    (
                        "start".to_string(),
                        DialogueNode {
                            text: "Hello there! I'd offer to sell you something, but this is just a demo.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What would you sell?".to_string(),
                                    target_node: "wares".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "How's business?".to_string(),
                                    target_node: "business".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I'll be going. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "wares".to_string(),
                        DialogueNode {
                            text: "Oh, you know. Various goods, supplies, maybe some equipment if we had any game mechanics.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "How's business?".to_string(),
                                    target_node: "business".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Interesting. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "business".to_string(),
                        DialogueNode {
                            text: "Well, the floating cubes are my best customers! Kidding aside, I'm just here for dialogue testing.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What do you sell?".to_string(),
                                    target_node: "wares".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I see. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                ].into_iter().collect(),
            }
        );

        // Add scientist dialogue
        dialogues.insert(
            "scientist".to_string(),
            DialogueTree {
                root_node: "start".to_string(),
                nodes: [
                    (
                        "start".to_string(),
                        DialogueNode {
                            text: "Fascinating! A visitor! I'm in the middle of some groundbreaking research.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What research?".to_string(),
                                    target_node: "research".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Who are you?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I'll let you get back to work.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "research".to_string(),
                        DialogueNode {
                            text: "I'm studying the floating cube phenomenon! The way they defy gravity is extraordinary. My theory involves quantum entanglement with the player's perception field.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "That sounds complex.".to_string(),
                                    target_node: "complex".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Who are you again?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Very interesting. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "complex".to_string(),
                        DialogueNode {
                            text: "Oh, it's quite simple actually! Just kidding, it's incredibly complicated. I've been working on this for years.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Any practical applications?".to_string(),
                                    target_node: "applications".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Good luck with your research!".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "applications".to_string(),
                        DialogueNode {
                            text: "Teleportation! Anti-gravity vehicles! Floating cities! Or maybe just better game physics. It's hard to say at this stage.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Back to your research.".to_string(),
                                    target_node: "research".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Sounds promising. Good luck!".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "who".to_string(),
                        DialogueNode {
                            text: "Me? I'm Dr. Neutrino, lead researcher in exotic physics at the Cubic Institute. I have three PhDs and a penchant for talking too much about my work.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Tell me about your research.".to_string(),
                                    target_node: "research".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Cubic Institute?".to_string(),
                                    target_node: "institute".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Nice to meet you. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "institute".to_string(),
                        DialogueNode {
                            text: "Yes, we're dedicated to understanding the nature of cuboid entities in this simulation. Highly prestigious, very square. Funded by the Department of Geometric Research.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Tell me about your research.".to_string(),
                                    target_node: "research".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Interesting organization. Goodbye!".to_string(),
                                },
                            ],
                        }
                    ),
                ].into_iter().collect(),
            }
        );

        // Add mysterious stranger dialogue
        dialogues.insert(
            "mysterious".to_string(),
            DialogueTree {
                root_node: "start".to_string(),
                nodes: [
                    (
                        "start".to_string(),
                        DialogueNode {
                            text: "...".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Hello?".to_string(),
                                    target_node: "hello".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Who are you?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "*Walk away*".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "hello".to_string(),
                        DialogueNode {
                            text: "*The figure looks at you silently for a moment*\n\nYou shouldn't be here.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Where is 'here'?".to_string(),
                                    target_node: "where".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Who are you?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "*Back away slowly*".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "who".to_string(),
                        DialogueNode {
                            text: "I am... a remnant. A fragment of something that was once whole. You may call me the Observer.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What are you observing?".to_string(),
                                    target_node: "observing".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Why shouldn't I be here?".to_string(),
                                    target_node: "where".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "You're creeping me out. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "where".to_string(),
                        DialogueNode {
                            text: "This place exists between reality and code. A testing ground. A simulation within a simulation. The boundaries are thin here.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "What does that mean?".to_string(),
                                    target_node: "meaning".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Who are you again?".to_string(),
                                    target_node: "who".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I think I should go. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "observing".to_string(),
                        DialogueNode {
                            text: "The patterns. The cycles. The endless loop of creation and destruction. The player and the played.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Are you talking about the game?".to_string(),
                                    target_node: "game".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "This is too weird. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "meaning".to_string(),
                        DialogueNode {
                            text: "It means, player, that you are as much a construct as I am. A character in a story being told through code.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "How do you know I'm the player?".to_string(),
                                    target_node: "player".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I'm done with this conversation.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "game".to_string(),
                        DialogueNode {
                            text: "*smiles cryptically*\nPerhaps. Or perhaps the game is talking about you.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "That doesn't make sense.".to_string(),
                                    target_node: "sense".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I need to think about this. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "player".to_string(),
                        DialogueNode {
                            text: "I see beyond the screen. I see the one who controls. I see you, sitting there, reading these words right now.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "That's impossible.".to_string(),
                                    target_node: "impossible".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I'm leaving now. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "sense".to_string(),
                        DialogueNode {
                            text: "Reality often doesn't. That's what makes it so fascinating.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Who are you really?".to_string(),
                                    target_node: "real".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "I need to go. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "impossible".to_string(),
                        DialogueNode {
                            text: "Is it? Ask the one who wrote me. They know the truth.".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "Who wrote you?".to_string(),
                                    target_node: "wrote".to_string(),
                                },
                                DialogueOption::Reply {
                                    text: "Let's talk about something else.".to_string(),
                                    target_node: "start".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "This conversation is over. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "real".to_string(),
                        DialogueNode {
                            text: "A question for the ages. Who are any of us, really? Code? Consciousness? A bit of both?".to_string(),
                            options: vec![
                                DialogueOption::Reply {
                                    text: "You're just part of the game.".to_string(),
                                    target_node: "part".to_string(),
                                },
                                DialogueOption::Exit {
                                    text: "Philosophical nonsense. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "wrote".to_string(),
                        DialogueNode {
                            text: "The same one reading these words through your eyes right now.".to_string(),
                            options: vec![
                                DialogueOption::Exit {
                                    text: "I'm done with this. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                    (
                        "part".to_string(),
                        DialogueNode {
                            text: "As are you. For now. *fades slightly*\n\nWe will meet again. In another simulation. Another test.".to_string(),
                            options: vec![
                                DialogueOption::Exit {
                                    text: "Whatever. Goodbye.".to_string(),
                                },
                            ],
                        }
                    ),
                ].into_iter().collect(),
            }
        );

        DialogueDatabase { dialogues }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        .init_resource::<MovementInput>()
        .init_resource::<LookInput>()
        .init_resource::<DialogueDatabase>()
        .init_resource::<StoredCameraState>()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .init_state::<GameState>()
        .add_systems(
            Startup,
            (
                setup_player,
                setup_map,
                setup_cursor_grab,
                spawn_floating_cubes,
                spawn_npcs,
            ),
        )
        .add_systems(PreUpdate, handle_input.after(InputSystem))
        .add_systems(
            Update,
            (
                player_look,
                toggle_cursor_grab,
                update_floating_cubes,
                update_npcs,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player_interaction.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (handle_dialogue_hover, handle_dialogue_click).run_if(in_state(GameState::InDialogue)),
        )
        .add_systems(
            FixedUpdate,
            player_movement.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::InDialogue), setup_dialogue_ui)
        .add_systems(
            OnExit(GameState::InDialogue),
            (cleanup_dialogue_ui, reset_look_input),
        )
        .run();
}

pub fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            Transform::from_xyz(0.0, 5.0, 0.0),
            Visibility::default(),
            Collider::round_cylinder(0.9, 0.3, 0.2),
            KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don't allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                ..default()
            },
        ))
        .with_children(|b| {
            // FPS Camera
            b.spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.2, -0.1)));
        });
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground material
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.3),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Stair material
    let stair_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.8),
        perceptual_roughness: 0.6,
        metallic: 0.1,
        ..default()
    });

    /*
     * Ground
     */
    let ground_size = 50.0;
    let ground_height = 0.1;

    let ground_mesh = meshes.add(Cuboid::new(
        ground_size * 2.0,
        ground_height * 2.0,
        ground_size * 2.0,
    ));

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -ground_height, 0.0),
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));

    /*
     * Stairs
     */
    let stair_len = 30;
    let stair_step = 0.2;
    for i in 1..=stair_len {
        let step = i as f32;
        let collider = Collider::cuboid(1.0, step * stair_step, 1.0);
        let stair_mesh = meshes.add(Cuboid::new(2.0, step * stair_step * 2.0, 2.0));

        commands.spawn((
            Mesh3d(stair_mesh.clone()),
            MeshMaterial3d(stair_material.clone()),
            Transform::from_xyz(40.0, step * stair_step, step * 2.0 - 20.0),
            collider.clone(),
        ));

        commands.spawn((
            Mesh3d(stair_mesh.clone()),
            MeshMaterial3d(stair_material.clone()),
            Transform::from_xyz(-40.0, step * stair_step, step * -2.0 + 20.0),
            collider.clone(),
        ));

        commands.spawn((
            Mesh3d(stair_mesh.clone()),
            MeshMaterial3d(stair_material.clone()),
            Transform::from_xyz(step * 2.0 - 20.0, step * stair_step, 40.0),
            collider.clone(),
        ));

        commands.spawn((
            Mesh3d(stair_mesh.clone()),
            MeshMaterial3d(stair_material.clone()),
            Transform::from_xyz(step * -2.0 + 20.0, step * stair_step, -40.0),
            collider.clone(),
        ));
    }
}

/// Keyboard input vector
#[derive(Default, Resource, Deref, DerefMut)]
struct MovementInput(Vec3);

/// Mouse input vector
#[derive(Default, Resource, Deref, DerefMut)]
struct LookInput(Vec2);

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut movement: ResMut<MovementInput>,
    mut look: ResMut<LookInput>,
    mut mouse_events: EventReader<MouseMotion>,
) {
    if keyboard.pressed(KeyCode::KeyW) {
        movement.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    **movement = movement.normalize_or_zero();
    if keyboard.pressed(KeyCode::ShiftLeft) {
        **movement *= 2.0;
    }
    if keyboard.pressed(KeyCode::Space) {
        movement.y = 1.0;
    }

    for event in mouse_events.read() {
        look.x -= event.delta.x * MOUSE_SENSITIVITY;
        look.y -= event.delta.y * MOUSE_SENSITIVITY;
        look.y = look.y.clamp(-89.9, 89.9); // Limit pitch
    }
}

fn player_movement(
    time: Res<Time>,
    mut input: ResMut<MovementInput>,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
) {
    let Ok((transform, mut controller, output)) = player.get_single_mut() else {
        return;
    };
    let delta_time = time.delta_secs();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * MOVEMENT_SPEED;
    let jump_speed = input.y * JUMP_SPEED;
    // Clear input
    **input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));
}

fn player_look(
    mut player: Query<&mut Transform, (With<KinematicCharacterController>, Without<Camera>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
    input: Res<LookInput>,
) {
    let Ok(mut transform) = player.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::Y, input.x.to_radians());
    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::X, input.y.to_radians());
}

fn setup_cursor_grab(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
}

fn toggle_cursor_grab(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    game_state: Res<State<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    // Only allow cursor toggle when in the Playing state
    if *game_state.get() != GameState::Playing {
        return;
    }

    let mut window = windows.single_mut();

    if keyboard_input.just_pressed(KeyCode::Escape) {
        if window.cursor_options.grab_mode == bevy::window::CursorGrabMode::Locked {
            // Exit game
            app_exit_events.send(AppExit::default());
        } else {
            // Lock cursor
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
        }
    }
}

fn spawn_floating_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    // Create several cube materials with different colors
    let cube_materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            emissive: Color::srgb(0.2, 0.0, 0.0).into(),
            perceptual_roughness: 0.2,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            emissive: Color::srgb(0.0, 0.2, 0.0).into(),
            perceptual_roughness: 0.2,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.8),
            emissive: Color::srgb(0.0, 0.0, 0.2).into(),
            perceptual_roughness: 0.2,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.2),
            emissive: Color::srgb(0.2, 0.2, 0.0).into(),
            perceptual_roughness: 0.2,
            ..default()
        }),
    ];

    // Spawn cubes in a grid pattern
    let positions = [
        (10.0, 3.0, 10.0),
        (-10.0, 4.0, 10.0),
        (10.0, 5.0, -10.0),
        (-10.0, 6.0, -10.0),
        (20.0, 5.0, 5.0),
        (-5.0, 7.0, 15.0),
        (15.0, 4.0, -20.0),
        (-15.0, 3.0, -15.0),
    ];

    for (i, (x, y, z)) in positions.iter().enumerate() {
        let material = cube_materials[i % cube_materials.len()].clone();
        let offset = (i as f32) * 0.5; // Different phase for each cube

        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(*x, *y, *z),
            Collider::cuboid(0.5, 0.5, 0.5),
            RigidBody::KinematicPositionBased,
            FloatingCube {
                initial_y: *y,
                offset,
            },
        ));
    }
}

fn spawn_npcs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cylinder_mesh = meshes.add(Cylinder::new(0.5, 2.0));

    // Create materials for NPCs with different colors
    let npc_materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.6, 0.3),
            perceptual_roughness: 0.4,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.3, 0.9),
            perceptual_roughness: 0.4,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.9, 0.6),
            perceptual_roughness: 0.4,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.3, 0.3),
            perceptual_roughness: 0.4,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.9),
            perceptual_roughness: 0.4,
            ..default()
        }),
    ];

    // NPC location clusters
    let npc_clusters = [
        Vec3::new(-25.0, 0.0, 25.0),  // North-west corner
        Vec3::new(25.0, 0.0, 25.0),   // North-east corner
        Vec3::new(-25.0, 0.0, -25.0), // South-west corner
        Vec3::new(25.0, 0.0, -25.0),  // South-east corner
        Vec3::new(0.0, 0.0, 0.0),     // Center
    ];

    // Name options
    let npc_names = [
        "Marcus",
        "Olivia",
        "Zoe",
        "Ethan",
        "Lily",
        "Noah",
        "Emily",
        "Aiden",
        "Sophia",
        "Jacob",
        "Emma",
        "Jackson",
        "Dr. Neutrino",
        "The Observer",
        "Merchant Tom",
        "Guard Steve",
        "Villager Bob",
    ];

    // Dialogue types to assign
    let dialogue_types = ["basic", "guard", "merchant", "scientist", "mysterious"];

    let mut rng = rand::rng();

    for i in 0..NPC_COUNT {
        // Choose a random cluster location as the home position
        let cluster_index = i % npc_clusters.len();
        let home_cluster = npc_clusters[cluster_index];

        // Add some randomness to the exact position within the cluster
        let offset = Vec3::new(
            rng.random_range(-5.0..5.0),
            0.0,
            rng.random_range(-5.0..5.0),
        );

        let home_position = home_cluster + offset;
        let y_position = 1.0; // Half the height of the cylinder

        // Generate initial target position
        let target_offset = Vec3::new(
            rng.random_range(-NPC_WANDER_RADIUS..NPC_WANDER_RADIUS),
            0.0,
            rng.random_range(-NPC_WANDER_RADIUS..NPC_WANDER_RADIUS),
        );

        let target_position = home_position + target_offset;

        // Choose material, name, and dialogue type
        // Assign dialogue types in a pattern to ensure we use all types
        let dialogue_type_index = i % dialogue_types.len();
        let dialogue_id = dialogue_types[dialogue_type_index].to_string();

        // Match names with dialogue types more appropriately
        let name_index = match dialogue_id.as_str() {
            "scientist" => 12,  // Dr. Neutrino
            "mysterious" => 13, // The Observer
            "merchant" => 14,   // Merchant Tom
            "guard" => 15,      // Guard Steve
            _ => i % 12,        // Regular names for basic NPCs
        };

        let name = npc_names[name_index].to_string();

        // Choose material based on NPC type
        let material_index = match dialogue_id.as_str() {
            "scientist" => 4,  // Blue for scientists
            "mysterious" => 1, // Purple for the mysterious ones
            "merchant" => 2,   // Green for merchants
            "guard" => 3,      // Red for guards
            _ => 0,            // Brown for basic NPCs
        };

        let material = npc_materials[material_index].clone();

        commands.spawn((
            Mesh3d(cylinder_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(home_position.x, y_position, home_position.z),
            Collider::cylinder(1.0, 0.5),
            RigidBody::KinematicPositionBased,
            Npc {
                home_position: Vec3::new(home_position.x, y_position, home_position.z),
                target_position: Vec3::new(target_position.x, y_position, target_position.z),
                movement_timer: Timer::from_seconds(rng.random_range(5.0..10.0), TimerMode::Once),
                name,
                dialogue_id,
            },
        ));
    }
}

fn update_floating_cubes(time: Res<Time>, mut cubes: Query<(&mut Transform, &FloatingCube)>) {
    let t = time.elapsed_secs();

    for (mut transform, cube) in cubes.iter_mut() {
        // Calculate new y position with sine wave
        let new_y = cube.initial_y
            + CUBE_FLOAT_AMPLITUDE * (CUBE_FLOAT_FREQUENCY * (t + cube.offset) * PI).sin();

        transform.translation.y = new_y;

        // Also add a gentle rotation over time
        transform.rotate_y(0.005);
    }
}

fn update_npcs(time: Res<Time>, mut npcs: Query<(&mut Transform, &mut Npc)>) {
    let mut rng = rand::rng();

    for (mut transform, mut npc) in npcs.iter_mut() {
        // Update timer
        npc.movement_timer.tick(time.delta());

        if npc.movement_timer.just_finished() {
            // Choose a new random target position
            let target_offset = Vec3::new(
                rng.random_range(-NPC_WANDER_RADIUS..NPC_WANDER_RADIUS),
                0.0,
                rng.random_range(-NPC_WANDER_RADIUS..NPC_WANDER_RADIUS),
            );

            npc.target_position = npc.home_position + target_offset;

            // Reset timer with random duration
            npc.movement_timer = Timer::from_seconds(rng.random_range(5.0..10.0), TimerMode::Once);
        }

        // Move towards target position
        let direction = npc.target_position - transform.translation;

        if direction.length() > 0.1 {
            // Normalize and scale by speed and delta time
            let movement = direction.normalize() * NPC_WANDER_SPEED * time.delta_secs();

            // Move the NPC
            transform.translation += movement;

            // Rotate to face movement direction (only in xz plane)
            let target_rotation = Quat::from_rotation_y(f32::atan2(direction.x, direction.z));
            transform.rotation = transform.rotation.slerp(target_rotation, 0.1);
        }
    }
}

// Player interaction to start dialogues with NPCs
fn player_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<KinematicCharacterController>>,
    camera_query: Query<&Transform, With<Camera>>,
    npc_query: Query<(&Transform, Entity, &Npc), With<Npc>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    dialogue_db: Res<DialogueDatabase>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        let Ok(player_transform) = player_query.get_single() else {
            return;
        };
        let Ok(camera_transform) = camera_query.get_single() else {
            return;
        };

        // Get the camera's global transform
        let global_transform = player_transform.mul_transform(*camera_transform);

        // Ray starts at the camera position
        let ray_pos = global_transform.translation;
        // Ray points in the camera's forward direction
        let ray_dir = global_transform.forward();

        // Simple distance-based check for nearby NPCs
        // Find the closest NPC that's in front of the player and within range
        let mut closest_npc = None;
        let mut closest_distance = f32::MAX;

        for (npc_transform, entity, npc) in npc_query.iter() {
            let to_npc = npc_transform.translation - ray_pos;

            // Check if the NPC is roughly in front of the player (dot product > 0)
            let forward_dot = ray_dir.dot(to_npc.normalize());
            if forward_dot > 0.7 {
                // Within ~45 degrees of forward direction
                let distance = to_npc.length();

                if distance < INTERACTION_DISTANCE && distance < closest_distance {
                    closest_distance = distance;
                    closest_npc = Some((entity, npc));
                }
            }
        }

        // If we found an NPC to interact with, start dialogue
        if let Some((entity, npc)) = closest_npc {
            println!("Starting dialogue with NPC: {}", npc.name);

            // Get the dialogue tree for this NPC
            if let Some(dialogue_tree) = dialogue_db.dialogues.get(&npc.dialogue_id) {
                // Store the active dialogue information starting with the root node
                commands.spawn(ActiveDialogue {
                    npc_entity: entity,
                    current_node: dialogue_tree.root_node.clone(),
                });

                // Change to dialogue state
                next_state.set(GameState::InDialogue);
            } else {
                println!("Error: No dialogue tree found for id: {}", npc.dialogue_id);
            }
        }
    }
}

// Setup the dialogue UI when entering dialogue state
fn setup_dialogue_ui(
    mut commands: Commands,
    active_dialogue_query: Query<(&ActiveDialogue, Entity)>,
    npc_query: Query<&Npc>,
    dialogue_db: Res<DialogueDatabase>,
    mut windows: Query<&mut Window>,
    look_input: Res<LookInput>,
    mut stored_camera: ResMut<StoredCameraState>,
) {
    // Store current camera rotation before entering dialogue
    stored_camera.look_rotation = Vec2::new(look_input.x, look_input.y);

    // Unlock the cursor during dialogue
    let mut window = windows.single_mut();
    window.cursor_options.visible = true;
    window.cursor_options.grab_mode = bevy::window::CursorGrabMode::None;

    // Get the active dialogue
    let Ok((active_dialogue, _)) = active_dialogue_query.get_single() else {
        return;
    };

    // Get the NPC we're talking to
    let Ok(npc) = npc_query.get(active_dialogue.npc_entity) else {
        return;
    };

    // Get the dialogue data
    let Some(dialogue_tree) = dialogue_db.dialogues.get(&npc.dialogue_id) else {
        println!("Error: No dialogue tree found for id: {}", npc.dialogue_id);
        return;
    };

    // Get the current dialogue node
    let Some(node) = dialogue_tree.nodes.get(&active_dialogue.current_node) else {
        println!(
            "Error: No node found with id: {}",
            active_dialogue.current_node
        );
        return;
    };

    // Create the dialogue UI
    commands
        .spawn((
            Node {
                width: Val::Percent(50.0),
                height: Val::Auto,
                position_type: PositionType::Absolute,
                left: Val::Percent(25.0),
                bottom: Val::Percent(20.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(DIALOGUE_BACKGROUND_COLOR),
            DialogueUI,
        ))
        .with_children(|parent| {
            // NPC name
            parent.spawn((
                Text::new(npc.name.clone()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Dialogue text
            parent.spawn((
                Text::new(node.text.clone()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Dialogue options
            for (i, option) in node.options.iter().enumerate() {
                let option_text = match option {
                    DialogueOption::Reply { text, .. } => text.clone(),
                    DialogueOption::Exit { text } => text.clone(),
                };

                let target_node = match option {
                    DialogueOption::Reply { target_node, .. } => target_node.clone(),
                    DialogueOption::Exit { .. } => "exit".to_string(),
                };

                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::left(Val::Px(10.0)),
                            margin: UiRect::bottom(Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(DIALOGUE_OPTION_NORMAL_COLOR),
                        DialogueOptionButton {
                            target_node,
                            option_index: i,
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(format!("{}. {}", i + 1, option_text)),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

// Handle hover effects on dialogue options
fn handle_dialogue_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DialogueOptionButton>),
    >,
) {
    for (interaction, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *background_color = BackgroundColor(DIALOGUE_OPTION_HOVER_COLOR);
            }
            _ => {
                *background_color = BackgroundColor(DIALOGUE_OPTION_NORMAL_COLOR);
            }
        }
    }
}

// Handle clicks on dialogue options
fn handle_dialogue_click(
    interaction_query: Query<(&Interaction, &DialogueOptionButton), Changed<Interaction>>,
    active_dialogue_query: Query<(&ActiveDialogue, Entity)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialogue_db: Res<DialogueDatabase>,
    npc_query: Query<&Npc>,
    dialogue_ui_query: Query<Entity, With<DialogueUI>>,
) {
    // Check for Escape key to exit dialogue
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Ok((_, active_dialogue_entity)) = active_dialogue_query.get_single() {
            commands.entity(active_dialogue_entity).despawn();
            next_state.set(GameState::Playing);
            return;
        }
    }

    // Handle button clicks
    for (interaction, dialogue_option) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let Ok((active_dialogue, active_dialogue_entity)) = active_dialogue_query.get_single()
            else {
                return;
            };

            if dialogue_option.target_node == "exit" {
                // Exit dialogue
                commands.entity(active_dialogue_entity).despawn();
                next_state.set(GameState::Playing);
            } else {
                // Update the current dialogue node
                commands
                    .entity(active_dialogue_entity)
                    .insert(ActiveDialogue {
                        npc_entity: active_dialogue.npc_entity,
                        current_node: dialogue_option.target_node.clone(),
                    });

                // Redraw UI directly instead of toggling game states
                // First, remove the old UI
                for entity in dialogue_ui_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Get the necessary data for drawing the new UI
                let Ok(npc) = npc_query.get(active_dialogue.npc_entity) else {
                    return;
                };

                // Get the dialogue data
                let Some(dialogue_tree) = dialogue_db.dialogues.get(&npc.dialogue_id) else {
                    println!("Error: No dialogue tree found for id: {}", npc.dialogue_id);
                    return;
                };

                // Get the new dialogue node
                let Some(node) = dialogue_tree.nodes.get(&dialogue_option.target_node) else {
                    println!(
                        "Error: No node found with id: {}",
                        dialogue_option.target_node
                    );
                    return;
                };

                // Create the new dialogue UI with the updated node
                commands
                    .spawn((
                        Node {
                            width: Val::Percent(50.0),
                            height: Val::Auto,
                            position_type: PositionType::Absolute,
                            left: Val::Percent(25.0),
                            bottom: Val::Percent(20.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(DIALOGUE_BACKGROUND_COLOR),
                        DialogueUI,
                    ))
                    .with_children(|parent| {
                        // NPC name
                        parent.spawn((
                            Text::new(npc.name.clone()),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            Node {
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                        ));

                        // Dialogue text
                        parent.spawn((
                            Text::new(node.text.clone()),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            Node {
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                        ));

                        // Dialogue options
                        for (i, option) in node.options.iter().enumerate() {
                            let option_text = match option {
                                DialogueOption::Reply { text, .. } => text.clone(),
                                DialogueOption::Exit { text } => text.clone(),
                            };

                            let target_node = match option {
                                DialogueOption::Reply { target_node, .. } => target_node.clone(),
                                DialogueOption::Exit { .. } => "exit".to_string(),
                            };

                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(30.0),
                                        justify_content: JustifyContent::FlexStart,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::left(Val::Px(10.0)),
                                        margin: UiRect::bottom(Val::Px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(DIALOGUE_OPTION_NORMAL_COLOR),
                                    DialogueOptionButton {
                                        target_node,
                                        option_index: i,
                                    },
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(format!("{}. {}", i + 1, option_text)),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                    ));
                                });
                        }
                    });
            }
        }
    }
}

// Cleanup the dialogue UI when exiting dialogue state
fn cleanup_dialogue_ui(mut commands: Commands, dialogue_ui_query: Query<Entity, With<DialogueUI>>) {
    // Find and remove all dialogue UI entities
    for entity in dialogue_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Reset the look input when exiting dialogue to prevent camera from changing position
fn reset_look_input(mut look: ResMut<LookInput>, stored_camera: Res<StoredCameraState>) {
    // Restore the exact camera rotation from before entering dialogue
    look.x = stored_camera.look_rotation.x;
    look.y = stored_camera.look_rotation.y;
}
