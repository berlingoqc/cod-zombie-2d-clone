use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{map::{MapElementPosition, Window, WindowPanel, Size}, health::Health};

use super::Player;

#[derive(Component)]
pub struct PlayerCurrentInteraction {
    // tell if or not there is an interaction available for the user
    pub interaction: bool,
    // cooldown between each interaction
    pub interaction_cooldown: f32,
    // entity that has the interaction component
    pub entity: Entity,
    pub child_entity: Entity,
    // type of interaction
    pub interaction_type: PlayerInteractionType,

    // tell if the player is doing the interaction
    pub interacting: bool,

    // when the user last trigger the interaction
    pub interaction_trigger_at: f32,
}

#[derive(Default, Clone, Copy)]
pub enum PlayerInteractionType {
    #[default]
    None = 0,

    RepairWindow,
}

#[derive(Default, Component)]
pub struct PlayerInteraction {
    pub interaction_available: bool,
    // the type of interaction , use to find the right handler for the action
    pub interaction_type: PlayerInteractionType,
    // the size of the zone where the player can trigger the animatin arround
    // the position of the interaction entity
    pub interaction_size: Vec2,
    // timeout before the interaction can be trigger again
    pub interaction_timeout: f32
}

pub fn system_interaction_player(
    mut query_player: Query<(&Transform, &mut PlayerCurrentInteraction), With<Player>>,
    time: Res<Time>,
    interaction_query: Query<
        (Entity, &Transform, &MapElementPosition, &PlayerInteraction),
        (
            With<MapElementPosition>,
            Without<Player>,
        ),
    >,

    keyboard_input: Res<Input<KeyCode>>,

    mut query_window: Query<(&mut Window, &Children)>,
    mut query_panel: Query<(&mut WindowPanel, &Size, &mut Health, &mut Sprite)>
) {

    for (player_transform, mut interaction) in query_player.iter_mut() {
        for (entity, transform, info, player_interaction) in interaction_query.iter() {
            let collision = collide(player_transform.translation, Vec2::new(25., 25.), info.position.extend(10.),  player_interaction.interaction_size);
            if collision.is_some() && player_interaction.interaction_available {
                // notify use that key perform action
                interaction.interaction = true;
                interaction.entity = entity.clone();
                interaction.interaction_type = player_interaction.interaction_type;
                interaction.interaction_cooldown = player_interaction.interaction_timeout;
            } else {
                if entity.id() == interaction.entity.id() {
                    match interaction.interaction_type {
                        PlayerInteractionType::RepairWindow => {
                            if interaction.interacting == true {
                                // TODO : duplicatate code
                                let (_,size, mut health, mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                                interaction.interacting = false;
                                health.current_health = 0.;
                                sprite.custom_size = Some(Vec2::new(0.,0.));
                            }
                        },
                        _ => {}
                    }

                    interaction.interaction = false;
                    interaction.interacting = false;
                    interaction.entity = Entity::from_raw(0);
                }
            }
        } 


        if interaction.interaction {
            if keyboard_input.pressed(KeyCode::F) {
                match interaction.interaction_type {
                    PlayerInteractionType::RepairWindow => {
                        if interaction.interacting == true {
                            // repair the window
                            let time_since_startup = time.time_since_startup().as_secs_f32();
                            if interaction.interaction_trigger_at + interaction.interaction_cooldown <= time_since_startup {
                                let (_,size, mut health, mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                                sprite.custom_size = Some(size.0);
                                health.current_health = 1.;
                                interaction.interacting = false;

                                if let Ok((mut window, _)) = query_window.get_mut(interaction.entity) {
                                    if window.destroy {
                                        window.destroy = false;
                                    }
                                }
                            } else {
                                let (_,size, _ , mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                                let time_diff = time_since_startup - (interaction.interaction_trigger_at + interaction.interaction_cooldown);
                                let percentage_time_diff_cooldown = 1. - (time_diff / interaction.interaction_cooldown);
                                sprite.custom_size = Some(size.0 / percentage_time_diff_cooldown);
                            }
                        } else {
                            let (_, children) = query_window.get(interaction.entity).unwrap();

                            for &child_entity in children.iter() {
                                let (_,size, mut health, mut sprite) = query_panel.get_mut(child_entity).unwrap();
                                if health.current_health <= 0. {
                                    // there is a panel to repair
                                    interaction.interacting = true;
                                    interaction.child_entity = child_entity.clone();
                                    interaction.interaction_trigger_at = time.time_since_startup().as_secs_f32();
                                    break;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            } else {
                if interaction.interacting {
                    interaction.interacting = false;
                    match interaction.interaction_type {
                        PlayerInteractionType::RepairWindow => {
                            let (_,size, mut health, mut sprite) = query_panel.get_mut(interaction.child_entity).unwrap();
                            health.current_health = 0.;
                            sprite.custom_size = Some(Vec2::new(0.,0.));
                        },
                        _ => {}
                    }
                }
            }
        }
    }

}

