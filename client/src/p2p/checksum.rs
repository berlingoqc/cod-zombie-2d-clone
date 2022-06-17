
use bevy::prelude::*;
use bevy_ggrs::Rollback;
use shared::{zombies::zombie::Zombie, utils::{Checksum, fletcher16}};



pub fn checksum_zombie(
    mut query: Query<(&Transform, &Zombie, &mut Checksum), With<Rollback>>,
) {
    for (t, v, mut checksum) in query.iter_mut() {
        let translation = t.translation;
        let mut bytes = Vec::with_capacity(28);
        bytes.extend_from_slice(&translation.x.to_le_bytes());
        bytes.extend_from_slice(&translation.y.to_le_bytes());
        bytes.extend_from_slice(&translation.z.to_le_bytes()); // this z will probably never matter, but removing it probably also will not matter...

		bytes.extend_from_slice(&[v.state.clone() as u8]);

        // naive checksum implementation
        checksum.value = fletcher16(&bytes);
    }
}
