use bevy::{
    diagnostic::{Diagnostics, DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};


#[derive(Component)]
pub struct FPSTextComponent();

fn counter_system(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FPSTextComponent>>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("{:.2}", average);
            }
        }
    };
}

fn setup_counter_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((FPSTextComponent{}, TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "\nAverage FPS: ".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 26.0,
                    color: Color::rgb(0.0, 1.0, 0.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    }));
}

pub struct FPSPlugin {}

impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_counter_text)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(LogDiagnosticsPlugin::default())
            .add_systems(Update, counter_system);
    }
}
